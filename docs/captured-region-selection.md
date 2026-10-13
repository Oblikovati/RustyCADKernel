# Captured candidate winding

Martensite's private issue #10 candidate selector now uses the checked
operation-owned `CapturedBezierWireStore`. The mapped legacy per-wire classifier
and native arrangement/candidate builders remain unchanged. This removes only
the demonstrated `SharedIncidenceReuse` gap; region/body completion remains
incomplete for construction and plane/parameter transport, complete candidate
search/history, self-intersection and tolerance.

One operation fixes the full requested root policy and seed ray. The consumer
assigns deterministic operation-local tokens to actual copied edge identities
and original trim domains. Orientation and current occurrence IDs remain outside
the cache key. The math store additionally binds exact coefficient/weight/trim
bits, so unrelated physical sources or changed scalar captures do not reuse a
proof merely because points are near or equal. These tokens are not persistent
application names or addresses.

The result owns the actual proof store and current occurrence/parent bindings.
It reports newly executed subdivisions, total charged operation work, reused
certificate count and historical proof costs separately. Failed or cancelled
queries retain their original typed cause, the owned complete-proof prefix and
actual charged work. No failed work is refunded and no partial proof is admitted
as complete. Successful warm traversal still checks the current IDs, directed
endpoint joins and exact wire closure.

The two opposite candidate traversals of one actual copied closed cubic share
one numerical proof. Reversal changes the winding contribution and endpoint
order; boundary results attribute both actual current occurrence IDs. A budget
consumed by the cold capture permits a later warm traversal with no new solver
work. Independent requests use fresh stores, and scalar changes invalidate reuse.

Native sampled cusp/complement discrepancies retain their existing diagnostic
status. The selector still returns `CandidateClassification::Incomplete`, with
unknown achieved geometric error, even when an exact seed selection is unique.
No source construction bound, split/root correspondence, prism theorem, healthy
body, naming, mapped API or application completion credit follows from reuse.
The exact dyadic metric and general pair/self providers must be adopted only
from checked integrated main; this change includes no peer work in progress.

Run the affected Rust regressions with:

```sh
cargo test -p rustycad-tkbo -p rustycad-tkmath --all-targets --all-features
```
