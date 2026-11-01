# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guard canonical filename roles and distinguish helper behavior from wrapper limits."""
import argparse
from pathlib import Path

from reference_source import ReferenceSource
import script_array_write_reference as writes

ROLES = {
    "CreateCanonicalFilename": (
        "assert(count > 0);", "do", "c = *filename++;", r"while ( c == '\\' );",
        "if ( c != '/' )", "while ( c >= ' ' )", "*newFilename++ = tolower(c);",
        "if ( !--count )", 'Com_Error(ERR_DROP, "Filename \'%s\' exceeds maximum length", filename);',
        "if ( c == '/' )", "break;", "c = *filename++;", r"if ( c == '\\' )",
        "c = '/';", "if ( !c )", "break;", "*newFilename = 0;"),
    "Scr_CreateCanonicalFilename": (
        "char newFilename[1024];", "CreateCanonicalFilename(newFilename, filename, sizeof(newFilename));",
        "return SL_GetString_(newFilename, 0, 7);"),
}


def inputs(reference: ReferenceSource) -> dict[str, str]:
    return {name: reference.function("src/cscr_stringlist.cpp", name) for name in ROLES}


def verify(profiles: dict[str, str]) -> None:
    for name, needles in ROLES.items():
        body, position = writes.compact(profiles[name]), 0
        for needle in needles:
            needle = writes.compact(needle)
            found = body.find(needle, position)
            if found < 0:
                raise ValueError(f"Selected script filename role changed: {name}")
            position = found + len(needle)


def mutations(profiles: dict[str, str]) -> int:
    count = 0
    for name, needles in ROLES.items():
        body = writes.compact(profiles[name])
        candidates = [body.replace(writes.compact(needle), "mutated_filename_role") for needle in dict.fromkeys(needles)]
        first = writes.compact(needles[0])
        candidates.append(body.replace(first, "", 1) + first)
        for changed_body in candidates:
            changed = dict(profiles)
            changed[name] = changed_body
            try:
                verify(changed)
            except ValueError:
                count += 1
                continue
            raise ValueError("Script filename guard accepted a scoped mutation")
    return count


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    args = parser.parse_args()
    profiles = inputs(ReferenceSource(args.reference))
    verify(profiles)
    print(f"Verified selected filename roles; rejected {mutations(profiles)} mutations")


if __name__ == "__main__":
    main()
