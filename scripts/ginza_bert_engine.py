"""BERT 批次预计算与 CPU 结构解码；共享单模型，按单元组合结果。"""
from __future__ import annotations

from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor, wait
from collections import deque
from multiprocessing import get_context, shared_memory
from pathlib import Path
import os
import sys
import time

from ginza_batching import decode_groups, length_batches, request_batches

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "experiments/ginza-optimized"))
MODEL = "ja_ginza_bert_large"


def configure_environment(threads):
    for name in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"):
        os.environ[name] = str(threads)
    os.environ["HF_HUB_OFFLINE"] = "1"
    os.environ["TRANSFORMERS_OFFLINE"] = "1"
    os.environ["TOKENIZERS_PARALLELISM"] = "false"
    if os.name == "nt":
        os.environ["CUDA_PATH"] = str(Path(sys.prefix) / "Lib/site-packages/nvidia/cuda_runtime")


def load_gpu(precision="fp16", threads=1, max_batch_items=8192):
    configure_environment(threads)
    import torch
    torch.set_num_threads(threads)
    if os.name == "nt":
        torchlib = Path(torch.__file__).parent / "lib"
        os.add_dll_directory(str(torchlib))
        os.environ["CUDA_PATH"] = str(torchlib.parent.parent / "nvidia/cuda_runtime")
    import spacy
    from thinc.api import set_gpu_allocator
    set_gpu_allocator("pytorch")
    spacy.require_gpu(0)
    nlp = spacy.load(MODEL)
    transformer = nlp.get_pipe("transformer")
    transformer.cfg["max_batch_items"] = max_batch_items
    if precision == "fp16":
        hf = transformer.model.transformer
        hf.half()
        def output_float32(module, inputs, output):
            output["last_hidden_state"] = output.last_hidden_state.float()
            return output
        hf.register_forward_hook(output_float32)
    elif precision == "amp":
        transformer.model.layers[0].shims[0]._mixed_precision = True
    return nlp


CPU_NLP = None
CPU_LIMIT = None


def initialize_cpu(threads=1):
    global CPU_NLP, CPU_LIMIT
    configure_environment(threads)
    import spacy
    from threadpoolctl import threadpool_limits
    spacy.require_cpu()
    CPU_LIMIT = threadpool_limits(limits=threads)
    CPU_NLP = spacy.load(MODEL, exclude=["transformer"])
    return os.getpid()


def unpack_docs(nlp, records):
    from spacy.tokens import Doc
    docs = []
    for data, user_data in records:
        doc = Doc(nlp.vocab).from_bytes(data)
        doc.user_data.update(user_data)
        docs.append(doc)
    return docs


def decode_batch(records, arrays, biases, morph, implementation="original", token_offset=0):
    import numpy as np
    from thinc.api import Model, NumpyOps
    nlp = CPU_NLP
    started = time.perf_counter()
    docs = unpack_docs(nlp, records)
    timings = {}
    for name in ("parser", "ner"):
        stage_start = time.perf_counter()
        component = nlp.get_pipe(name)
        if implementation != "original":
            raise ValueError("生产解码仅使用 spaCy 原版状态解码")
        else:
            # spaCy 原版状态解码读取同一份 GPU 预计算的连续数组。
            model = component.model
            tok2vec = model.layers[0]
            lower = model.layers[1]
            flat = np.zeros((sum(map(len, docs)), lower.get_dim("nI")), dtype="float32")
            model._layers[0] = Model("prepared_tokens", lambda m, x, is_train: (flat, lambda d: []), ops=NumpyOps())
            original_forward = lower._func
            count = sum(map(len, docs))
            prepared = np.concatenate([arrays[name][:1], arrays[name][token_offset + 1:token_offset + count + 1]])
            lower._func = lambda m, x, is_train: (prepared, lambda d: [])
            try:
                component.set_annotations(docs, component.predict(docs))
            finally:
                lower._func = original_forward
                model._layers[0] = tok2vec
        timings[name] = time.perf_counter() - stage_start
    nlp.get_pipe("morphologizer").set_annotations(docs, morph)
    for name in ("compound_splitter", "bunsetu_recognizer"):
        component = nlp.get_pipe(name)
        docs = [component(doc) for doc in docs]
    records = [(doc.to_bytes(exclude=["user_data"]), doc.user_data) for doc in docs]
    seconds = time.perf_counter() - started
    timings.update(task_started=started, task_finished=started + seconds,
                   documents=len(docs), tokens=sum(map(len, docs)), worker_pid=os.getpid())
    return records, seconds, timings


