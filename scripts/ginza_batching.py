"""有界文本批次、按 token 成本分组，以及跨请求轮转调度。"""
from collections import deque
import math


def length_batches(indexed, batch_size, token_budget, sort_lengths=True):
    if batch_size < 1 or token_budget < 1:
        raise ValueError("批次大小和 token 预算必须大于零")
    items = sorted(indexed, key=lambda item: len(item[1])) if sort_lengths else indexed
    batch, tokens = [], 0
    for item in items:
        length = len(item[1])
        if length > token_budget:
            raise ValueError(f"单元含 {length} token，超过缓冲容量 {token_budget}")
        if batch and (len(batch) >= batch_size or tokens + length > token_budget):
            yield batch
            batch, tokens = [], 0
        batch.append(item)
        tokens += length
    if batch:
        yield batch


def request_batches(queues, batch_size, token_budget, bucket_width=64):
    """每批先服务轮转队首，再从其他请求选择同桶单元；身份随批次传递。"""
    if min(batch_size, token_budget, bucket_width) < 1:
        raise ValueError("批次大小、token 预算和分桶宽度必须大于零")
    queues = [deque(queue) for queue in queues]
    for queue in queues:
        for _, doc in queue:
            if len(doc) > token_budget:
                raise ValueError(f"单元含 {len(doc)} token，超过缓冲容量 {token_budget}")
    active = deque(i for i, queue in enumerate(queues) if queue)
    while active:
        anchor = active.popleft()
        item = queues[anchor].popleft()
        batch, tokens = [item], len(item[1])
        bucket = len(item[1]) // bucket_width
        if queues[anchor]:
            active.append(anchor)
        skipped = 0
        while active and len(batch) < batch_size and skipped < len(active):
            request = active.popleft()
            candidate = queues[request][0]
            length = len(candidate[1])
            if length // bucket_width == bucket and tokens + length <= token_budget:
                batch.append(queues[request].popleft())
                tokens += length
                skipped = 0
            else:
                skipped += 1
            if queues[request]:
                active.append(request)
        yield batch


def decode_groups(lengths, workers):
    """连续 token 范围形成约两倍 worker 数的任务，长任务优先提交。"""
    target = max(1, math.ceil(sum(lengths) / max(1, workers * 2)))
    groups, start, tokens = [], 0, 0
    for index, length in enumerate(lengths):
        if index > start and tokens + length > target:
            groups.append((start, index, tokens))
            start, tokens = index, 0
        tokens += length
    if start < len(lengths):
        groups.append((start, len(lengths), tokens))
    return sorted(groups, key=lambda group: -group[2])
