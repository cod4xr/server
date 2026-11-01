# SPDX-License-Identifier: AGPL-3.0-or-later
"""Selected developer planning joins, distinct from initialized/created runtime cells."""
from local_scope_emission_model import block


def expected(scenario: int, later: int) -> list[str]:
    names = [1, 3]
    if scenario == 7:
        names += [2]
    elif scenario == 8:
        names += [5, 4]
    elif scenario == 9:
        names += [5, 2, 4]
    if later and 2 not in names:
        names += [2]
    rows = [block("R", names)]
    if scenario in (0, 4):
        return rows + [block("B", [1, 3, 2], abort=3 if scenario == 4 else 0)]
    if scenario == 1:
        return rows + [block("B", [1, 3, 2, 4])]
    if scenario == 2:
        # A common branch name joins only into its developer parent. Branch
        # public prefixes grow with that common join; the outer parent stays private.
        return rows + [block("B", [1, 3, 2])] + [block("B", [1, 3, 2], public=3)] * 2
    if scenario == 3:
        return rows + [block("B", [1, 3]), block("B", [1, 3, 2]), block("B", [1, 3, 4])]
    if scenario == 5:
        return rows + [block("B", [1, 3]), block("B", [1, 3, 2])]
    if scenario in (6, 7):
        loop_names = [1, 3] + ([2] if scenario == 7 else [])
        return rows + ["A 4096", block("B", loop_names, public=len(loop_names)),
                       block("B", [1, 3, 2], abort=2 if scenario == 6 else 0)]
    loop_names = [1, 3, 5] + ([2] if scenario == 9 else []) + [4]
    return rows + ["A 4096"] + [block("B", loop_names, public=len(loop_names))] * 2 + [
        block("B", [1, 3, 5, 2], public=3, abort=2 if scenario == 8 else 0)]


def validate(output: bytes) -> None:
    model = {f"D {scenario} {later}": expected(scenario, later)
             for scenario in range(10) for later in range(2)}
    actual = {}
    current = None
    for row in output.decode("ascii").splitlines():
        if row.startswith("D "):
            if row not in model or row in actual:
                raise ValueError("Unknown/duplicate native developer planning case")
            current = row
            actual[row] = []
        elif current is None:
            raise ValueError("Native developer planning row outside a case")
        else:
            actual[current].append(row)
    if actual != model:
        raise ValueError("Native developer planning differs from selected independent joins")
