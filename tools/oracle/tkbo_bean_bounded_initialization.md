# BeanFace explicit bounds overloads

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The probe calls the ten-argument native constructor and `Init` overload on six
line/plane configurations. Its 24 snapshots cover initial state, `Perform`,
reinitialization with different bounds and tolerances, and the next `Perform`.
The integration test replays the captured output, including retained completion
state and cleared results after `Init`.

Rust groups the six range/bounds scalars into `Range` and `[f64; 4]`, and the two
tolerances into `[f64; 2]`. Initialization order remains native: copy adaptors
and tolerances, initialize surface state, then assign explicit curve and surface
bounds. Supplemental coverage does not prove arbitrary adaptor copy/cache
semantics or full BeanFace behavior, and does not map new legacy declarations.
