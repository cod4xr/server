# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent expectations for the selected synthetic emission profiles only."""


def block(label: str, names: list[int], *, created: int = 2, public: int = 2,
          initialized: int = 3, abort: int = 0) -> str:
    return " ".join(map(str, (label, abort, created, public, len(names), initialized, 0))) + \
        " " + " ".join(f"n:{name}" for name in names)


def expected_if(scenario: int) -> list[str]:
    prior = 2 if scenario in (2, 4) else 4 if scenario == 5 else None
    parent_names = [1, 3] + ([prior] if prior is not None else [2] if scenario == 1 else [])
    child_names = [1, 3] + ([4, 2] if scenario == 5 else
                          [2, 4] if scenario in (3, 4) else [] if scenario == 6 else [2])
    created = 3 if prior is not None else 2
    public = created
    parent = dict(created=created, initialized=7 if prior is not None else 3)
    requests = (
        "94 22:2 23:1", "94 22:2 23:1", "22:2 94", "94 22:2 22:4 23:2",
        "22:2 94 22:4 23:1", "22:4 94 22:2 23:1", "94",
    )[scenario]
    return [block("P", parent_names, **parent), block("B", child_names, public=public),
            block("R", parent_names, **parent),
            block("F", child_names, created=public, public=public,
                  initialized=(1 << len(child_names)) - 1), f"O {requests} p:1"]


def expected_else(scenario: int) -> list[str]:
    parent_names = ([1, 3, 2] if scenario == 0 else [1, 3, 2, 4] if scenario == 2 else
                    [1, 3, 4] if scenario == 3 else [1, 3])
    then_names = [1, 3, 4, 2] if scenario == 3 else [1, 3, 2, 4] if scenario == 2 else [1, 3, 2]
    else_names = [1, 3, 2] if scenario == 0 else [1, 3, 2, 4] if scenario == 2 else \
        [1, 3] if scenario == 4 else [1, 3, 4]
    public = len(parent_names)
    requests = (
        "94 22:2 98 22:2", "94 22:2 23:1 98 22:4 23:1",
        "94 22:2 22:4 98 22:2 22:4", "22:4 94 22:2 23:1 98", "94 22:2 23:1 98",
    )[scenario]
    return [block("P", parent_names, created=3 if scenario == 3 else 2,
                  initialized=7 if scenario == 3 else 3),
            block("B", then_names, public=public), block("D", else_names, public=public),
            block("R", parent_names, created=public, initialized=(1 << public) - 1),
            block("F", then_names, created=public, public=public,
                  initialized=(1 << len(then_names)) - 1),
            block("G", else_names, created=public, public=public,
                  initialized=(1 << len(else_names)) - 1), f"O {requests} p:2"]


def expected_transfer(continuing: int, created: int, target_public: int) -> list[str]:
    removed = created - (2 if continuing else target_public)
    names = [1, 3] + list(range(4, created + 2))
    return [block("R", names, abort=1 if continuing else 2),
            "O " + (f"23:{removed} " if removed else "") + "98 p:1"]


def validate(output: bytes) -> None:
    expected = {f"I {scenario}": expected_if(scenario) for scenario in range(7)}
    expected.update({f"E {scenario}": expected_else(scenario) for scenario in range(5)})
    expected.update({f"T {continuing} {created} {public}": expected_transfer(continuing, created, public)
                     for continuing in range(2) for created in range(2, 6) for public in range(3)})
    actual = {}
    current = None
    for row in output.decode("ascii").splitlines():
        if row.startswith(("I ", "E ", "T ")):
            if row not in expected or row in actual:
                raise ValueError("Unknown/duplicate native emission case")
            current = row
            actual[row] = []
        elif current is None:
            raise ValueError("Native emission row outside case")
        else:
            actual[current].append(row)
    if actual != expected:
        raise ValueError("Native emission trace differs from selected independent scope expectations")
