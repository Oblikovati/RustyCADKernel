# Martensite irrational joint-root construction fixture

The existing general pair pipeline now additionally proves actual curves
A(t)=(t,t²), B(u)=(u,1-u²) on [0,1]². Fx forces t=u and Fy gives
2t²-1=0. Independent Jacobian isolation confirms one positive root in this
domain. This is a test/proof addition, not a new specialized solver path.

The Rust regression requires a non-singleton certified t bracket and exact
lo²<1/2<hi² inequalities. It independently bounds squared construction distance
by the maximum endpoint x-distance squared plus (constructed_y-1/2)², using
exact rational arithmetic. The outward reported world radius must dominate that
bound and meet the unchanged requested1e-6 limit. No point residual or floating
sqrt is treated as a root-error certificate. The world envelope also contains
the exact y=1/2. Earlier quarter-point/native fixtures remain unchanged.

Compact Linux execution:25 pair tests pass; all-target/all-feature Clippy,
formatter, focused max20-function/file-under500 audit pass. No benchmark or
whole public-provider/application acceptance. Source-bound policy/accounted
budget and remaining complete/incomplete report acceptance stay open.
