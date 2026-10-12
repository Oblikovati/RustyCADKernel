# Martensite camera self-assignment regression

Pinned native V3d/CView execution records three exact slot-identity, target-identity
and intrusive-count rows for Camera self-alias and same-target assignment. Two
native runs are byte-identical. The Rust-only regression runs in a worker with a
three-second completion bound, so a future recursive lock cannot hang its suite.

The ORIGINAL production setter passes the regression. Rust evaluates the right
operand of assignment before its assignee place; cloning the argument therefore
precedes the destination write guard. The initial static deadlock suspicion was
disproved by executable evidence. This checkpoint adds regression/provenance
only, without a production fix, API credit or global downgrade.

Run `cargo test --locked -p rustycad-tkservice --all-targets`.
