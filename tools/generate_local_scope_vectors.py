# SPDX-License-Identifier: AGPL-3.0-or-later
"""Record actual selected x86 local-name planning; not native parser/VM cleanup."""
import argparse
from pathlib import Path

from local_scope_reference import run
from reference_source import ReferenceSource

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/script/tests/fixtures/local_scope_vectors.txt"


def validate(output: bytes) -> None:
    """Reject incomplete cases or malformed blocks before publishing any fixture."""
    cases = {}
    current = None
    for line in output.decode("ascii").splitlines():
        parts = line.split()
        if not parts:
            raise ValueError("Empty native scope trace row")
        if parts[0] == "C":
            if len(parts) != 3:
                raise ValueError("Malformed native scope case")
            current = (int(parts[1]), int(parts[2]))
            if current in cases:
                raise ValueError("Duplicate native scope case")
            cases[current] = []
        elif current is None:
            raise ValueError("Native scope row outside a case")
        elif parts[0] == "A":
            if parts != ["A", "4096"]:
                raise ValueError("Unexpected native scope auxiliary allocation")
        elif parts[0] in ("R", "B"):
            if parts[0] == "R" and cases[current]:
                raise ValueError("Repeated native parent scope")
            header = list(map(int, parts[1:7]))
            names = [int(name.removeprefix("n:")) for name in parts[7:] if name.startswith("n:")]
            if len(header) != 6 or not 2 <= header[3] <= 64 or len(names) != header[3]:
                raise ValueError("Malformed native scope block")
            if len(parts) != 7 + len(names) or len(set(names)) != len(names):
                raise ValueError("Duplicate/unrecognized native scope names")
            if names[:2] != [1, 3] or header[1] != 2 or header[4:] != [3, 0]:
                raise ValueError("Native inherited scope/initialization changed")
            cases[current].append((parts[0], header, names))
        else:
            raise ValueError("Unknown native scope trace row")
    expected = {(case, later) for case in (*range(8), *range(10, 19)) for later in (0, 1)}
    if set(cases) != expected or any(not rows or rows[0][0] != "R" for rows in cases.values()):
        raise ValueError("Incomplete native scope inventory")
    # Independent selected consequences, including deliberately different exit paths.
    for case, names in {0: [1, 3], 1: [1, 3, 2], 2: [1, 3], 3: [1, 3, 4],
                        4: [1, 3, 2], 10: [1, 3, 2], 11: [1, 3],
                        12: [1, 3, 2], 14: [1, 3, 4], 15: [1, 3, 5, 2, 4],
                        16: [1, 3, 5, 2, 4], 17: [1, 3, 5, 4], 18: [1, 3]}.items():
        if cases[case, 0][0][2] != names:
            raise ValueError("Selected native branch/loop name-planning consequence changed")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--assembler", default="nasm")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    output = run(reference, args.cc, args.assembler)
    validate(output)
    expected = (
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n"
        f"# Upstream {reference.revision}; hashes in upstream.lock.json\n"
        "# Actual selected x86-32 CalcLocalVars helpers; O0/O2 C drivers agree.\n"
        "# Synthetic AST/name IDs, bounded modeled allocator/globals; no parser/VM/freeing claim.\n"
        "# C case later_direct_b; R parent, B allocated block, A auxiliary allocation bytes.\n"
        "# Block: abort created public total init_bits_lo init_bits_hi then n:name IDs.\n"
        "# Names: 1 inherited_a, 3 condition, 2 b, 4 c, 5 for_initializer.\n"
        "# Cases 0 one-sided if; 1 common if/else; 2 disjoint; 3 returning then; 4 returning else;\n"
        "# 5 ordinary block; 6 nested common branches; 7 different declaration orders;\n"
        "# 10 while fallthrough; 11 break; 12 continue; 13 common branches; 14 mixed exits;\n"
        "# 15 for fallthrough; 16 for continue; 17 for break; 18 nested while break.\n"
        "# Later declarations are separate analysis calls; these are planning snapshots, not emitted cleanup.\n"
    ).encode("ascii") + output
    if args.check:
        if OUTPUT.read_bytes() != expected:
            raise SystemExit("Native local-scope fixtures differ")
    else:
        OUTPUT.write_bytes(expected)
    print("Verified 34 native local-name planning cases at driver O0/O2")


if __name__ == "__main__":
    main()
