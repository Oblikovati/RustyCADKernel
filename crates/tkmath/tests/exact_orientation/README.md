# Exact orientation evidence

This is an extension corpus, not a claim of mapped OCCT parity. The owning
[decision](../../../../docs/adr/0003-exact-extension-orientation.md) preserves
existing mapped behavior. The application still consumes Martensite v0.2.1;
this local kernel addition does not change its release or enable general extrude.

`oracle.csv` stores six binary64 bit patterns and an exact sign per row. Python
`Fraction` computes the homogeneous 3x3 determinant by cyclic expansion, without
the production integer-unit conversion or translated 2x2 expression. The 391
cases include 256 rounded near-collinear constructions, 128 random finite bit
patterns, exact zeros, signed zeros, repeated points, minimum subnormals,
maximum finite values, cancellation, and the Go audit's `1e-200` underflow case.
Inputs are stored as bits so fixture parsing introduces no decimal conversion.

Regenerate with `python3 crates/tkmath/tests/exact_orientation/generate.py` and
compare its output byte-for-byte with `oracle.csv`. Rust tests check all six
point permutations and all 2047 finite exponent fields. They also require the
corpus to contain more than 50 cases where a rounded determinant is wrong or
nonfinite. Every input position is tested with NaN and both infinities; error
identity, display text, and first-error order are checked.

For the cancellation triangle `(0,0), (1+2^-52,1), (1,1-2^-52)`, Jacobian's
`matrix.determinant.compute` independently returned the exact determinant
`-1/20282409603651670423947251286016` (2026-10-13). The Go underflow probe is
recorded in Oblikovati's `architecture/audits/` kernel comparison dated
2026-10-12, section 5.4. Its correct positive sign follows from positive axis
lengths; the incorrect Go zero is not treated as an oracle. No native OCCT,
Python, or Jacobian process is required to run the Rust regression corpus.

Starting kernel revision: `ede3518dbd7c2ae5e49d54cfef3e5bcf9029ae9e`.
Validation runs on local Linux. Windows/macOS execution, production consumer
acceptance, a filtered fast path, and a released dependency remain unverified.

Local validation on 2026-10-13 used Rust 1.99.0 and Python 3.12.3:

- `cargo test -p rustycad-tkmath --all-features --locked`: 11 unit tests,
  2,730 integration tests, and 37 documentation tests passed.
- `cargo clippy -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings`
  passed. Changed Rust files passed `rustfmt --check`.
- `cargo llvm-cov -p rustycad-tkmath --test toolkit --all-features --locked
  --json --output-path /tmp/exact-orientation-coverage.json -- exact_orientation`
  measured all 33 lines, 65 regions, and four functions in `src/predicates.rs`
  covered. This is coverage of the new primitive, not toolkit-wide coverage.
- Corpus regeneration was byte-identical. Markdownlint passed for both new
  documents. Jscpd found zero duplicate lines in the two new Rust files.

No application code or mapped OCCT numerical expression changed. The primitive
uses integer shifts of at most 2045 bits and no platform-sized coordinate casts,
map iteration, native geometry, or floating determinant. These are source checks;
platform execution remains limited to Linux. No push or release was performed.
