# Checked GUI surface outcomes integration

Integrated source: `15f2367e6`. Candidate source: `89d2c0a1e`;
candidate evidence: `216845a08`. The coordinator ran the committed `replay.py`
from integrated main with exclusive GPU ownership and the pinned validation layer.

- AMD and lavapipe: **836 tests passed per driver**, zero failed, six ignored.
  The new ignored native phase test ran explicitly on both drivers: all 24 cases
  passed. The five earlier ignores remain excluded.
- Native window/display lifecycle and GUI acceptance passed on both drivers.
  Seven separate rapid-replacement processes passed the unchanged exact
  framebuffer checker. All **174 capture files are byte-identical to the
  candidate's archived captures**, including both policies and both drivers.
- Two independent host processes emitted identical canonical results, each
  checking serial and parallel operations. Native concurrent operation safety
  is outside that host-only check.
- All-target Clippy with warnings denied, documentation with warnings denied,
  formatting and eight release-plan tests passed. The doctest command passed
  with zero executable doctests.
- All **38 changed crate/tool files** match the candidate exactly. Source hashes
  were unchanged throughout the replay and were rechecked before archiving.
  The candidate archive and all 271 of its retained file hashes were verified.

`integrated-main.json` contains counts, candidate comparisons, capture hashes and
the manifest for all 214 files in `integrated-raw.tar.gz`. The archive includes
commands, full logs, source snapshots, capture bytes, relative checker-link
mapping, binary hashes, and paired timing/RSS observations. The retention script
is preserved as `integrated-retain.py.gz`.

The paired timing runs use identical accepted workloads, 20 warmups and 100
samples, in an unoptimized debug build on a shared host. They are observations,
not controlled previous-release performance acceptance. ThreadSanitizer remains
incomplete after its recorded startup failure; the GUI sanitizer suite was not
executed. No release, application acceptance, default-policy migration, physical
display visibility, arbitrary native race safety, or full API/test parity is
claimed. LegacyV1 remains the default and CheckedV2 remains opt-in.
