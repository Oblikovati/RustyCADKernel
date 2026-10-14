# Embedding registration reconciliation

Source `ed0475f8cd1bd42df3d120e7a27e49779901cb96` merges main `f4cd79ae2`
with no manual edits. Of the 34 candidate crate files, only the shared
`region_certificate/mod.rs` registration changed: two lines add the integrated
embedding module. `registration.diff` retains that exact change; the compressed
upstream source diff retains the whole imported main delta.

All 39 tolerance tests and workspace formatting pass. All 15,804 tracked build
source input hashes match before/after. The original full matrix and earlier
replay receipts remain unchanged. This registration-only check does not replace
the integration owner's final combined matrix.
