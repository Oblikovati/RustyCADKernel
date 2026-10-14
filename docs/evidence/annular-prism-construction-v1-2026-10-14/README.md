# Actual annular construction and structural chart capture

Source `9e9634893`, original-path baseline `4e82b841e`, reviewed fixture/encoding
dependency `94e47df87`, main base `0a429b1af`. ADR0050 records the scope.

The actual retained operation constructs four faces, six wires, six edges, four
vertices and twelve coedges. The fresh cumulative-occurrence graph Audit has no
issues. Both cap wires and both side seams are present; source IDs 101 and 202
select independently authored cubic coefficients. Four trim roles per side have
independent pcurve formulas/ranges and the seam uses share one physical edge with
opposite orientation. All four actual chart inspections succeed. The operation
still returns Incomplete and no achieved-error/body-validity certificate.

`gate.json` and archived logs record 1,701 TKBO tests plus 35 doctests. Strict
all-target/all-feature Clippy and workspace formatting checks passed. Explicit
source manifests for lint and both final replay runs are identical; no source
edits occurred during the full test run. The seven baseline payloads equal both
final independent processes byte for byte. Existing replay tests also include
thread comparisons; no new thread-safety claim or TSan waiver is made.

The structural helper preserves current history, actual face enumeration, V1
face-count admission, missing-image checks, chart callbacks and final V1 role
admission in that order. New focused cases retain typed early admission refusal
and reject duplicated actual history before the admission callback. An annular
four-face test callback admits individual chart capture only. Full source-domain,
prism-copy, exact face-domain and publication composition remain separate work.

`replay_gate.py PHASE` reruns all seven replay commands and records source
bookends. The baseline receipt predates extraction; final-first/final-second
receipts identify frozen source. `artifact-seal.json` hashes the retained evidence
files excluding itself. No live OCCT oracle was used.
