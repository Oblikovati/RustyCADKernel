# ADR0048 early preparation diagnostic prefix

Source: `1117e2fcff4c87ebd4749322d1ead0c14d29aceb`.
This is a private development prefix; the invocation-bound publication wrapper,
full final integration gate and application acceptance are not complete.

## Executed checks

- Rust 1.99.0, `CARGO_BUILD_JOBS=2` on the shared Linux host; exact version/platform
  and complete source hash bookends are in `preparation-prefix.tar.gz`.
- `cargo test -p rustycad-tkbo --all-features --lib owned_request:: -- --test-threads=1`:
  76 passed, including the existing actual cancellation corpus, the complete
  request codec matrix and 28 diagnostic tests.
- `cargo clippy -p rustycad-tkbo --all-targets --all-features -- -D warnings`: passed.
  `cargo fmt --all --check`: passed. New projection source/test files satisfy the
  focused function/file size audit; no production lint suppression was added.
- Two independent test-binary processes emitted identical 4,333-byte receipts,
  SHA256 `1b96918b7508fedfb093ff7139da58bfd4a7385096149e8de7051f89dbe00203`.
  Eight independent worker results and eight shared immutable readers equal the
  single-thread result. These fixtures compare complete context bytes plus typed
  test observations; they do not establish a production cause persistence format.

The corpus preserves operation class independently of projection completeness,
invalid numeric bits, native exception type/parent/message/stack bytes, actual
child callback phase/ordinal, caller revision/identity and isolated request/native
buffers. Nested census/contact/region and unavailable-stage payloads remain
explicit projection gaps with the original local box retained. A named fixture
proves an empty outer owner inventory cannot hide a nested cause. Identical
geometry with different caller IDs produces different context receipts.

## Retained failures and limits

The first actual-contact fixture incorrectly expected `world_endpoint_limit` as
an existing field label. The unchanged source emits `world endpoint limit`; the
fixture was corrected against `sketch_region_contact_transport/transaction.rs`.
The failed log is retained. Strict Clippy initially found two redundant return
bindings in test helpers; these were removed, and the passing rerun is retained.
No production behavior or legacy expected receipt changed to obtain these passes.

`preparation-tsan-startup.tar.gz` contains the pinned nightly 2026-09-25 control,
runner, build/run logs, binary hashes and exact commands. The instrumented control
built, then exited with SIGSEGV (`-11`) before its main marker. The ordinary
control built and exited zero with its marker. No LD_PRELOAD, host masking or
TSAN_OPTIONS were present. **The actual diagnostic suite was neither built nor
executed under ThreadSanitizer; this verification remains incomplete.**

No native OCCT/GPU workload, performance comparison, complete operation,
publication/refusal corpus or release acceptance is claimed. The private local
retention carrier is not asserted to be a published Send/Sync result. Root review
and final serialized integration remain required.
