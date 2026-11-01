#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Recover structural parser data/replays without executing original semantic actions."""

import argparse
from pathlib import Path
import subprocess

from reference_source import ReferenceSource
import script_argument_reference as arguments
import script_array_reference as arrays
import script_array_write_reference as array_writes
import script_array_alias_reference as array_aliases
import script_call_link_reference as call_links
import script_function_reference as function_references
import script_predicate_reference as predicates
import script_switch_reference as switches
import script_string_access_reference as string_access
import script_defined_reference as definedness
import script_field_reference as fields
import script_field_update_reference as field_updates
import script_object_comparison_reference as object_comparisons
import script_expression_reference as expressions
import script_for_reference as for_loops
import script_lexer_reference as lexical
import script_statement_reference as statements
import script_syntax_reference as syntax
import script_update_reference as updates
import script_loop_transfer_reference as transfers
import script_while_reference as loops
import script_vector_constructor_reference as vectors
import script_vector_index_reference as vector_indices


ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / "crates/script/src/syntax"
FIXTURES = ROOT / "crates/script/tests/fixtures/syntax_vectors.txt"


def cases() -> list[tuple[str, str]]:
    scripts = ["", "main() {}", "main(a,b) { return a+b; }", "main() { x=1; wait 0.1; }",
               "main() { for(i=0;i<10;i++) { if(i==3) break; } }",
               "main() { self thread worker(); self endon(\"done\"); self waittill(\"ready\"); }",
               "#include maps\\mp\\common;\nmain() {}", "#using_animtree(\"mp\");\nmain() {}",
               "main() { a=[]; a[0]=\"value\"; b=(1,2,3); return undefined; }",
               "main() { switch(x) { case 1: return; default: break; } }",
               "main() { do { x--; } while(x>0); }", "main() { foreach(x in y) {} }",
               "main( { }", "main() { return 1 }", "main() { x=1.0e3; }", "main() {} trailing",
               "/* unfinished", "main() { x=; }", "main() { x=1; }}",
               "main( { } /* unfinished", "main() {} /* unfinished"]
    expressions = ["1+2*3", "(1,2,3)", "self.origin", "level.array[0]", "f(1,2)",
                   "true && !false", "undefined", "\"private-script\"", "a?b:c", "[]",
                   "(1+2", "1+", "main() {}", "", "1.0e3", "1;2", "{}", "return 1;",
                   "if(x) return 1;", "x=1;", "wait 1;"]
    result = [("script", text) for text in scripts] + [("fragment", text) for text in expressions]
    for depth in (1, 2, 8, 32, 128):
        expression = "(" * depth + "1" + ")" * depth
        result += [("fragment", expression), ("script", f"main() {{ return {expression}; }}")]
    for count in (1, 2, 8, 64):
        result.append(("script", "\n".join(f"f{index}() {{ return {index}; }}" for index in range(count))))
    # Every prefix of a representative valid module gives deterministic EOF and
    # syntax failures independently of a Rust parser's tree construction.
    module = "main(a) { if(a) return 1; else return 0; }"
    result += [("script", module[:length]) for length in range(len(module) + 1)]
    return result


