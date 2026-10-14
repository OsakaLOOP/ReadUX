# cython: language_level=3, boundscheck=False, wraparound=False, cdivision=True
"""GiNZA greedy 推理解码：复用 spaCy 状态，融合合法动作评分与选择。"""
from libcpp.vector cimport vector
import time
from spacy.pipeline._parser_internals.stateclass cimport StateClass
from spacy.pipeline._parser_internals._state cimport StateC
from spacy.pipeline._parser_internals.transition_system cimport TransitionSystem, Transition


def decode(component, docs, const float[:, :, :, ::1] cached,
           const float[:, ::1] bias, int token_offset=0):
    cdef TransitionSystem moves = component.moves
    batch = moves.init_batch(docs)
    cdef StateClass state
    cdef vector[StateC*] states
    for state in batch:
        if not state.is_final():
            states.push_back(state.c)
    cdef int features = cached.shape[1]
    cdef int actions = cached.shape[2]
    cdef int pieces = cached.shape[3]
    cdef vector[int] ids
    ids.resize(features)
    cdef int i, j, f, p, best, row, width, k
    cdef float value, score, maximum, best_score
    cdef vector[float] scores
    cdef vector[const float*] rows
    width = actions * pieces
    scores.resize(width)
    rows.resize(features)
    cdef const float* bias_data = &bias[0, 0]
    cdef float* score_data = &scores[0]
    cdef Transition action
    started = time.perf_counter()
    # 每个单元的动作顺序独立；一次分配上下文数组，整个批次复用。
    with nogil:
        for i in range(states.size()):
            while not states[i].is_final():
                states[i].set_context_tokens(&ids[0], features)
                for f in range(features):
                    row = 0 if ids[f] < 0 else ids[f] + token_offset + 1
                    rows[f] = &cached[row, f, 0, 0]
                # 连续数组相加保持特征次序，为编译器提供向量化循环。
                for k in range(width):
                    score_data[k] = 0
                for f in range(features):
                    for k in range(width):
                        score_data[k] = score_data[k] + rows[f][k]
                for k in range(width):
                    score_data[k] = score_data[k] + bias_data[k]
                best = -1
                best_score = 0
                for j in range(actions):
                    action = moves.c[j]
                    if not action.is_valid(states[i], action.label):
                        continue
                    maximum = 0
                    for p in range(pieces):
                        value = score_data[j * pieces + p]
                        if p == 0 or value > maximum:
                            maximum = value
                    score = maximum
                    if best == -1 or score > best_score:
                        best = j
                        best_score = score
                action = moves.c[best]
                action.do(states[i], action.label)
    kernel_seconds = time.perf_counter() - started
    component.set_annotations(docs, batch)
    return kernel_seconds
