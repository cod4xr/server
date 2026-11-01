# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent phase order expectations, including disabled scripts and time wrap."""


def expected() -> dict[str, list[str]]:
    cases = {}
    for mode in range(5):
        for boundary in range(2):
            for operation in range(3):
                time = 0xffffff if boundary else 7
                rows = []
                elapsed = calls = resumes = logs = frees = 0
                bucket = mode != 2
                for _ in range(2 if operation == 2 else 1):
                    if mode:
                        rows.append(f"B {time}")
                        if mode != 1:
                            rows.append(f"L {time}")
                            if bucket:
                                if mode == 4:
                                    logs += 1
                                    rows.append(f"O {time}")
                                resumes += 1
                                rows += [f"R {time}", f"D {time}"]
                                bucket = False
                        rows.append(f"B {time}")
                        elapsed += 1
                        calls += 2
                if operation:
                    rows.append(f"F {time}")
                    time = (time + 1) & 0xffffff
                    frees = 1
                rows.append(f"G {time} {100 + elapsed} {calls} {resumes} {logs} {frees} {int(bucket)}")
                cases[f"T {mode} {boundary} {operation}"] = rows
    return cases


def validate(output: bytes) -> None:
    model = expected()
    actual = {}
    current = None
    for row in output.decode("ascii").splitlines():
        if row.startswith("T "):
            if row not in model or row in actual:
                raise ValueError("Unknown/duplicate native scheduler phase case")
            current = row
            actual[row] = []
        elif current is None:
            raise ValueError("Native scheduler phase row outside a case")
        else:
            actual[current].append(row)
    if actual != model:
        raise ValueError("Native scheduler phase trace differs from independent expectations")
