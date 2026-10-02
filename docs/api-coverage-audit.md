# API coverage audit

This is a snapshot of the callable-member audit for the pinned OCCT baseline.
It complements `porting/parity.json`; it does not certify complete API coverage.

## Snapshot

| Measure | Result |
| --- | ---: |
| Unique public package directories scanned | 408 |
| Public header members without ledger rows | 37,574 |
| Ledger rows matching no scanned public member | 0 |
| API rows currently in `porting/parity.json` | 26,957 |

The source baseline is OCCT commit `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The audit enumerated unique package paths in `porting/inventory.json` and ran
`tools/audit_api_rows.py <package> --source <OCCT checkout>` for each path. The
inventory has 410 package entries because two paths are shared references; those
duplicates were scanned once.

## Limits

`audit_api_rows.py` is a heuristic C++ declaration scanner. It checks public
member functions, constructors, operators, nested enums, and the ledger's
excluded-member rules. It is not a compiler front end and does not establish
coverage for every type, constant, public data field, template contract,
macro-generated API, inherited extension point, generated header, command API,
or build/platform configuration.

The missing-member count is a concrete backlog, not a coverage percentage: API
ledger rows include explicit partial/excluded mappings and other identities that
are not in this scanner's denominator. `api_inventory_complete` therefore stays
false, and the project must not report 100% API coverage from this audit.

The largest callable-member gaps in this snapshot are Graphic3d (1,724), OpenGl
(1,256), AIS (1,213), StepBasic (999), and IFSelect (855). Continue the audit by
toolkit and dependency order, reconcile every scanner gap, and extend inventory
for API categories the heuristic cannot parse.