def decode_shared(records, descriptors, biases, morph, implementation, token_offset, batch_index, submitted):
    import numpy as np
    handles = [shared_memory.SharedMemory(name=item["name"]) for item in descriptors.values()]
    arrays = {key: np.ndarray(item["shape"], dtype="float32", buffer=handle.buf)
              for (key, item), handle in zip(descriptors.items(), handles)}
    try:
        result = decode_batch(records, arrays, biases, morph, implementation, token_offset)
        result[2].update(batch_index=batch_index, task_submitted=submitted, token_offset=token_offset)
        return result
    finally:
        arrays.clear()
        for handle in handles:
            handle.close()


class FeatureSlot:
    """一个批次独占的 pinned staging 与共享特征缓冲；由 engine 释放。"""
    def __init__(self, shapes, capacity):
        import cupy as cp
        import numpy as np
        self.handles, self.pinned, self.host, self.shared = {}, {}, {}, {}
        try:
            for name, shape in shapes.items():
                shape = (capacity + 1, *shape)
                size = int(np.prod(shape)) * 4
                self.handles[name] = shared_memory.SharedMemory(create=True, size=size)
                self.shared[name] = np.ndarray(shape, dtype="float32", buffer=self.handles[name].buf)
                self.pinned[name] = cp.cuda.alloc_pinned_memory(size)
                self.host[name] = np.ndarray(shape, dtype="float32", buffer=self.pinned[name])
        except BaseException:
            self.close()
            raise

    def close(self):
        self.host.clear()
        self.shared.clear()
        self.pinned.clear()
        for handle in self.handles.values():
            handle.close()
            handle.unlink()
        self.handles.clear()


