# Bounding-volume compatibility notes

Baseline: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
These notes describe implemented behavior, including differences between similarly
named upstream classes. The implementation and oracle use the pinned source as
truth; comments in OCCT are not substituted for executable behavior.

| Operation | Preserved behavior |
| --- | --- |
| `Bnd_Box::Update` | Updates stored finite corners even when a side is open. |
| `Bnd_Box2d::Update` | Leaves an already-open side's stored corner unchanged. |
| `Bnd_Box::Add(box)` | Updates the gap before its whole-box early return. |
| `Bnd_Box2d::Add(box)` | Whole-box early returns can leave the original gap unchanged. |
| `Bnd_Box::Transformed` | Transforms all available finite corners, then open directions. |
| `Bnd_Box2d::Transformed` | For rotation, excludes corners incident on open sides; direction flags alone can leave the result void. |
| `Bnd_Box::IsOut(plane)` | Compares strictly positive equation signs at corners; tangency is not a symmetric non-strict test. |
| `Bnd_Box::IsOut(p1,p2,dir)` | Tests the strip swept by a segment along a direction. |
| `Bnd_Box2d::IsOut(p1,p2)` | Tests a finite segment with a non-strict line-intersection stage. |
| `Bnd_B2::IsOut(p1,p2)` | Uses a strict line-intersection stage; this differs from `Bnd_Box2d`. |
| `Bnd_B2::Limit` | Applies only one limiting branch per coordinate; it does not necessarily compute the geometric intersection. |
| `Bnd_B2::IsIn` | Uses strict containment; an identical box does not count as inside. |
| `Bnd_Sphere::SquareDistances` | Computes estimates from `d² ± r²`, rather than squaring `Distances`. |
| `Bnd_Sphere::Project` | Returns the stored center, center distance, and true inside/success flags. |
| `Bnd_Sphere::Add` | Copy branches copy metadata; expansion retains U/V and invalidates the entity. |
| `gp_Ax3::Scale` | Negative scale reverses all three stored directions, including the normal. |

Float compact-box arithmetic is performed in `f32` wherever OCCT operates in its
`RealType`, and promoted to `f64` where the source promotes it. Storing doubles and
rounding only the final corners would change behavior.

The differential corpus compares finite values, infinities, and signed zeros by
bits. NaNs are compared by classification, with raw sign/payload differences
retained. See [TKMath progress](tkmath-progress.md) for the current numeric scope
and the separately recorded matrix NaN representation difference.

These notes and passing legacy suites do not establish complete API coverage.
Unimplemented overloads and remaining transform forms stay in the port's scope.