def render(tables: dict[str, list[int]], names: list[str], revision: str) -> str:
    lines = ["// SPDX-License-Identifier: AGPL-3.0-or-later",
             "// Generated structural grammar data; no semantic actions or native pointers.",
             f"// Source revision: {revision}."]
    if "PACT" in names:
        lines += ["pub(super) const LAST: usize = 1347;", "pub(super) const FINAL: usize = 261;",
                  "pub(super) const UNSET: i16 = -32768;"]
    for name in names:
        values = tables[name]
        lines.append(f"pub(super) const {name}: [i16; {len(values)}] = [")
        for index in range(0, len(values), 16):
            lines.append("    " + ", ".join(map(str, values[index:index + 16])) + ",")
        lines.append("];\n")
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = ReferenceSource(args.reference)
    source = reference.files["src/asmsource/_scr_yacc.asm"]
    loader = reference.files["tools/cod4_dasm/src/_scr_main.asm"]
    if 'db "+",0' not in loader or 'mov dword [edx+0x20018], _cstring_' not in loader:
        raise ValueError("Script-loader selector differs from pinned source")
    tables = syntax.recover(source)
    for_loops.verify(source, tables, reference.files["src/asmsource/_scr_compiler.asm"])
    vectors.verify(source, reference.files["src/asmsource/_scr_parsetree.asm"],
                   reference.files["src/asmsource/_scr_compiler.asm"],
                   reference.files["src/asmsource/_scr_vm.asm"], reference.files["src/scr_vm.h"],
                   reference.function("src/cscr_variable.cpp", "Scr_CastVector"))
    vector_indices.verify(source, reference.files["src/asmsource/_scr_parsetree.asm"],
                          reference.files["src/asmsource/_scr_compiler.asm"],
                          reference.files["src/asmsource/_scr_vm.asm"], reference.files["src/scr_vm.h"],
                          reference.function("src/cscr_variable.cpp", "Scr_EvalArray"),
                          reference.function("src/cscr_variable.cpp", "Scr_EvalArrayRef"))
    syntax.verify_declarations(source, tables)
    expressions.verify(source, tables, reference.files["src/scr_vm.h"])
    statements.verify(source, tables, reference.files["src/scr_vm.h"])
    updates.verify(reference.files["src/asmsource/_scr_compiler.asm"],
                   reference.files["src/asmsource/_scr_vm.asm"], reference.files["src/scr_vm.h"])
    loops.verify(reference.files["src/asmsource/_scr_compiler.asm"],
                 reference.files["src/asmsource/_scr_vm.asm"], reference.files["src/scr_vm.h"])
    transfers.verify(reference.files["src/asmsource/_scr_compiler.asm"],
                     reference.files["src/asmsource/_scr_vm.asm"], reference.files["src/scr_vm.h"])
    arguments.verify(reference.files["src/asmsource/_scr_compiler.asm"],
                     reference.files["src/asmsource/_scr_vm.asm"], reference.files["src/scr_vm.h"],
                     reference.files["src/cscr_variable.cpp"],
                     reference.function("src/cscr_variable.cpp", "GetNewVariableIndexInternal3"))
    call_links.verify(reference.files["src/asmsource/_scr_compiler.asm"], reference.files["src/scr_vm.h"])
    function_references.verify(reference.files["src/asmsource/_scr_compiler.asm"],
                               reference.files["src/asmsource/_scr_vm.asm"], reference.files["src/scr_vm.h"])
    predicates.verify(*predicates.inputs(reference))
    arrays.verify(*arrays.inputs(reference))
    array_writes.verify(*array_writes.inputs(reference))
    array_aliases.verify(array_aliases.inputs(reference))
    string_access.verify(*string_access.inputs(reference))
    definedness.verify(*definedness.inputs(reference))
    fields.verify(*fields.inputs(reference))
    field_updates.verify(reference.files["src/asmsource/_scr_compiler.asm"])
    object_comparisons.verify(object_comparisons.inputs(reference))
    switches.verify(*switches.inputs(reference))
    syntax.verify_symbols(reference.files["src/asmsource/_scr_compiler.asm"],
                          reference.files["src/cscr_variable.h"])
    lex = lexical.recover(source)
    outputs = {}
    for name, keys in (("legacy_actions.rs", ["TRANSLATE", "PACT", "DEFAULT_REDUCE", "ACTION", "CHECK"]),
                       ("legacy_productions.rs", ["LHS", "LENGTH", "GOTO_OFFSET", "DEFAULT_GOTO"])):
        formatted = subprocess.run(["rustfmt", "--edition", "2024", "--emit", "stdout"],
                                   input=render(tables, keys, reference.revision), text=True,
                                   capture_output=True, check=True).stdout
        outputs[DIRECTORY / name] = formatted
    rows = ["# Structural table replay only; NOT original semantic actions/parser execution.",
            "# mode source_hex status reduction_sequence"]
    for mode, text in cases():
        data = text.encode("ascii")
        tokens, status = syntax.source_tokens(lex, data)
        status, reductions = syntax.parse(tables, tokens, mode, status)
        rows.append(f"{mode} {data.hex() or '~'} {status} {','.join(map(str, reductions)) or '~'}")
    outputs[FIXTURES] = "\n".join(rows) + "\n"
    for path, expected in outputs.items():
        if args.check:
            if path.read_text(encoding="utf-8") != expected:
                raise SystemExit(f"Structural parser data/replays differ: {path.name}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(expected, encoding="utf-8", newline="\n")
    print("Verified grammar/declaration/symbol/expression/statement/update/argument/array/local-call/function-reference/predicate/switch/while/transfer/for/vector/index "
          f"profiles and {len(rows) - 2} structural replays")


if __name__ == "__main__":
    main()
