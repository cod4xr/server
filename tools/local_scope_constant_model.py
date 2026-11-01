# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent selected literal-true emission/literal-false analysis expectations."""
from local_scope_emission_model import block


def expected(scenario: int) -> list[str]:
    names = ([1, 3, 2] if scenario == 0 else [1, 3] if scenario in (1, 6) else
             [1, 3, 5, 4] if scenario in (3, 7) else [1, 3, 4, 2] if scenario == 4 else [1, 3, 5, 4, 2])
    analysis_only = scenario in (1, 3)
    flags = 3 if analysis_only or scenario == 6 else 7 if scenario in (0, 7) else 11 if scenario == 4 else 23
    rows = [block("P", names), block("R", names, created=2 if analysis_only else len(names), initialized=flags)]
    if scenario == 0:
        rows += [block("B", names, created=3, public=3, initialized=7)]
    elif scenario == 1:
        rows += [block("B", [1, 3, 2], abort=2)]
    elif scenario == 2:
        rows += [block("B", names, created=5, public=5, initialized=23, abort=2),
                 block("B", names, created=5, public=5, initialized=15)]
    elif scenario == 3:
        rows += [block("B", [1, 3, 5, 4, 2], public=4, abort=2), block("B", names, public=4)]
    elif scenario == 4:
        rows += [block("B", names, created=4, public=4),
                 block("B", names, created=4, public=4, initialized=11, abort=2),
                 block("B", names, created=4, public=4, initialized=7, abort=1)]
    elif scenario == 5:
        rows += [block("B", names, created=5, public=5, initialized=7),
                 block("B", names, created=5, public=5, initialized=15),
                 block("B", names, created=5, public=5, initialized=23, abort=2),
                 block("B", names, created=5, public=5, initialized=15, abort=1)]
    elif scenario == 6:
        rows += [block("B", names), block("B", [1, 3, 2], initialized=7, abort=2),
                 block("B", [1, 3, 4], initialized=7, abort=2)]
    else:
        rows += [block("B", names, created=4, public=4, initialized=7),
                 block("B", names, created=4, public=4, initialized=15),
                 block("B", [1, 3, 5, 4, 2], created=4, public=4, initialized=23, abort=2),
                 block("B", names, created=4, public=4, initialized=15, abort=2)]
    requests = ("22:2 98 99 p:3", "p:0", "22:5 22:4 22:2 98 99 p:3", "p:0",
                "22:4 22:2 94 98 98 98 99 p:5", "22:5 22:4 22:2 94 98 98 98 99 p:5",
                "94 22:2 23:1 98 98 22:4 23:1 98 99 p:5", "22:5 22:4 94 22:2 23:1 98 98 98 99 p:5")
    return rows + ["O " + requests[scenario], "Q " + str(0 if analysis_only else 2 if scenario >= 4 else 1), "D 1"]


def validate(output: bytes) -> None:
    model = {f"K {scenario}": expected(scenario) for scenario in range(8)}
    actual = {}
    current = None
    for row in output.decode("ascii").splitlines():
        if row.startswith("K "):
            if row not in model or row in actual:
                raise ValueError("Unknown/duplicate native constant-scope case")
            current = row
            actual[row] = []
        elif current is None:
            raise ValueError("Native constant-scope row outside a case")
        else:
            actual[current].append(row)
    if actual != model:
        raise ValueError("Native constant scope differs from independent selected expectations")
