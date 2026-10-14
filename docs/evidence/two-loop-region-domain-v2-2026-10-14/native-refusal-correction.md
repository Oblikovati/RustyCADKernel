# Native refusal class correction

After structural cleanup `2e1655f61`, two new regressions failed: a retained
native copied-tree read failure and a generated-universe mapped native failure
were classified Incomplete. Their original causes were retained, but the class
was wrong. The bounded classifier now preserves Failed through those wrappers;
region/partition capture causes delegate to the existing surface classifier.
No admission or successful proof operand changes.

Rust 1.99/jobs2 focused `cargo +1.99.0 test -p rustycad-tkbo --all-features
two_loop -- --test-threads=1 --nocapture`: 48 tests passed. The original red
command used `two_loop::native_refusal_tests -- --nocapture`; both tests failed.
Exact red/green logs are retained. Complete V2 bytes remain identical to the
43,373-byte pre-cleanup fixture. Full final gates follow separately.
