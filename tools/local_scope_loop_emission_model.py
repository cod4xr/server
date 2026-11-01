# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent selected dynamic/omitted-condition loop state/request expectations."""
from local_scope_emission_model import block

REQUESTS = (
    "22:2 94 99 p:2", "94 22:2 23:1 98 99 p:4", "22:2 94 98 99 p:4",
    "22:2 94 94 98 99 p:4", "22:4 94 94 22:2 23:1 98 98 98 99 p:6",
    "22:5 22:2 22:4 94 99 p:2", "22:5 22:2 22:4 94 98 99 p:4",
    "22:5 22:4 94 22:2 23:1 98 99 p:4", "94 94 22:2 23:1 98 99 99 p:6",
    "22:5 22:4 22:2 98 99 p:3",
)


def expected(scenario: int) -> list[str]:
    names = [1, 3]
    if scenario in (0, 2, 3):
        names += [2]
    elif scenario == 4:
        names += [4]
    elif scenario in (5, 6):
        names += [5, 2, 4]
    elif scenario == 7:
        names += [5, 4]
    elif scenario == 9:
        names += [5, 4, 2]
    parent_flags = 7 if scenario in (5, 6, 7) else 23 if scenario == 9 else 3
    rows = [block("P", names), block("R", names, created=len(names), initialized=parent_flags)]
    if scenario in (0, 2, 3):
        rows += [block("B", names, created=3, public=3, initialized=7)] * (3 if scenario == 3 else 1)
    elif scenario == 1:
        rows += [block("B", [1, 3, 2], initialized=7)]
    elif scenario == 4:
        rows += [block("B", names, created=3, public=3),
                 block("B", [1, 3, 4, 2], created=3, public=3, initialized=11, abort=2),
                 block("B", names, created=3, public=3, initialized=7, abort=1)]
    elif scenario in (5, 6):
        rows += [block("B", names, created=5, public=5, initialized=15, abort=scenario - 5),
                 block("B", names, created=5, public=5, initialized=31)]
    elif scenario in (7, 9):
        rows += [block("B", [1, 3, 5, 4, 2], created=len(names), public=len(names),
                       initialized=23, abort=2),
                 block("B", names, created=len(names), public=len(names), initialized=15)]
    else:
        rows += [block("B", names), block("B", [1, 3, 2], initialized=7)]
    rows += ["O " + REQUESTS[scenario], "Q " + str(0 if scenario == 9 else 2 if scenario in (3, 4, 8) else 1)]
    return rows


def validate(output: bytes) -> None:
    model = {f"L {scenario}": expected(scenario) for scenario in range(10)}
    actual = {}
    current = None
    for row in output.decode("ascii").splitlines():
        if row.startswith("L "):
            if row not in model or row in actual:
                raise ValueError("Unknown/duplicate native loop-emission case")
            current = row
            actual[row] = []
        elif current is None:
            raise ValueError("Native loop-emission row outside a case")
        else:
            actual[current].append(row)
    if actual != model:
        raise ValueError("Native loop emission differs from independent scope/condition expectations")
