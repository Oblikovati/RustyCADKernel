# Martensite shared polynomial engine entry

Standalone branch extension/martensite-shared-polynomial-solve starts at checked
main eebf3997e. It only moves the existing private solve_polynomial and
solve_polynomial_accounted bodies into roots/polynomial_solve.rs, retaining their
original aliases under bezier_line::solve. There are no pair-provider sources,
refinement changes, numerical-policy changes, new mapped APIs or native fallbacks.

The equivalence manifest verifies complete function-token equality after only
IntegerPolynomial qualification depth and formatter whitespace normalization.
Original polls, successful-split events, no-op compatibility observer, source
ownership and failure ordering therefore remain the same expressions.

Executed Linux checks with debug0/incremental0/two jobs:
`cargo test -p rustycad-tkmath --all-targets --all-features` passed2933 reported
cases (51library,2882toolkit); benchmark and example executables completed0.
`cargo clippy -p rustycad-tkmath --all-targets --all-features -- -D warnings`
completed0; doctests and formatter completed0. Existing root_signs bench prints
elapsed_ns during all-target functional checks; that contended output is not
accepted performance evidence. No benchmark timing acceptance was run.

No global ledger/inventory edits or API credits. Root integrates the whole
structural branch/checks/deletes it before the unfinished pair branch consumes
checked main. Its strict world-error reds and broader unfinished scope remain
separate and preserved.
