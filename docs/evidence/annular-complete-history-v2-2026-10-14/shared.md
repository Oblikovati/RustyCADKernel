# Shared scalar and sweep extraction

Baseline fb49a31ce and actual fixture b3f7a8763 remain unchanged. Native borrowed
inputs contain bridge/receipt/publication references only; they grant no proof
construction authority. Parent scalar mapping now receives only publication
bindings. Existing sweep and native budget bodies use this view; V1 wrappers
retain admission/order/error paths. Narrow scalar/context/copier helpers have
frozen-subtree visibility for the upcoming annular orchestration.

46 existing history tests pass, including all budget/callback/refusal cases.
All eight full replay payloads compare byte-for-byte equal to baseline. Strict
all-target/all-feature Clippy passes after moving a new helper above the existing
test module; the initial lint refusal is retained. cargo fmt applied. Source
manifest is shared-source.json; no annular history success is implemented yet.
