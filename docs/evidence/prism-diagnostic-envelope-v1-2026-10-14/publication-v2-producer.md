# ADR0048 publication V2 producer prefix

Source: `8d04f0896`. Baseline `75dab08f3`; structural extraction `b83215772`.
Rust 1.99, jobs 2, ordinary Linux host. Before/after hashes match for all 12,858
Rust/build files through the recorded checks. The archive retains the full logs,
replays and hash maps; the JSON records exact commands and toolchain.

- 42 focused publication tests passed, including the existing V1 corpus, three
  pinned local failure baselines and nine additive V2 tests.
- 35 TKBO doctests passed; strict all-target/all-feature Clippy and workspace
  formatting passed.
- Two fresh processes each produced identical V1 publication, V2 publication and
  prior typed preparation diagnostic bytes. V1 publication remains 2,202,990 bytes
  (`accba93e30c455d9f888a96c5b70f684a1dbacbbf3afec9076de3544c7a13107`).
  V2 is 2,202,990 bytes (`e68be877959b90ae723b2386ae83db09e0a169149571433d5f3e01fe1be7d878`);
  the independently expected change is the requested publication version field.
  Every child proof is compared exactly. Typed preparation remains 5,807 bytes
  (`a27a9465306496b6d8fa5c732e80bc39e7a2976aa6a074ae67612bddfad196a8`).
- Four independent V2 worker operations matched serial complete result bytes.
  Existing input-isolation and lock-free result-reading tests also passed.

The explicit V2 entry retains callback child phase/class/cause before outer
callback precedence replaces the header. Tests cover all three original child
classes and all three callback stop variants, nested and absent children,
actual cancelled validation, checkpoint exhaustion without calling the observer,
version mismatch with no callback, and successful V2 labeling. V1 remains exact.

This is a private producer prefix, not the sealed invocation diagnostic adapter.
The parent requested source review before extending that adapter. Full toolkit
and integrated operation gates remain for the combined candidate. ThreadSanitizer
remains incomplete because of the already recorded unchanged-host startup
failure; no redundant control was run. No native OCCT, GPU, controlled release
performance comparison, public refusal completeness or application acceptance
is claimed.
