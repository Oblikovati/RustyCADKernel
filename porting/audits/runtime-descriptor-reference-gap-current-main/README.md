# Native RTTI descriptor-reference gaps

This audit identifies twelve existing implemented claims whose legacy Rust
selectors expose descriptor pointees or pointee arguments rather than genuine
native handle references. It requests partial status pending checked source
integration; it does not refresh the migration as unchanged behavior.

The two row files preserve exact upstream IDs, compiler declaration IDs, native
signatures, expansion/spelling positions, upstream header hashes, current ledger
selectors, and current compiler Rust bindings. Rust source anchors refer to the
main implementation, including macro invocation anchors where Rustdoc reports
expanded methods. The audit changes no global ledger or generated inventory.

The pinned seven-row derived-sequence generator proves a getter can return an
actual mutable descriptor slot, change the original caller handle, and fail after
the change. Retained aliases observe that same field without rerunning the getter.
The old pointee APIs cannot express those contracts. See the linked checkpoint
receipt for both identical native runs and Rust replay provenance.

The locally authored required field protocol has not landed on main. Class claims
remain partial and the ordinary chain retirement adapter remains a separate
unfinished contract. Replacement bindings require a fresh compiler capture after
the actual source migration is checked; these rows supply no such promotion.
