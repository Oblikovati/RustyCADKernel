# Orientation reconciliation with reviewed lineage and tolerance

Merge `c23deb36fd8c05fa28287f199f3b56198290be12` combines the original orientation
source/evidence (`e60b9542f`, `d9632ae2b`) with reviewed main `e6955e6db`.
The merge was clean. All orientation implementation files remain unchanged;
main's test-module visibility change coexists with the orientation registration.
The original evidence and its cited revisions are preserved without rewriting.

On Rust 1.99.0 with two Cargo jobs, offline locked dependencies:

- 22 focused orientation tests passed, including two-process byte replay and
  eight independent workers with four operations each.
- The original embedding replay test passed unchanged.
- Orientation and embedding portable receipts equal their original frozen bytes.
- TKBO all-target/all-feature Clippy with `-D warnings` and workspace formatting
  passed.
- All 15,830 tracked source/configuration file hashes match before and after.

`receipt.json` retains exact commands, elapsed times, source revision and hashes
for all 16 candidate source/ADR files relative to merged main. `runs.tar.gz`
contains 33 sealed files: raw logs, source bookends, candidate files and runners.
The original full matrix of 4,447 affected tests and 88 doctests remains recorded
in the parent directory. The coordinator will run the combined affected TKBO
matrix after integration; this reconciliation does not claim that additional run.

Ordinary-host ThreadSanitizer was not repeated: its original executable crashed
before test output, so sanitizer acceptance remains incomplete. No instrumentation
or protection changes were made. Orientation remains conditional evidence over
the admitted embedding and does not complete body validation or Kernel#10.
