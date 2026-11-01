# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent expectations for selected developer-list modes and private cleanup."""


def block(label: str, names: list[int], *, created: int = 0, public: int = 0,
          initialized: int = 0, abort: int = 0) -> str:
    fields = (abort, created, public, len(names), initialized, 0)
    return label + " " + " ".join(map(str, fields)) + "".join(f" n:{name}" for name in names)


def expected(scenario: int, enabled: int, nesting: int) -> list[str]:
    created = 1 if scenario == 3 else 2
    parent = block("P", [1, 3], created=created, public=2, initialized=1 if scenario == 3 else 3)
    if nesting:
        return [parent, block("B", [1, 3, 2]), f"G {nesting} 123 1 0 0 0 0", "M", "O", "W 170"]
    if scenario == 0:
        return [parent, block("B", [1, 3], created=2, public=2, initialized=3),
                f"G 0 123 0 {1 - enabled} 0 0 0", "M", "O", "W 170"]
    newly_created = scenario in (1, 3)
    child_created = created + int(newly_created and not enabled)
    initialized = 7 if scenario == 1 else 3
    child = block("B", [1, 3, 2], created=child_created, public=created,
                  initialized=initialized, abort=3 if scenario == 4 else 0)
    adds = int(bool(enabled and newly_created))
    drops = int(scenario != 4)
    globals_row = f"G 0 123 0 {1 - enabled} {adds} {drops} {adds}"
    modes = f"M {1 if enabled else 2}"
    requests = "O 22 23" if newly_created and enabled else "O 22" if newly_created else "O"
    output = f"W 170 22 {3 if scenario == 3 else 2} 0 23 1" if newly_created and enabled else "W 170"
    return [parent, child, globals_row, modes, requests, output]


def validate(output: bytes) -> None:
    model = {f"C {scenario} {enabled} 0": expected(scenario, enabled, 0)
             for scenario in range(5) for enabled in range(2)}
    model.update({f"C 1 1 {nesting}": expected(1, 1, nesting) for nesting in (1, 2)})
    for external in (0, 1):
        for borrowed in (0, 1):
            drops = "T" if borrowed else "T 2 3" if external else "T 2"
            model[f"F {external} {borrowed}"] = [f"G 2 {100 - external} 0 0 0", drops, "W 170"]
    actual = {}
    current = None
    for row in output.decode("ascii").splitlines():
        if row.startswith(("C ", "F ")):
            if row not in model or row in actual:
                raise ValueError("Unknown/duplicate native developer case")
            current = row
            actual[row] = []
        elif current is None:
            raise ValueError("Native developer row outside a case")
        else:
            actual[current].append(row)
    if actual != model:
        raise ValueError("Native developer trace differs from independent expectations")
