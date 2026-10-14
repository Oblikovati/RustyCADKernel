# Exact coordinate comparison oracles

`oracle.py` generates 296 cases using Python `Fraction` arithmetic on the exact
supplied binary64 values. The seed is 3563. Rows retain eight hexadecimal bit
patterns followed by the exact signs of:

- `|b-a|² - |c-a|²`, for squared-distance ordering;
- `(b-a) · (c-d)`, for projection ordering along `a` to `b`.

The first eight cases use ordinary representable metric values. The remaining
cases include subnormals, exponent extremes, adjacent floats, signed zeros,
coincident points, perpendicular vectors, and deterministic generated inputs.
The Rust tests also check all 2,047 finite exponent fields, operand reversals,
non-finite argument precedence, and finite false ties from rounded arithmetic.

`occt_oracle.cpp` uses OCCT 8.0.0 `gp_Pnt2d::SquareDistance` and `gp_Vec2d::Dot`
for the first eight ordinary cases. `occt-8.0.0.csv` retains case index and both
signs. Native rounded metrics are not an oracle for extreme or near-tie cases;
the exact rational corpus supplies those expected signs. Rust tests require
neither Python nor OCCT at runtime.

```sh
python3 crates/tkmath/tests/exact_comparisons/oracle.py > /tmp/comparisons.csv
cmp /tmp/comparisons.csv crates/tkmath/tests/exact_comparisons/oracle.csv
g++ -std=c++17 -I../occt-install/include/opencascade \
  crates/tkmath/tests/exact_comparisons/occt_oracle.cpp \
  -L../occt-install/lib -Wl,-rpath-link,../occt-install/lib \
  -lTKMath -lTKernel -o /tmp/comparison-oracle
LD_LIBRARY_PATH=../occt-install/lib \
  /tmp/comparison-oracle /tmp/comparisons.csv \
  > /tmp/comparisons-occt.csv
cmp /tmp/comparisons-occt.csv \
  crates/tkmath/tests/exact_comparisons/occt-8.0.0.csv
cargo test --locked -p rustycad-tkmath --test toolkit exact_comparisons
```

Both predicates order supplied coordinates. They do not establish geometric
construction accuracy or complete intersection roots. See ADR 0013 for the
application deletion and release gate.
