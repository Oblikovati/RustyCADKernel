# Martensite selected application owners

`AisInteractiveContext::has_applicative()` and `applicative()` follow the current
selection cursor to the selected interactive object's application owner. The
returned application handle is an owning copy, independent of subsequent owner
replacement or clearing. These queries do not use the last detected object.

Seven committed rows from two identical executions of pinned OCCT preserve null
application owners, replacement, clearing, retained copies, exact intrusive counts,
and cursor advancement to a second actual AIS point. Rust replays the same rows
without OCCT. The native oracle and receipts are in
[the evidence directory](../porting/runtime-evidence/selected-applicative/receipt.json).

Native calls require a selected interactive object and dereference null otherwise.
Martensite diagnoses that invalid precondition with a typed failure; this guard is
not presented as a native exception. Overall context parity remains partial.

```rust,ignore
context.init_selected();
if context.has_applicative()? {
    let application_owner = context.applicative()?;
}
```

Run the affected Rust suite with `cargo test -p rustycad-tkv3d --all-features --all-targets`.
These CPU queries submit no primitives and establish no new renderer baseline.
