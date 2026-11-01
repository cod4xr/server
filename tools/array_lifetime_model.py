# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent integer-owner release model for the selected native lifetime corpus."""
from collections import deque


def release(depth: int, cycle_at: int, aliases: int) -> tuple[int, int, int]:
    graph = [[] for _ in range(depth)]
    for index in range(depth - 1):
        graph[index].append(index + 1)
    if cycle_at >= 0:
        graph[cycle_at].append(cycle_at)
    owners = [0] * depth
    owners[0] = 1
    for edges in graph:
        for target in edges:
            owners[target] += 1
    owners[0] -= 1
    pending = deque([0] if owners[0] == 0 else [])
    freed = set()
    fields = 0
    while pending:
        current = pending.popleft()
        if current in freed:
            raise ValueError("Independent release model double-free")
        freed.add(current)
        fields += aliases + len(graph[current])
        for target in graph[current]:
            owners[target] -= 1
            if owners[target] == 0:
                pending.append(target)
    return len(freed), fields, (depth - len(freed)) * aliases


def verify(rows: bytes) -> None:
    covered = set()
    for row in rows.decode("ascii").splitlines():
        left, right = row.split("|", 1)
        columns = left.split()
        if columns[0] != "Q":
            raise ValueError("Unknown native lifetime row")
        depth, cycle_at, aliases, width, freed, fields, retained, detached, excess = map(int, columns[1:10])
        label = depth, cycle_at, aliases, width
        if label in covered:
            raise ValueError("Duplicate native lifetime case")
        covered.add(label)
        if (freed, fields, retained) != release(depth, cycle_at, aliases):
            raise ValueError("Native freeing/alias state differs from independent owner-release model")
        if detached != int(retained > 0) or excess != max(0, retained - 1):
            raise ValueError("Native COW decision/excess differs from independent model")
        old = [f"i:{index}" for index in reversed(range(width))]
        selected = list(reversed(old)) if detached else old
        if columns[10:] != old or right.split() != selected:
            raise ValueError("Native lifetime key order differs from independent model")
    expected = {(depth, cycle_at, aliases, width)
                for depth in range(1, 5) for cycle_at in range(-1, depth)
                for aliases in range(1, 4) for width in range(5)}
    if covered != expected:
        raise ValueError("Native lifetime corpus does not cover exactly the intended cases")
