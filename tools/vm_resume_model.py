# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent queue-service scenarios for actual selected VM_Resume execution."""


def expected() -> dict[str, list[str]]:
    sequences = ((), (1,), (1, 2, 3), (1, 1, 1), (1, 3, 2),
                 (1, 2, 3), (1, 4, 2, 3), (1,), (1,), (1,))
    cases = {}
    for scenario, keys in enumerate(sequences):
        rows = []
        positions = {4: (0, 2, 1), 6: (0, 3, 1, 2)}.get(scenario, tuple(range(len(keys))))
        for index, key in zip(positions, keys):
            values = 3 if scenario == 8 or (scenario == 6 and key == 4) else 1 if scenario == 7 else 0
            scalars = " 40 6 41 6 42 0" if values == 3 else " 40 6" if values else ""
            rows.extend(f"{event} {key}" for event in ("P", "W", "F"))
            rows += [f"S {key} {1000 + index} 1 0 {values}{scalars}", f"E {key}"]
        clock = 2 if scenario in (6, 9) else 1
        count = len(keys)
        rows += ["C 88", f"G {count} {clock} {count} {count} {count} {count} {count} {clock}"]
        cases[f"R {scenario}"] = rows
    return cases


def validate(output: bytes) -> None:
    model = expected()
    actual = {}
    current = None
    for row in output.decode("ascii").splitlines():
        if row.startswith("R "):
            if row not in model or row in actual:
                raise ValueError("Unknown/duplicate native resume case")
            current = row
            actual[row] = []
        elif current is None:
            raise ValueError("Native resume row outside a case")
        else:
            actual[current].append(row)
    if actual != model:
        raise ValueError("Native resume trace differs from independent expectations")
