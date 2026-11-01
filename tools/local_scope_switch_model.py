# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent selected integer-switch scope state and opcode-request expectations."""
from local_scope_emission_model import block

REQUESTS = (
    "124 22:2 23:1 98 125 p:2", "124 22:2 23:1 98 22:4 23:1 98 125 p:3",
    "124 22:2 98 22:2 98 125 p:3", "124 22:2 23:1 22:4 98 125 p:2",
    "124 22:2 98 22:2 98 125 p:3", "22:2 124 98 22:4 23:1 98 125 p:3",
    "124 94 22:2 98 22:2 98 22:2 98 125 p:5",
    "124 22:2 22:4 23:1 98 22:2 98 125 p:3", "22:2 22:4 94 124 98 98 125 99 p:5",
)


def expected(scenario: int) -> list[str]:
    names = [1, 3] + ([4] if scenario == 3 else [2, 4] if scenario == 8 else
                      [2] if scenario in (2, 4, 5, 6, 7) else [])
    rows = [block("P", names, created=3 if scenario == 5 else 2,
                  initialized=7 if scenario == 5 else 3),
            block("R", names, created=len(names), initialized=3 if scenario == 8 else
                  7 if len(names) == 3 else 3)]
    if scenario == 0:
        rows += [block("B", [1, 3, 2], initialized=7, abort=2)]
    elif scenario == 1:
        rows += [block("B", [1, 3, name], initialized=7, abort=2) for name in (2, 4)]
    elif scenario == 3:
        rows += [block("B", [1, 3, 2], initialized=7),
                 block("B", names, created=3, public=3, initialized=7, abort=2)]
    elif scenario == 5:
        rows += [block("B", names, created=3, public=3, initialized=7, abort=2),
                 block("B", [1, 3, 2, 4], created=3, public=3, initialized=15, abort=2)]
    elif scenario == 7:
        rows += [block("B", [1, 3, 2, 4], created=3, public=3, initialized=15, abort=2),
                 block("B", names, created=3, public=3, initialized=7, abort=2)]
    elif scenario == 8:
        rows += [block("B", names, created=4, public=4, initialized=11),
                 block("B", names, created=4, public=4, initialized=7, abort=1),
                 block("B", names, created=4, public=4, initialized=11, abort=2)]
    else:
        joined = block("B", names, created=3, public=3, initialized=7, abort=2)
        rows += [joined]
        if scenario == 4:
            rows += [block("B", [1, 3])]
        if scenario == 6:
            rows += [block("B", names, created=3, public=3, initialized=7)] * 2
        rows += [joined]
    return rows + ["O " + REQUESTS[scenario], "Q " + str(2 if scenario in (6, 8) else 1)]


def validate(output: bytes) -> None:
    model = {f"S {scenario}": expected(scenario) for scenario in range(9)}
    actual = {}
    current = None
    for row in output.decode("ascii").splitlines():
        if row.startswith("S "):
            if row not in model or row in actual:
                raise ValueError("Unknown/duplicate native switch-scope case")
            current = row
            actual[row] = []
        elif current is None:
            raise ValueError("Native switch-scope row outside a case")
        else:
            actual[current].append(row)
    if actual != model:
        raise ValueError("Native switch scope differs from independent selected expectations")
