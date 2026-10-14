# ThreadSanitizer startup diagnosis

Status: **incomplete**. No actual operation suite passed ThreadSanitizer here.
The [receipt](receipt.json) seals the retained source, commands, logs, executable
and archive hashes. This extends the earlier owned-refusal startup attempt;
it does not replace its failed evidence.

## Observations

- The unchanged instrumented control from nightly 2026-09-25 (LLVM 23.1.1)
  failed three of three ordinary executions with SIGSEGV before its `main`
  marker. Standard output and standard error were empty.
- The corresponding ordinary control entered `main` and exited successfully
  in all three executions. Both executable hashes match the prior archive.
- A targeted syscall trace records the system loader reading
  `/etc/ld.so.preload` and loading `libAppProtection.so`, followed by its
  dependencies. It ends with a null-address segmentation fault before `main`.
  Library injection is observed; its responsibility for the fault is **unproved**.
- The same control source built with the installed newer nightly
  `1.101.0-nightly (8d1a76430 2026-10-06)`, LLVM 23.1.3, also failed before
  `main`. Its recorded build uses `-Zsanitizer=thread`,
  `-Cforce-frame-pointers=yes`, `-Zbuild-std` and the explicit Linux target.
  The newer runtime does not resolve this host's observed startup failure.

The trace is diagnostic evidence, not a sanitizer acceptance run. No host
protection, loader configuration, personality or kernel setting was changed.
The denied `mmap_rnd_bits` read supplies no value or causal conclusion.

## Retained artifacts and next gate

`diagnosis.tar.gz` contains the repeated-run observations, syscall trace, ELF
headers, newer control source/manifests, build/run logs and exact newer binary.
The previous instrumented and ordinary binaries are referenced by verified hash
in the existing `owned-prism-refusal-v1-2026-10-14/tsan-startup-attempt.tar.gz`.
Compiled standard-library objects and incremental build directories are excluded.
All 29 new archive entries were read back and checked against the receipt.

Acceptance needs an ordinary supported runner on which the negative control
enters `main` and an intentional-race positive control reports its race, followed
by the fully instrumented actual operation suite. These remaining executions
have not occurred. The startup failure cannot waive the operation gate.

Compiler references used in the investigation:
[Rust ThreadSanitizer](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/sanitizer.html#threadsanitizer)
and [Clang limitations](https://clang.llvm.org/docs/ThreadSanitizer.html#limitations).
