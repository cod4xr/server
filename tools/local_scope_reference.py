# SPDX-License-Identifier: AGPL-3.0-or-later
"""Execute selected pinned x86 compiler helpers with explicit modeled scope services."""
import argparse
from pathlib import Path
import re
import subprocess
import sys
import tempfile

from reference_source import ReferenceSource


ROOT = Path(__file__).resolve().parents[1]
FUNCTIONS = (
    "Scr_CalcLocalVarsVariableExpressionRef", "Scr_CalcLocalVarsStatement",
    "EvalPrimitiveExpression", "EvalBinaryOperatorExpression",
)
EXTERNALS = {
    "CompileError", "Com_Error", "Scr_AllocVector", "var_typename", "AddRefToValue",
    "Scr_EvalBinaryOperator", "gScrVarPub", "memcpy", "Hunk_AllocateTempMemoryHigh",
    "RemoveRefToValue", "Scr_CastBool", "scrCompileGlob", "gScrCompilePub",
}
DRIVERS = ("local_scope_platform.c", "local_scope_services.c", "local_scope_ast.c", "local_scope_cases.c",
           "local_scope_loops.c", "local_scope_loop_cases.c")


def function(source: str, name: str) -> str:
    """Keep the actual complete function and all its local assembly labels."""
    start = source.index(name + ":\n")
    following = re.search(r"(?m)^(?:;[^\n]+\(|SECTION )", source[start:])
    if following is None:
        raise ValueError("Native scope function boundary changed")
    return source[start:start + following.start()]


def assembly(reference: ReferenceSource, functions: tuple[str, ...] = FUNCTIONS,
             externals: set[str] = EXTERNALS, *,
             source_name: str = "src/asmsource/_scr_compiler.asm",
             modeled_globals: frozenset[str] = frozenset()) -> str:
    source = reference.files[source_name]
    body = "\n".join(function(source, name) for name in functions)
    externs = re.findall(r"(?m)^\s*extern (\w+)\s*$", source)
    used = {name for name in externs if re.search(r"\b" + re.escape(name) + r"\b", body)}
    labels = set(re.findall(r"(?m)^(\w+):", body))
    used.update(set(re.findall(r"\bcall (\w+)", body)) - labels)
    for name in modeled_globals:
        if not re.search(rf"(?m)^{re.escape(name)}: resb 0x[0-9a-f]+$", source) or not re.search(
                r"\b" + re.escape(name) + r"\b", body):
            raise ValueError("Modeled native zero-initialized global boundary changed")
    used.update(modeled_globals)
    if used != externals:
        raise ValueError("Selected native scope service boundary changed")
    # Tables must retain their actual labels/entries; do not reimplement dispatch.
    data = source[source.index("SECTION .data"):]
    boundaries = list(re.finditer(r"(?m)^(?:\w+:|SECTION )", data))
    pieces = []
    for index, match in enumerate(boundaries):
        label = re.match(r"(\w+):", match.group())
        if label and label[1] not in modeled_globals and re.search(r"\b" + re.escape(label[1]) + r"\b", body):
            end = boundaries[index + 1].start() if index + 1 < len(boundaries) else len(data)
            pieces.append(data[match.start():end])
    return ("BITS 32\nSECTION .text\n"
            + "\n".join("extern " + name for name in sorted(used)) + "\n"
            + "\n".join("global " + name for name in functions) + "\n"
            + body + "\nSECTION .rdata\n" + "\n".join(pieces))


def checked(command: list[str], directory: Path) -> None:
    result = subprocess.run(command, cwd=directory, capture_output=True, timeout=60)
    if result.returncode:
        raise ValueError(result.stderr.decode("utf-8", errors="replace")[:4000])


def run(reference: ReferenceSource, cc: str = "gcc", assembler: str = "nasm", *,
        functions: tuple[str, ...] = FUNCTIONS, externals: set[str] = EXTERNALS,
        drivers: tuple[str, ...] = DRIVERS, extra_sources: tuple[str, ...] = (),
        source_name: str = "src/asmsource/_scr_compiler.asm",
        modeled_globals: frozenset[str] = frozenset()) -> bytes:
    """Build temporary freestanding 32-bit tools; never create Cargo targets/caches."""
    if sys.platform not in ("win32", "linux"):
        raise ValueError("Native scope harness currently supports Windows/Linux x86 hosts")
    with tempfile.TemporaryDirectory(prefix="cod4x-local-scope-") as scratch:
        directory = Path(scratch).resolve()
        if directory.parent != Path(tempfile.gettempdir()).resolve() or not directory.name.startswith("cod4x-local-scope-"):
            raise ValueError("Native scope scratch escaped its temporary directory")
        (directory / "native.asm").write_text(
            assembly(reference, functions, externals, source_name=source_name,
                     modeled_globals=modeled_globals), encoding="utf-8")
        driver = "\n\n".join((ROOT / "tools/oracles" / name).read_text(encoding="utf-8") for name in drivers)
        (directory / "driver.c").write_text(driver, encoding="utf-8")
        additional = []
        for index, source in enumerate(extra_sources):
            name = f"selected_{index}.c"
            (directory / name).write_text(source, encoding="utf-8")
            additional.append(name)
        windows = sys.platform == "win32"
        format_name = "win32" if windows else "elf32"
        prefix = ["--prefix", "_"] if windows else []
        checked([assembler, "-f", format_name, *prefix, "native.asm", "-o", "native.o"], directory)
        libraries = []
        entry = "scope_start"
        linker = ["-static", "-no-pie"]
        if windows:
            (directory / "kernel.def").write_text(
                "LIBRARY KERNEL32.dll\nEXPORTS\nGetStdHandle@4\nWriteFile@20\nExitProcess@4\n",
                encoding="ascii")
            checked(["dlltool", "-m", "i386", "--as-flags=--32", "-k", "-d", "kernel.def", "-l", "kernel.a"], directory)
            libraries = ["kernel.a"]
            entry = "_scope_start"
            linker = []
        outputs = []
        executable = directory / ("scope.exe" if windows else "scope")
        for optimization in ("-O0", "-O2"):
            checked([cc, "-m32", "-std=c11", optimization, "-ffreestanding", "-fno-builtin",
                     "-fno-stack-protector", "-fno-pie", "-nostdlib", "-Wall", "-Wextra", "-Werror",
                     *linker, "-Wl,--entry," + entry, "driver.c", *additional, "native.o", *libraries,
                     "-o", str(executable)], directory)
            flags = subprocess.CREATE_NO_WINDOW if windows else 0
            result = subprocess.run([str(executable)], capture_output=True, timeout=30, creationflags=flags)
            if result.returncode:
                raise ValueError(f"Native scope driver failed ({result.returncode}): {result.stdout[:2000]!r}")
            outputs.append(result.stdout.replace(b"\r\n", b"\n"))
        if outputs[0] != outputs[1]:
            raise ValueError("Native scope traces differ between driver O0/O2")
        return outputs[0]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--cc", default="gcc")
    parser.add_argument("--assembler", default="nasm")
    args = parser.parse_args()
    sys.stdout.buffer.write(run(ReferenceSource(args.reference), args.cc, args.assembler))


if __name__ == "__main__":
    main()
