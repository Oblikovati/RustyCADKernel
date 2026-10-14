# Shared orientation primitives: structural checkpoint

Baseline `4e75cb9dabc8453d690ec84b3f7d6bb685761337`; structural source
`4ea2c0b79dbcc0805395ca17f3809abd263b9386`, 2026-10-14.

The existing determinant/unit-sign/outward-face helpers have broader private
visibility. The existing trim-direction expression moved into a shared primitive
at the same call position, after unsupported-occurrence admission and before
actual-direction comparison. No winding policy, callback, error, domain or
authority change is introduced. Two literal sign-table tests cover the exposed
primitive. The future annular consumer is not implemented by this checkpoint.

Executed with Rust 1.99.0, `CARGO_BUILD_JOBS=2` and isolated target
`/tmp/martensite-annular-orientation-target`:

```sh
cargo +1.99.0 test -p rustycad-tkbo --lib --all-features orientation
cargo +1.99.0 clippy -p rustycad-tkbo --all-targets --all-features -- -D warnings
cargo +1.99.0 fmt --all -- --check
```

The orientation filter passed 68 tests before extraction and 70 afterward.
Strict Clippy and formatting passed. Three payloads were each emitted by two
independent processes before and after extraction; all complete bytes match:

| Payload | Bytes | SHA-256 |
| --- | ---: | --- |
| Orientation V1 | 752 | `67df03c3156f98f17484f304d3a67078534778fa6fbe87882c8234ef5ac50f31` |
| Joined orientation V1 | 215,177 | `d642f0f81b03da8689cd7930d2fa27b269faec055d9629bdb507422bcd321968` |
| Publication V1 | 2,202,990 | `accba93e30c455d9f888a96c5b70f684a1dbacbbf3afec9076de3544c7a13107` |

The exact test selectors are visible in each process log. Tracked-file manifests
identify both revisions; their expected differences are the five structural/test
source files. Earlier baseline bytes remain retained. This is a focused structural
check, not a full toolkit or annular acceptance gate. No OCCT, GPU, sanitizer,
release benchmark or application acceptance execution is claimed.
