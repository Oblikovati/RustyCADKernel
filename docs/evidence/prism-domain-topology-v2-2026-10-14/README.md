# Face-domain topology prerequisite

Source `c8559bf9a` passes 1,698 TKBO tests and 35 documentation tests, strict
all-target Clippy, formatting and warning-denied generated documentation.
The source bookends are unchanged. All 1,689 previous-main passing test names
remain present; nine annular finite-incidence tests are added.

The structural change (`14d5fb4b4`, `e17315410`) extracts two internal policy
hooks: face-domain admission and Euler evaluation. V1 retains its original
single-wire check, witnesses, callback order, sphere characteristic and complete
certificate bytes. It is still the only production policy and entrypoint.

Six independent process executions preserve three full prior outputs:

| Receipt | Bytes | SHA-256 |
| --- | ---: | --- |
| Topology V1 | 96,088 | `852bd440732f89512ab35e27c2f13b70d00b9af818c3dc3266122b3bee75a9a7` |
| Complete private publication | 2,202,990 | `accba93e30c455d9f888a96c5b70f684a1dbacbbf3afec9076de3544c7a13107` |
| Conditional physical volume | 601,860 | `15f229b2a11115e2c5f4a7f37e5e845e0b5576f71aa83d8ec865de39941f180e` |

The new fixtures exercise the same incidence implementation with **test-only**
annular assumptions. Their four faces have parameter-domain contributions
`0 + 0 + 1 + 1`, giving `4 - 6 + 2 = 0`. An independent cell model cuts both caps
along authored radial edges. Exact rational boundary ranks are 3 and 3, with
Betti numbers `(1, 2, 1)`. The retained Jacobian results and standalone
`verify_boundary_words.py` agree. These are finite cell words, not tessellated
geometry or an oracle for a production annular solid.

Tests retain full face/wire membership and source binding, reject missing,
duplicate, foreign and unused assumptions, exercise opposite edge uses and
pinched vertex links, and fail/cancel at every observed callback. Four workers
also read the same immutable fixture concurrently. No test policy can publish
a topology certificate or successful body. Its connected genus-zero domains
and inner/outer orientations remain explicit assumptions, not geometric proofs.

`final/` contains the complete matrix, six replay logs/outputs, prior-name check,
source bookends and runners. The generic runner's `integrated-*` filenames here
describe the **candidate checkout** at the recorded source revision; root main
integration is a separate gate. `baseline/`, `structural/` and `domain-fixture/`
retain the earlier checkpoints. The compiler cache was shared with root's
`target` directory; source inputs were this worktree's recorded files.

Actual plural join/region binding, production face-domain issuance, full annular
validation/history/measures, application workflow and release remain open.
ThreadSanitizer acceptance remains incomplete. No new OCCT execution occurred.
