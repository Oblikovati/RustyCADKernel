# Release reference maintenance integration

Main `12faf0730` integrates source `255f626f905d62bfdd6688e21e6e2b0c6c8dfc7d`.
Root review confirmed the narrow exact direct-reference equality check and
rejection of symbolic aliases, changed tag objects (including reannotation of
the same commit), inconsistent old objects and lookup errors. New release tags
continue through the original receipt, source, annotation, plan and package gates.
All 15 verification functions are AST-identical; the hook script is unchanged.

Root checks passed **94 release-tool tests**, **8 verification-gate tests** and
**4 pre-release test modules**. The installed hook was exercised only in isolated
temporary repositories, including two pack-refs runs, garbage collection,
new/changed tag refusal and the existing verified-candidate/tampered-assets flow.
All candidate source files and 28 archived payload hashes match. Tool/hook source
bookends are identical. Candidate Black 25.1.0 formatting is retained unchanged.

The original failure was a zero-old-OID packed-storage insertion for an already
existing logical annotated tag. Exact unpeeled object equality proves logical
no-op maintenance; it does not authorize creation or replacement. No actual
release tag, repository configuration, host protection, remote state or release
publication was changed by these tests. Kernel #10 release authorization remains
conditional on its complete capability and gates.
