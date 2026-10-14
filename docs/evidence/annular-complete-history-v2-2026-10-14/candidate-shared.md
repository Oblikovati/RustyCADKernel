# Shared candidate, partition metadata and diagnostic projection

Unchanged-body extraction follows bd41ee8d5. Candidate member/copy-tree and
generated-member projection now accept the narrow native view; original wrappers
resolve the same V1 operands in the same order. Partition metadata helpers reuse
the same native endpoint association checks. Native diagnostic admission keeps
its exact original cause/local owner envelope and classification. No annular
history issuer or acceptance is added by this structural prefix.

46 existing history tests and strict all-target/all-feature Clippy pass. All eight
baseline complete replay payloads match byte-for-byte; the final focused process
again matches the original 421,702-byte history payload. A temporary borrowed-view
lifetime compile refusal and an overly wide old-wrapper visibility lint refusal
are retained with their corrected successful reruns. Neither fix changes values,
order, admission, classification or callbacks. cargo fmt applied. Source hashes:
candidate-source.json. Full final-source integration gate remains separate.
