/* SPDX-License-Identifier: AGPL-3.0-or-later */
/* Freestanding process I/O only; no CRT, host game state or Rust FFI. */
typedef unsigned long Word;
_Static_assert(sizeof(Word) == 4 && sizeof(void *) == 4, "Selected scope helpers require x86-32");

#if defined(_WIN32)
typedef void *Handle;
__attribute__((dllimport, stdcall)) Handle GetStdHandle(Word);
__attribute__((dllimport, stdcall)) int WriteFile(Handle, const void *, Word, Word *, void *);
__attribute__((dllimport, stdcall, noreturn)) void ExitProcess(unsigned int);
static void scope_write(const char *bytes, Word length)
{
    Word written;
    if (!WriteFile(GetStdHandle((Word)-11), bytes, length, &written, 0) || written != length)
        ExitProcess(91);
}
static __attribute__((noreturn)) void scope_exit(unsigned int code) { ExitProcess(code); }
#else
static void scope_write(const char *bytes, Word length)
{
    long written;
    __asm__ volatile("int $0x80" : "=a"(written) : "0"(4), "b"(1), "c"(bytes), "d"(length) : "memory");
    if (written != (long)length) {
        __asm__ volatile("int $0x80" : : "a"(1), "b"(91) : "memory");
        __builtin_unreachable();
    }
}
static __attribute__((noreturn)) void scope_exit(unsigned int code)
{
    __asm__ volatile("int $0x80" : : "a"(1), "b"(code) : "memory");
    __builtin_unreachable();
}
#endif

static void text(const char *value)
{
    static Word output_bytes;
    Word length = 0;
    while (value[length]) ++length;
    if (length > 65536 - output_bytes) scope_exit(92);
    output_bytes += length;
    scope_write(value, length);
}
static void decimal(Word value)
{
    char bytes[16];
    Word count = 0;
    do { bytes[count++] = (char)('0' + value % 10); value /= 10; } while (value);
    while (count) scope_write(&bytes[--count], 1);
}
static __attribute__((noreturn)) void unexpected(const char *name)
{
    text("unsupported native service: "); text(name); text("\n"); scope_exit(90);
}