class BertEngine:
    def __init__(self, nlp, workers=0, threads=1, implementation="original",
                 async_d2h=True, buffer_tokens=8192):
        self.nlp = nlp
        self.implementation = implementation
        self.pool = None
        self.workers = workers
        self.buffer_tokens = buffer_tokens
        self.slots = []
        self.publisher = None
        self.copy_stream = None
        if implementation != "original" or workers < 0 or buffer_tokens < 1:
            raise ValueError("解码模式、worker 数或缓冲容量无效")
        for name in ("parser", "ner"):
            component = nlp.get_pipe(name)
            if component.model.attrs["has_upper"] or component.model.attrs["unseen_classes"]:
                raise ValueError(f"{name} 模型不支持预计算解码")
        self.biases = {name: nlp.get_pipe(name).model.get_ref("lower").get_param("b").get()
                       for name in ("parser", "ner")}
        if workers and async_d2h:
            import cupy as cp
            shapes = {name: tuple(nlp.get_pipe(name).model.get_ref("lower").get_dim(dim)
                                  for dim in ("nF", "nO", "nP")) for name in ("parser", "ner")}
            try:
                for _ in range(2):
                    self.slots.append(FeatureSlot(shapes, buffer_tokens))
            except BaseException:
                self.close()
                raise
            self.copy_stream = cp.cuda.Stream(non_blocking=True)
            self.publisher = ThreadPoolExecutor(1, thread_name_prefix="ginza-transfer")
        try:
            if workers:
                self.pool = ProcessPoolExecutor(workers, mp_context=get_context("spawn"),
                                                initializer=initialize_cpu, initargs=(threads,))
                # 显式预热所有 worker，测量区间复用进程。
                futures = [self.pool.submit(time.sleep, .2) for _ in range(workers)]
                for future in futures:
                    future.result()
            else:
                initialize_cpu(threads)
                import spacy
                spacy.require_gpu(0)
        except BaseException:
            self.close()
            raise
        self.metrics = {}

    def prepare(self, docs, slot=None):
        import cupy as cp
        import numpy as np
        import torch
        started = time.perf_counter()
        gpu_start = cp.cuda.Event()
        gpu_end = cp.cuda.Event()
        gpu_start.record()
        with torch.inference_mode():
            transformer = self.nlp.get_pipe("transformer")
            with torch.profiler.record_function("ginza.bert"):
                docs = list(transformer.pipe(docs, batch_size=len(docs)))
            bert_end = cp.cuda.Event()
            bert_end.record()
            device_arrays = {}
            with torch.profiler.record_function("ginza.heads"):
                for name in ("parser", "ner"):
                    model = self.nlp.get_pipe(name).model
                    tokens = model.layers[0].predict(docs)
                    device_arrays[name] = model.layers[1].predict(tokens)
                morph = self.nlp.get_pipe("morphologizer").predict(docs)
            gpu_end.record()
            transfer_started = time.perf_counter()
            transfer = None
            if slot is None:
                gpu_end.synchronize()
                arrays = {name: np.ascontiguousarray(value.get()) for name, value in device_arrays.items()}
                self.metrics["d2h_seconds"] += time.perf_counter() - transfer_started
                self._event_metrics(gpu_start, bert_end, gpu_end)
            else:
                arrays = {name: slot.host[name][:len(value)] for name, value in device_arrays.items()}
                self.copy_stream.wait_event(gpu_end)
                copy_start, copy_end = cp.cuda.Event(), cp.cuda.Event()
                with torch.profiler.record_function("ginza.d2h_enqueue"), self.copy_stream:
                    copy_start.record()
                    for name, value in device_arrays.items():
                        value.get(out=arrays[name], stream=self.copy_stream, blocking=False)
                    copy_end.record()
                transfer = (copy_start, copy_end, gpu_start, bert_end, gpu_end, device_arrays)
        for doc in docs:
            doc._.trf_data = None
            doc.user_data.pop(("._.", "trf_data", None, None), None)
        records = [(doc.to_bytes(exclude=["user_data"]), doc.user_data) for doc in docs]
        self.metrics["gpu_prepare_seconds"] += time.perf_counter() - started
        self.metrics["transfer_bytes"] += sum(array.nbytes for array in arrays.values())
        self.metrics["batches"].append({"documents": len(docs), "tokens": sum(map(len, docs)),
                                         "prepare_started": started, "prepare_finished": time.perf_counter(),
                                         "slot": len(self.metrics["batches"]) % 2 if slot else None,
                                         "d2h_enqueued": transfer_started,
                                         "transfer_bytes": sum(array.nbytes for array in arrays.values())})
        if slot is None:
            self.metrics["batches"][-1].update(
                gpu_event_seconds=cp.cuda.get_elapsed_time(gpu_start, gpu_end) / 1000,
                bert_interval_seconds=cp.cuda.get_elapsed_time(gpu_start, bert_end) / 1000,
                heads_interval_seconds=cp.cuda.get_elapsed_time(bert_end, gpu_end) / 1000)
        return records, arrays, morph, [len(doc) for doc in docs], transfer

    def _event_metrics(self, gpu_start, bert_end, gpu_end):
        import cupy as cp
        self.metrics["gpu_compute_seconds"] += cp.cuda.get_elapsed_time(gpu_start, gpu_end) / 1000
        self.metrics["bert_interval_seconds"] += cp.cuda.get_elapsed_time(gpu_start, bert_end) / 1000
        self.metrics["heads_interval_seconds"] += cp.cuda.get_elapsed_time(bert_end, gpu_end) / 1000

    def submit(self, prepared, slot, batch_index):
        import numpy as np
        import cupy as cp
        records, arrays, morph, lengths, transfer = prepared
        event = self.metrics["batches"][batch_index]
        if transfer:
            copy_start, copy_end, gpu_start, bert_end, gpu_end, device_arrays = transfer
            copy_end.synchronize()
            event["d2h_ready"] = time.perf_counter()
            event["d2h_event_seconds"] = cp.cuda.get_elapsed_time(copy_start, copy_end) / 1000
            self.metrics["d2h_seconds"] += event["d2h_event_seconds"]
            self._event_metrics(gpu_start, bert_end, gpu_end)
            event.update(gpu_event_seconds=cp.cuda.get_elapsed_time(gpu_start, gpu_end) / 1000,
                         bert_interval_seconds=cp.cuda.get_elapsed_time(gpu_start, bert_end) / 1000,
                         heads_interval_seconds=cp.cuda.get_elapsed_time(bert_end, gpu_end) / 1000)
            device_arrays.clear()
        started = time.perf_counter()
        if self.pool is None:
            result = decode_batch(records, arrays, self.biases, morph, self.implementation)
            result[2]["batch_index"] = batch_index
            return result, [], batch_index
        handles = []
        descriptors = {}
        futures = []
        try:
            for key, array in arrays.items():
                if slot:
                    handle = slot.handles[key]
                    np.copyto(slot.shared[key][:len(array)], array)
                else:
                    handle = shared_memory.SharedMemory(create=True, size=array.nbytes)
                    handles.append(handle)
                    np.ndarray(array.shape, dtype="float32", buffer=handle.buf)[:] = array
                descriptors[key] = {"name": handle.name, "shape": array.shape}
            event["shared_ready"] = time.perf_counter()
            for start, end, _ in decode_groups(lengths, self.workers):
                submitted = time.perf_counter()
                future = self.pool.submit(decode_shared, records[start:end], descriptors,
                                          self.biases, morph[start:end], self.implementation,
                                          sum(lengths[:start]), batch_index, submitted)
                futures.append((start, future))
        except BaseException:
            wait([future for _, future in futures])
            for handle in handles:
                handle.close()
                handle.unlink()
            raise
        self.metrics["submit_seconds"] += time.perf_counter() - started
        event["submit_finished"] = time.perf_counter()
        return sorted(futures), handles, batch_index

    def collect(self, job):
        started = time.perf_counter()
        if self.publisher:
            try:
                job = job.result()
            except BaseException:
                self.metrics["publish_wait_seconds"] += time.perf_counter() - started
                raise
        pending, handles, batch_index = job
        self.metrics["publish_wait_seconds"] += time.perf_counter() - started
        self.metrics["batches"][batch_index]["collect_started"] = started
        started = time.perf_counter()
        try:
            results = [future.result() for _, future in pending] if self.pool else [pending]
            self.metrics["decode_tasks"].extend(timings for _, _, timings in results)
            self.metrics["cpu_decode_seconds"] += sum(seconds for _, seconds, _ in results)
            for name in ("parser", "ner"):
                self.metrics[name + "_decode_seconds"] += sum(timings[name] for _, _, timings in results)
                self.metrics[name + "_kernel_seconds"] += sum(timings.get(name + "_kernel", 0) for _, _, timings in results)
            self.metrics["cpu_wait_seconds"] += time.perf_counter() - started
            self.metrics["batches"][batch_index]["collected"] = time.perf_counter()
            return [record for records, _, _ in results for record in records]
        finally:
            if self.pool:
                wait([future for _, future in pending])
            for handle in handles:
                handle.close()
                handle.unlink()

    def analyze(self, texts, batch_size=32, overlap=True, sort_lengths=True):
        started = time.perf_counter()
        indexed = list(enumerate(self.nlp.tokenizer.pipe(texts)))
        tokenizer_seconds = time.perf_counter() - started
        batches = length_batches(indexed, batch_size, self.buffer_tokens, sort_lengths)
        return self._analyze_batches(batches, len(indexed), tokenizer_seconds, overlap)

    def analyze_requests(self, requests, batch_size=32, bucket_width=64):
        """一个 GPU 所有者合批多个已接收请求，按 request/unit 身份返回结果。"""
        request_ids = [request["request_id"] for request in requests]
        if len(set(request_ids)) != len(request_ids):
            raise ValueError("request_id 必须唯一")
        started = time.perf_counter()
        queues, identities, offset = [], [], 0
        for request in requests:
            units = request["units"]
            if len({unit["id"] for unit in units}) != len(units):
                raise ValueError("请求内 unit id 必须唯一")
            docs = list(self.nlp.tokenizer.pipe(unit["text"] for unit in units))
            queues.append(list(zip(range(offset, offset + len(docs)), docs)))
            identities.extend((request["request_id"], unit["id"]) for unit in units)
            offset += len(docs)
        tokenizer_seconds = time.perf_counter() - started
        batches = request_batches(queues, batch_size, self.buffer_tokens, bucket_width)
        docs = self._analyze_batches(batches, offset, tokenizer_seconds, True, identities)
        result, offset = [], 0
        for request in requests:
            count = len(request["units"])
            result.append({"request_id": request["request_id"], "units": [
                {"id": unit["id"], "doc": doc}
                for unit, doc in zip(request["units"], docs[offset:offset + count])]})
            offset += count
        return result

    def _analyze_batches(self, batches, count, tokenizer_seconds, overlap, identities=None):
        self.metrics = {"gpu_prepare_seconds": 0., "gpu_compute_seconds": 0.,
                        "cpu_decode_seconds": 0., "cpu_wait_seconds": 0., "transfer_bytes": 0}
        self.metrics.update(parser_decode_seconds=0., ner_decode_seconds=0.)
        self.metrics.update(parser_kernel_seconds=0., ner_kernel_seconds=0.)
        self.metrics.update(bert_interval_seconds=0., heads_interval_seconds=0.,
                            d2h_seconds=0., submit_seconds=0., publish_wait_seconds=0.)
        self.metrics.update(batches=[], decode_tasks=[])
        self.metrics["tokenizer_seconds"] = tokenizer_seconds
        self.metrics["buffer_bytes"] = sum(handle.size for slot in self.slots for handle in slot.handles.values()) * 2
        output = [None] * count
        pending = deque()
        # 两个在途批次允许 CPU 解码与下一批 GPU 准备重叠。
        depth = 2 if self.pool and overlap else 1
        def finish():
            indices, job = pending.popleft()
            for index, record in zip(indices, self.collect(job)):
                output[index] = record
        try:
            for batch_index, batch in enumerate(batches):
                if len(pending) >= depth:
                    finish()
                indices, docs = zip(*batch)
                slot = self.slots[batch_index % depth] if self.slots else None
                prepared = self.prepare(list(docs), slot)
                event = self.metrics["batches"][-1]
                event["unit_indices"] = list(indices)
                if identities:
                    event["identities"] = [identities[index] for index in indices]
                job = (self.publisher.submit(self.submit, prepared, slot, batch_index) if self.publisher
                       else self.submit(prepared, slot, batch_index))
                pending.append((indices, job))
            while pending:
                finish()
        finally:
            # 出错后等待已经提交的读取结束，保证缓冲复用和释放的时序。
            while pending:
                _, job = pending.popleft()
                try:
                    self.collect(job)
                except Exception:
                    pass
        # 重建与结果转换在主进程执行，计入完整耗时。
        started = time.perf_counter()
        docs = unpack_docs(self.nlp, output)
        self.metrics["reconstruct_seconds"] = time.perf_counter() - started
        return docs

    def close(self):
        if self.publisher:
            self.publisher.shutdown()
            self.publisher = None
        if self.pool:
            self.pool.shutdown()
            self.pool = None
        if self.copy_stream:
            self.copy_stream.synchronize()
        for slot in self.slots:
            slot.close()
        self.slots.clear()
