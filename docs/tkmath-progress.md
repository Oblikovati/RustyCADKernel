# TKMath port status

**In progress; toolkit acceptance is not complete.** The source inventory contains
120 GoogleTest files and 2023 declaration candidates. 93 complete source suites
currently account for 1,835 mapped declarations, with matching native and Rust passes.

| Source | Rust target | Declarations |
| --- | --- | ---: |
| gp_XYZ_Test.cxx | gp_xyz | 17 |
| Bnd_Range_Test.cxx | bnd_range | 54 |
| gp_Mat_Test.cxx | gp_mat | 1 |
| gp_Pnt_Test.cxx | gp_pnt | 16 |
| gp_Trsf_Test.cxx | gp_trsf | 1 |
| gp_Dir_Test.cxx | gp_dir | 18 |
| gp_Vec_Test.cxx | gp_vec | 18 |
| gp_Ax3_Test.cxx | gp_ax3 | 1 |
| gp_GTrsf_Test.cxx | gp_gtrsf | 2 |
| gp_Pnt2d_Test.cxx | gp_pnt2d | 14 |
| gp_Dir2d_Test.cxx | gp_dir2d | 14 |
| gp_Vec2d_Test.cxx | gp_vec2d | 17 |
| gp_Lin_Test.cxx | gp_lin | 13 |
| gp_Pln_Test.cxx | gp_pln | 11 |
| Bnd_Box_Test.cxx | bnd_box | 47 |
| Bnd_Box2d_Test.cxx | bnd_box2d | 38 |
| Bnd_Sphere_Test.cxx | bnd_sphere | 20 |
| Bnd_B2_Test.cxx | bnd_b2 | 18 |
| Bnd_B3_Test.cxx | bnd_b3 | 21 |
| gp_Circ_Test.cxx | gp_circ | 8 |
| gp_Quaternion_Test.cxx | gp_quaternion | 5 |
| math_DoubleTab_Test.cxx | math_double_tab | 11 |
| Poly_ArrayOfNodes_Test.cxx | poly_array_of_nodes | 2 |
| math_Vector_Test.cxx | math_vector | 31 |
| math_Matrix_Test.cxx | math_matrix | 11 |
| math_Gauss_Test.cxx | math_gauss | 12 |
| math_Crout_Test.cxx | math_crout | 11 |
| math_GaussLeastSquare_Test.cxx | math_gauss_least_square | 14 |
| math_Householder_Test.cxx | math_householder | 10 |
| math_Jacobi_Test.cxx | math_jacobi | 11 |
| math_SVD_Test.cxx | math_svd | 13 |
| math_EigenValuesSearcher_Test.cxx | math_eigen_values_searcher | 30 |
| math_ComputeKronrodPointsAndWeights_Test.cxx | math_compute_kronrod_points_and_weights | 1 |
| math_BracketedRoot_Test.cxx | math_bracketed_root | 14 |
| math_BissecNewton_Test.cxx | math_bissec_newton | 12 |
| math_BracketMinimum_Test.cxx | math_bracket_minimum | 13 |
| math_BrentMinimum_Test.cxx | math_brent_minimum | 16 |
| math_NewtonFunctionSetRoot_Test.cxx | math_newton_function_set_root | 14 |
| math_FunctionSetRoot_Test.cxx | math_function_set_root | 17 |
| math_FunctionRoot_Test.cxx | math_function_root | 12 |
| math_BFGS_Test.cxx | math_bfgs | 13 |
| math_FRPR_Test.cxx | math_frpr | 14 |
| math_Powell_Test.cxx | math_powell | 11 |
| math_NewtonMinimum_Test.cxx | math_newton_minimum | 15 |
| math_NewtonFunctionRoot_Test.cxx | math_newton_function_root | 16 |
| math_Uzawa_Test.cxx | math_uzawa | 14 |
| math_DirectPolynomialRoots_Test.cxx | math_direct_polynomial_roots | 19 |
| math_TrigonometricFunctionRoots_Test.cxx | math_trigonometric_function_roots | 16 |
| math_FunctionRoots_Test.cxx | math_function_roots | 15 |
| math_FunctionAllRoots_Test.cxx | math_function_all_roots | 16 |
| math_Integration_Test.cxx | math_integration | 16 |
| math_PSO_Test.cxx | math_pso | 13 |
| math_GlobOptMin_Test.cxx | math_glob_opt_min | 16 |
| MathUtils_Bracket_Test.cxx | mathutils_bracket | 16 |
| MathRoot_Test.cxx | mathroot | 51 |
| MathRoot_Comparison_Test.cxx | mathroot_comparison | 15 |
| MathUtils_Functor_Test.cxx | mathutils_functor | 49 |
| MathOpt_1D_Test.cxx | mathopt_1d | 27 |
| MathOpt_1D_Comparison_Test.cxx | mathopt_1d_comparison | 13 |
| MathOpt_FRPR_Test.cxx | mathopt_frpr | 12 |
| MathLin_EigenSearch_Test.cxx | mathlin_eigen_search | 7 |
| MathLin_Comparison_Test.cxx | mathlin_comparison | 12 |
| MathOpt_Newton_Test.cxx | mathopt_newton | 15 |
| MathOpt_NDim_Test.cxx | mathopt_ndim | 25 |
| MathOpt_NDim_Comparison_Test.cxx | mathopt_ndim_comparison | 12 |
| MathLin_Test.cxx | mathlin | 50 |
| MathPoly_Comparison_Test.cxx | mathpoly_comparison | 18 |
| MathPoly_Test.cxx | mathpoly | 47 |
| MathPoly_Laguerre_Test.cxx | mathpoly_laguerre | 20 |
| MathRoot_Trig_Test.cxx | mathroot_trig | 50 |
| MathRoot_Multiple_Test.cxx | mathroot_multiple | 27 |
| MathInteg_Comparison_Test.cxx | mathinteg_comparison | 21 |
| MathInteg_New_Test.cxx | mathinteg_new | 24 |
| MathInteg_Test.cxx | mathinteg | 60 |
| MathSys_Comparison_Test.cxx | mathsys_comparison | 24 |
| CSLib_Test.cxx | cslib | 36 |
| PLib_Test.cxx | plib | 19 |
| PLib_JacobiPolynomial_Test.cxx | plib_jacobi_polynomial | 11 |
| PLib_HermitJacobi_Test.cxx | plib_hermit_jacobi | 9 |
| MathSys_LM_Test.cxx | mathsys_lm | 30 |
| MathSys_Newton2D_Test.cxx | mathsys_newton2d | 12 |
| MathSys_Newton3D_Test.cxx | mathsys_newton3d | 10 |
| MathSys_Newton4D_Test.cxx | mathsys_newton4d | 8 |
| MathOpt_PSO_Test.cxx | mathopt_pso | 28 |
| MathOpt_GlobOpt_Test.cxx | mathopt_global | 24 |
| ElCLib_Test.cxx | elclib | 12 |
| BVH_Box_Test.cxx | bvh_box | 86 |
| BVH_Ray_Test.cxx | bvh_ray | 19 |
| BVH_Tools_Test.cxx | bvh_tools | 45 |
| BVH_RadixSorter_Test.cxx | bvh_radix_sorter | 35 |
| BVH_QuickSorter_Test.cxx | bvh_quick_sorter | 13 |
| BVH_BuildQueue_Test.cxx | bvh_build_queue | 17 |
| BVH_Tree_Test.cxx | bvh_tree | 23 |

Run `cargo test -p rustycad-tkmath --all-features` for Rust tests and
`python3 tools/verify_toolkit.py TKMath --jobs 4` for unchanged native suites,
exact runtime-name checks, and the numeric differential corpus. Evidence is in
`target/oracle-tkmath/`. Reports hash source inputs and libraries and remain scoped
to the mapped suites. Missing, skipped, or extra tests fail verification.

## Numeric comparison policy

The 10454-case corpus exercises coordinate algebra, range operations and periodic
splits, matrix arithmetic and powers, point transformations, and direction
operations, frame direction changes, affinities, planar transforms, line and plane
metrics, bounding-box state, transforms and intersections, and bounding-sphere estimates
and unions, quaternion/Euler conversions and frame displacement, and compact-box operations in both float and double storage. Finite values, infinities, and signed zeros require exact bits.
NaNs require matching classification. Raw native and Rust results are retained.
Five cases currently have NaN representation differences between GCC and Rust:
a zero-axis rotation and degenerate solver cases with negative thresholds; `nan_representation_difference_cases` records this explicitly. Raw NaN
representation parity for these cases is not claimed. The corpus also checks bounded
numeric-table storage, copy/move ownership, external writes, and rebasing, plus
mesh-node precision conversion, rounding, overflow, signed zeros, and resizing,
and bounded-vector norms, arithmetic, matrix products, moves, and checked failures.
Dynamic matrix and LU/Crout probes include custom bounds, singularities, pivot
thresholds, lower-triangle inverses, and RHS/output aliasing. Least-squares and
Householder checks include rectangular matrices, multiple right-hand sides,
oversized buffers, and extraction from a padded matrix.

OCCT disables some out-of-line matrix checks in release builds. Matrix row/column
fallbacks, singular inversion, and zero-axis rotations preserve that release
behavior by default. `occt-debug-checks` enables the corresponding gp_Mat checks;
both Rust feature configurations are tested. A complete native debug-profile
oracle is still pending. Header-level matrix division and coordinate checks remain
active, as in the pinned native build.

The pinned gp_Dir::Mirror(gp_Ax1) reads both its B and C coefficients from Y.
The Rust port reproduces this, including results with non-unit magnitude. A
supplemental regression and differential probes preserve the observable difference
from gp_Dir::Mirror(gp_Dir). Direction JSON parsing and failed cross products also
retain native partial writes without silently renormalizing the result.

The gp_Ax3 negative-scale implementation reverses all three directions through
its gp_Ax1 member, despite the upstream comment saying its normal is unchanged.
Plane scaling preserves this behavior; differential probes compare the complete
frame and oriented equation. Bnd_Box's three-argument IsOut tests a swept strip,
not simply a finite segment. Its projection paths and floating-point operation
order are preserved, as are strict plane-side boundary tests.

Scale construction and inversion retain native zero-scale exceptions. Transform
powers and location operations return `Result` where OCCT can raise an exception.
The compound Power path squares its temporary scale before using it for the
translation update; this is preserved, including a focused numerical regression.

The two-argument gp_Ax3 constructor stores its chosen X direction directly.
The corresponding gp_Ax2 and gp_Pln constructors additionally orthogonalize it.
The port preserves this distinction, which affects the last bit of displacement
matrices even for geometrically equivalent frames.

Bounded-vector norms retain OCCT's four independent accumulators. `Divide`
performs direct division while `Divided` multiplies by a reciprocal. `Invert`
retains the native lower-bound-dependent loop limit: for bounds 5..9 it swaps
only the first and last entries. Empty-vector norms preserve the native failure.

Crout reads only the lower triangle. Its aliased `Solve(B,B)` reuses intermediate
writes, including writes made earlier in the same row; `solve_in_place` retains
that observable behavior. Only the defined lower triangle of its inverse is
compared. Gaussian progress/cancellation support is still pending.

Jacobi preserves descending selection order, including the last-equal-value tie
rule. Tridiagonal QL eigenanalysis retains its native unsorted output order and
floating-point deflation test. SVD supports tall and wide matrices and preserves
stateful singular-value truncation: lowering the cutoff on a later solve does not
restore previously discarded values. Differential checks include full eigenvectors,
pseudoinverses, and repeated solves.

Kronrod generation retains the Jacobi-Kronrod recurrence and sorting of computed
nodes, including the order-125 OCC33048 regression. Thirty-five quadrature orders
have exact native node/weight comparisons.

Scalar solver probes compare callback arguments and outputs, including failure
returns. BracketedRoot ignores the callback boolean, as native OCCT does.
BissecNewton can stop before reevaluating the last trial, preserving cached function
and derivative values. BrentMinimum retains an earlier successful Done flag if a
later Perform returns early on callback failure. These behaviors have explicit
regressions. Minimum bracketing retains supplied-value flags and limit handling.
Newton system probes also compare the in-place LU factors visible to callbacks,
clipped trial points, output copies, and state-save calls. FunctionSetErrors returns
the Newton step, not the residual vector.

Native subclass customization hooks, numerical solver dumps, and some storage
constructors remain pending. NewtonFunctionSetRoot declares StateNumber in the
pinned header but has no implementation in the source tree; no native comparison
for that declaration is claimed. Kronrod orders below one enter undefined native
indexing; Rust rejects them rather than claiming a numerical compatibility result.

## Remaining coverage

Legacy-test coverage does not establish complete API coverage. Transform3 currently
supports identity, translation, rotation, uniform scale, and their compositions.
Mirror transform forms, coordinate-system conversions, the remaining geometry and
numerical packages, and DRAW coverage remain pending. Axis and vector APIs are
partial. Bnd_Range has explicit overload mappings; authoritative toolkit-wide API
enumeration is still required. No TKMath acceptance flag is enabled.

The general function-set solver preserves Newton/gradient direction switching,
rectangular least-squares/SVD fallback, clipping, line searches, callback state,
and native convergence at stationary points with nonzero residuals. Scalar
FunctionRoot delegates to this solver and preserves its final value callback.
The differential probe omits the last-step vector only when native convergence
precedes the first step: OCCT leaves that storage uninitialized. Roots, Jacobians,
status, and callback traces are still compared for those cases. Custom subclass
stopping hooks and diagnostic dumps are not yet mapped.

The four classical optimizers preserve OCCT's callback order, line searches,
mutable caches, state capture, and boundary behavior. The OCC30492 test includes
both BFGS and FRPR. BFGS checks convergence before direction-search success;
FRPR can return a cached direction as its gradient. NewtonMinimum retains Done
when its status reports an iteration limit or a late direction-search error.
The 256 optimization requests compare objective/gradient/Hessian callbacks,
repeated solves, status, guarded results and copies, and exact result bits.
Native output arguments left uninitialized before a callback are not read by
the probe. NewtonMinimum::IsConvex is declared but has no definition in the pinned
OCCT sources; it has not been claimed as a verified callable mapping.

The scalar Newton solver preserves its best-sampled root while retaining the
last callback's value and derivative. Its iteration counter includes the final
increment, and an exact zero derivative remains a failure even at a root.
Bounded, unbounded and repeated runs are compared in the scalar differential
requests. Workspace coverage generation and Windows GNU/macOS ARM cross-checks
passed through this slice; coverage percentages do not establish API parity.

Uzawa preserves the implementation's equality-then-inequality count order,
which differs from the names in its header. Dual variables remain accessible
when Done is false. Its native initial-error allocation uses the column count;
tall inputs write beyond that allocation, so Rust rejects that undefined shape.
Numeric checks compare the initialized residual prefix and the inverse's defined
lower triangle after equality solves; unspecified storage is not claimed as parity.

Direct polynomial roots retain algorithm order and multiplicity, threshold-based
degree reduction, scaling, and the exact nine-step refinement loop. Protected
Solve operations retain prior Done and InfiniteRoots flags. The probe verifies
these repeated-call states and stale, previously initialized roots after overflow.
It avoids reading uninitialized native root counts after fresh overflow failures.

Trigonometric root comparisons cover analytical reductions, periodic windows,
repeated roots, coefficient scaling, Newton refinement, and the distinct floating
point expressions used by separate and combined callbacks. Sampled root checks
retain OCCT's duplicate tolerance, callback order and saved states, exact-zero
neighbor behavior, shifted targets, and failed sampling. Null-interval checks
include both boundary states, multiple intervals, isolated points between intervals,
and native NumericError paths. FunctionSample retains the source's unchecked index
arithmetic; its point count and endpoints are not silently normalized.

Integration probes cover every quadrature order from 1 through 129, native Gauss
and Kronrod table literals, computed fallback nodes and weights, adaptive interval
splitting, error estimates, callback failure, and repeated Perform calls. Particle
swarm probes compare the Bullard generator, grid order, particle positions and
velocities, current and best objectives, ties, ignored callback status, and reuse
of caller-owned pools. Constructor iteration counts and Perform defaults retain
their separate native meanings.

Global optimization comparisons include derivative capability dispatch, continuity
settings, Lipschitz estimation and locking, local-box reduction, retained values
and solution storage after SetGlobalParams, degenerate boxes, and the native
cell-filter branch's duplicate behavior after its threshold. All source files
named `math_*_Test.cxx` now have complete declaration mappings; this does not cover
the separate modern `MathUtils`, `MathRoot`, `MathOpt`, `MathLin`, `MathSys`,
`MathPoly`, and `MathInteg` packages or establish full public API acceptance.

The first modern MathUtils suite adds 16 declarations covering scaled norms,
extended-precision dot products, configuration, domains, convergence, adaptive
derivatives, bracketing, line searches, and polynomial duplicate policy. The
complete numeric corpus passes 10,969 cases, including 515 MathUtils comparisons.
See [the arithmetic profile](mathutils-numerics.md) for the pure Rust x87 adapter
and cross-platform verification limits. Remaining MathUtils functor tests depend
on modern MathRoot and MathOpt solvers and remain outside the mapped count.

Modern scalar root solvers now cover Newton, bounded Newton, Brent, bisection,
hybrid bisection/Newton, secant, and automatic secant initialization. Both root
legacy sources pass: 51 correctness declarations and 15 comparisons with legacy
solvers. The complete numeric corpus passes 11,225 comparisons, including 256
requests covering all seven modern entry points, callback order, optional
result fields, iteration limits, callback refusal, and nonfinite results.


The complete 961-declaration gate passes 12,121 numeric cases. The new functor
coverage exercises scalar algebra, vector objectives, numerical derivatives,
and their solver integration. Modern Powell and BFGS have 128 differential cases;
modern scalar minimizers have 256, including callback traces and partial results.
Finite values, infinities, and signed zeros remain bit-exact on the pinned Linux
oracle profile. Five existing NaN representation differences remain classified
and recorded; no finite discrepancies are accepted.


The 986-declaration gate passes 12,249 numeric cases. Modern FRPR adds 128
requests, each checking all four formulas and an invalid enum value, both with
analytic and numerical gradients. Native restart counters, partial results,
and callback order match exactly.


The 1,005-declaration gate passes 12,761 numeric cases, adding 256 eigenanalysis
and 256 LU requests. This covers natural and legacy tridiagonal layouts, Jacobi
recipe sorting, spectral powers, scaled pivot selection, partial factorization
results, and absent determinant diagnostics on overflow. Eigenanalysis keeps
all 120 upstream random cases as exact libstdc++ inputs; the reproducible native
generator is `tools/oracle/mathlin_eigen_random_inputs.cxx`.

Both probes reject unknown operations. The verifier checks this before running
the corpus, preventing stale binaries from silently testing another operation.

The 1,107-declaration gate passes 13,401 numeric cases. The additions cover all
modern Newton variants, bounded BFGS, limited-memory BFGS, Crout, Householder QR,
SVD, and least squares. Every result field is compared, including failed and
partially computed factors. The 50 declarations in `MathLin_Test.cxx` preserve
native assertion tolerances and the Linux oracle's seed-42 random matrix.
The Rust runtime uses no C dependency to produce that fixture.

Modern SVD reuses the legacy decomposition recipe with the native scale, sort,
and wide-matrix padding rules. Least squares preserves solver selection, rank,
weighted residuals, Tikhonov filtering, and the GCV logarithmic parameter grid.
The remaining modern polynomial, integration, global optimization, nonlinear
system, and multiple-root headers are still outside this completed test scope.

The 1,125-declaration gate passes 13,657 numeric cases. All 18 modern/legacy
polynomial comparison declarations pass unchanged. Linear, quadratic, cubic,
and quartic direct solvers preserve native scaling, compensated FMA
arithmetic, software x87 isolation, and multiplicity classification.

The MathPoly package is now complete for its three sources: MathPoly_Test.cxx
(47 declarations), MathPoly_Laguerre_Test.cxx (20) and the comparison suite.
`laguerre`, `laguerre_n`, `quintic`, `sextic` and `octic` port the checked
Laguerre iteration, deflation, cluster consolidation and long-double Newton
polish in native operation order. 512 native Laguerre requests (degrees up to
20, repeated real and complex roots, clustered roots, global scaling to 1e±300,
zero iteration limits and invalid input) match every root, multiplicity and
status bit for bit, including partial roots kept on failure.

Laguerre results depend on how the pinned GCC 13.3 / glibc 2.39 profile evaluates
`std::complex`. The port reproduces it in software, not with `num-complex`
arithmetic. Full complex products and quotients follow libgcc `__muldc3`/`__divdc3`
(and the x87 `__mulxc3`/`__divxc3` in the multiplicity refinement), including the
GCC 12+ scaled Smith division and infinity recovery. Complex-by-real operations are
componentwise, and a real addend does not touch the imaginary part. `std::sqrt` is
glibc `csqrt`. `std::abs` is glibc `hypot`, and `std::abs` of a long-double complex
is glibc's ldbl-96 `hypotl` (Borges' corrected kernel, not an x87
`fsqrt(x*x+y*y)`). `std::norm` is `x*x + y*y`, as libstdc++ 13 computes it. 512 native
`std::complex<double>` requests cover huge, tiny, subnormal, zero, infinite and NaN
parts. `tools/oracle/extended_complex_fixture.cxx` also records 1,024 bit-exact
`__mulxc3`/`__divxc3`/`hypotl` results as a unit-test fixture.

Deliberate deviations: `laguerre` takes a slice and returns `InvalidInput` when
the slice is shorter than `degree + 1`, where native code reads past the array.
`laguerre_n` uses the slice length as the native `theSize`. Default arguments
are the constants `LAGUERRE_TOLERANCE` and `LAGUERRE_MAX_ITERATIONS`. Root sorts use
the libstdc++ `std::sort` port in `rustycad_tkernel::stl`, so equal-valued roots with
different zero signs also keep the native order for 17 to 20 roots.

### MathPoly::Utils

`MathPoly::Utils` is public as `math_poly::utils`: every helper of MathPoly_Utils.hxx
(`Validate`, `ToDouble`, `ScalePolynomial`, `Evaluate`, `HasSmallResidual`,
`RootBound`, `AppendUnique`, `SolveDistinctReal`, `SolveDirect` and the constants) and
of the Utils namespace in MathPoly_Laguerre.hxx (`IterationResult`, `EvaluateComplex`,
the complex `HasSmallResidual`, `LaguerreIteration`, `FindRoot`, `DeflateReal`,
`DeflateComplex`, `HasMultiplicity`, `HasComplexMultiplicity` and both `Consolidate`
helpers). The direct and Laguerre solvers call these functions; the earlier private
copies are deleted. `long double` is the public `math_poly::LongDouble`, a software
x87 80-bit value (see [the arithmetic profile](mathutils-numerics.md)); its
`from_bits`/`to_bits` expose the exact pattern. 1,400 `mathpoly_utils_long` and
`mathpoly_utils_complex` differential requests print long doubles as 80-bit patterns
and match OCCT exactly.

Mapping rules: native pointers become a slice plus the native `int` degree, output
references become return values (`evaluate` returns the value, derivative and magnitude
triple) or `&mut` parameters, and the `THE_*` long-double constants are functions. A
null pointer is `None` in `validate`. A slice shorter than `degree + 1`, a fifth
distinct root in a four-root buffer, and a negative derivative degree are undefined
behavior natively and panic on a bounds check here. `solve_distinct_real` with a
degree below 1 recurses forever natively; Rust returns false.

The differential probe found one earlier deviation: `LaguerreIteration` records a
candidate only when `|p(z)| <= tolerance * max(1, magnitude)`. The private port tested
the negated `>` comparison, which accepts a NaN tolerance. This did not affect the
solvers, because they reject NaN tolerances first. The public function now follows the
native comparison, and a regression test pins it.

`num-complex` 0.4.6 is isolated behind `math_poly::Complex` and supplies only
storage. Version reference: https://docs.rs/num-complex/0.4.6/num_complex/.

When 1,419 declarations were mapped, the gate passed 16,599 numeric cases. All 50 trigonometric
root declarations pass in both implementations. Differential cases compare
native statuses, all four root slots, the infinite-roots flag, and periodic
count outputs on success and failure. The periodic count uses software x87
arithmetic for the pinned Linux/GCC profile. The native Halley fallback formula,
including its `-4*b*cos(x)*sin(x)` term, is preserved.

### Multiple-root callback compatibility

All 27 multiple-root declarations pass against OCCT. The 256 differential cases
compare roots, failure prefixes, null intervals and complete callback traces.
Golden-section refinement preserves callback output when a callback leaves it
untouched; a recorded 428-event native trace guards that behavior.

### Gauss integration

All 21 Gauss comparison declarations pass. Fixed, composite and adaptive rules
retain native failure fields, work accounting and interval orientation. Another
256 numeric cases compare complete callback traces and modern eigen-generated
quadrature tables, including orders above 61.

### Complete integration declaration suites

All 105 declarations across the three MathInteg suites pass in both kernels.
The port includes Gauss, Gauss–Kronrod, double-exponential, tensor-product and
vector-valued rules. The 1,024 integration differential requests compare full
result fields and callback traces, including callback failures, non-finite
outputs, partial output writes, work limits, infinite domains and singularity
splits. Tensor orders use the existing bounded `Array1<i32>` storage; this does
not imply that the separate `math_IntegerVector` API inventory is complete.

Workspace tests, strict linting, Windows GNU and macOS ARM compilation, Python
verifier tests and coverage generation pass at this checkpoint.

### Generic nonlinear Newton systems

All 24 MathSys comparison declarations pass. The implementation preserves
column-equilibrated rank classification, LU/QR/SVD selection, projected active
bounds, software x87 merit comparisons and finalization callbacks. Another 256
differential requests compare every result field and the full callback trace.

### Levenberg–Marquardt systems

All 30 LM declarations pass in both kernels. The port retains augmented QR/SVD
steps, damping updates, active-bound stationarity and final result fields.
The 256 LM differential requests match complete results and callback traces.

### Specialized Newton systems

All 30 declarations for general 2D/3D/4D Newton, symmetric 2D Newton and the
curve/surface and surface/surface extrema adapters pass in Rust. Another 512
native differential requests match results and callback traces exactly.
Software x87 square root matches 2,048 native significand/exponent fixtures,
including subnormals. The combined toolkit gate passed all 1,419 declarations mapped at that time.

### Particle swarm optimization

All 28 PSO declarations pass in Rust, including the old/new API comparison,
seed fitness re-evaluation, restart counters and budget exhaustion. The 382
native differential requests match result fields, statistics and complete
callback traces. They include clamp, reflect and wrap boundary policies,
seeded and random initialization, callback failures and invalid seeds.

### CSLib surface normals and 2D classification

All 36 CSLib declarations pass in both kernels, including the eight-thread lazy
grid activation test, which uses scoped `std::thread`s over one shared
classifier. The 2,528 CSLib differential requests (19,383 in the whole corpus)
match exactly. They cover all normal overloads and statuses, singular points up
to degree 26, normal derivatives, `NormalPolyDef`, and `Class2d` across
containers, extreme bounds, special tolerances, the grid, copies and moves.

C++ output references become returned values. `normal` is `None` exactly when
OCCT leaves its `gp_Dir&` untouched. Singular-point orders are always returned,
as OCCT always writes them. The lazy grid keeps OCCT's atomic state machine. The
cells are published through a `OnceLock` before the release store of `Ready`.
Allocation failure uses `try_reserve` and resets to `NotBuilt`, as natively.

The singular-point sign analysis keeps two native quirks. A first retained
interval at index 0 leaves `aFirst` at 0. The next interval then restarts the
comparison, which can hide a sign change. `NormalPolyDef` returns 0 when
|cos X| or |sin X| <= RealSmall. A midpoint sample at 0 or ±π/2 is therefore
0, which hides a sign change. When 0 is the final sample, OCCT picks -Vk0.
`cslib_behavior.rs` pins both cases; for example, F = cos X gives `Defined`
with -Vk0 at an interior point.

There are two deliberate deviations. `NormalPolyDef::new` raises OCCT's
`PLib::Bin` Standard_OutOfRange for degrees above 25 at construction, because
the `bool` evaluation callbacks cannot carry an exception. OCCT raises it on the
first evaluation that is not at a singular angle, so the corpus uses regular
angles only for degree 26. Degrees below -1 fail with Standard_RangeError. OCCT
passes them to an allocator-dependent NCollection_Array1 size. `PLib::Bin` and
the Precision constants are small private copies (`cslib/binomial.rs`,
`cslib/precision.rs`). They must be replaced by the PLib and Precision ports
when those merge.

### PLib polynomial library

All 39 declarations of the three PLib suites pass in both kernels. The port
covers the complete public API of PLib, PLib_JacobiPolynomial and
PLib_HermitJacobi, and the whole GeomAbs enumeration package. Another 2,240
differential requests match with raw bits, including NaN payloads; the numeric
gate now passes 22,647 cases.

`tools/generate_plib_jacobi_tables.py` copies every Jacobi table literal from
the pinned `.pxx` files as text, so both compilers round the same decimal
strings. It proves that each per-degree coefficient array is a prefix of the
degree-30 array and then keeps only that array. `--check` detects drift.

Several native behaviors are preserved. A derivative request of 0 takes the
general Horner branch, which starts from `0 * U + c`; an infinite U therefore
gives NaN there but not in NoDerivativeEvalPolynomial. The unrolled
Dimension == 3 loops of RationalDerivative(s) perform the same operations as
the general loop, so one loop reproduces both. AverageError sums only up to the
Q degree, not WorkDegree. A singular HermiteInterpolate system raises
StdFail_NotDone from the solve before `false` can be returned; with no
components it still reports success. JacobiParameters writes WorkDegree before
its MaxDegree and Code checks can raise. Weights accepts non-tabulated Gauss
counts and reads the native table layout, and the gp_Pnt CoefficientsPoles
overload seeds its pole temporary from the coefficients.

Native `double&` buffers become slices. Where OCCT would access outside such a
buffer or a table, or where a negative degree, dimension or derivative count
makes its pointer arithmetic undefined, Rust returns Standard_OutOfRange with
the offending value. The point CoefficientsPoles overloads reject pole arrays
longer than the coefficients, which natively read uninitialized storage.
Weighted input without an output weight array (a native null dereference) is
an error. Native exceptions keep their types.

### ElCLib, ElSLib and the gp conics and quadrics

All 12 ElCLib declarations pass in both kernels. ElSLib has no upstream suite, and
gp_Torus_Test needs Geom_ToroidalSurface from TKG3d, so it is not mapped; supplemental
tests cover ElSLib, gp_Elips/Hypr/Parab, their 2D counterparts, gp_Ax22d and the
quadrics. The 5,200 elementary differential requests (25,607 in the whole corpus)
match exactly. They cover every ElCLib and ElSLib static function, the gp-object
overloads, To3d, and all members of the new gp types, including degenerate frames,
invalid radii, the parabola resolution boundary, negative derivative orders, centre
and axis points, infinite periods and zero scale factors.

Every static function is a free function in `elclib`/`elslib`. Overloads that differ
by dimension take a `_2d` suffix, `To3d` overloads are `to3d_<kind>`, and the
gp-object overloads are generic `value`/`d1`/`d2`/`d3`/`dn`/`parameter(s)` functions
over the `ElementaryCurve` and `ElementarySurface` traits. `asinh` comes from the
platform libm through the `rustycad-cmath` adapter crate, because Rust's `f64::asinh` uses a different formula.

The port keeps these native results:

- `IsOdd` is `n % 2 == 1`, so a negative odd order is neither odd nor even. For
  such an order, `HyperbolaDN` returns a zero vector. The differential probe found
  this, and a regression test pins it.
- Planar D2/D3 reverse the summed vector, but 3D sums negated terms. The planar
  parabola tangent is `u·(X/(2F)) + Y`; the 3D tangent is `(u/(2F))·X + Y`. ElSLib
  D3 negates `A2·X - A1·Y`, but D1/D2 use `-A2·X + A1·Y`. Each pair differs in
  rounding or in the sign of a zero, so each keeps its own form.
- ElSLib.cxx defines `No_Exception`, so the inline gp checks it uses are compiled
  out. Iso-circles accept a negative radius, directions never raise, and all ElSLib
  functions are infallible. `SphereD3` returns `Vuvv` equal to `Vuuu`, and
  `ConeDN(0,0)` omits the axial `v·cos(a)·Z` term.
- `CircleParameter` returns 0 in 3D for the centre, but raises in 2D. There, the
  offset becomes a `gp_Dir2d`. The 2D ellipse raises gp_VectorWithNullMagnitude.
- `gp_Hypr2d::Asymptote1` normalizes `Y + Y·(b/a)`, so it returns Y itself.
  `Asymptote2` builds `Y·(1 - b/a)` in a gp_Vec2d and raises for equal radii.
  `Directrix2` offsets `Focus1` by `p/e` and does not mirror `Directrix1`. Both
  3D asymptote errors use the "Asymptote1" message.
- In-place `gp_Elips2d::Transform` and `gp_Torus::Scale`/`Transform` multiply by
  `|factor|`, so zero radii stay `+0.0`. The copying variants give `-0.0`.
- `gp_Ax3::Translate(P1,P2)` adds `gp_Vec(P1,P2)`. `gp_Ax2`, like gp_Pnt, adds the
  destination and then subtracts the origin. `gp_Ax2`/`gp_Ax3::Mirror(gp_Ax1)` mirror
  X/Y with the Y-for-Z gp_Dir overload. gp_Ax3 mirrors its normal with the gp_Dir
  overload, so its frame can become non-orthogonal.

gp_Ax22d constructors renormalize the quarter-turned direction through
`gp_Dir2d::SetCoord`. The native zero-norm raise there is unreachable, because a
stored direction is never zero. Exception messages add the offending values to the
native text. The planar conics' `Transform` members accept every Transform2 form.
Transform2 now builds every gp_Trsf2d form, and the point-mirror and scale branches of
`gp_Dir2d::Transform` are covered by the planar gp probe (see below).

### BVH core (phase 1)

All 238 declarations of the seven BVH core suites pass in both kernels. The
1,331 BVH differential requests (21,738 in the whole corpus) match exactly. They
cover boxes, rays, tools, JSON, quick and radix sorts, binned builds, SAH, QBVH
collapse and 4D transforms, in float and double for N = 2, 3 and 4. See
[the BVH design note](bvh-design.md) for the mapping of the templates to Rust.

The port keeps these native behaviors:

- Float instantiations compute in single precision. `BVH_Transform::Apply` still
  scales by `(T)(1.0 / w)`, which is computed in double.
- `BVH_Transform::Apply` offsets only X, Y and Z of the minimum corner, so every
  transformed corner keeps the minimum W. A zero W gives infinite coordinates.
- `BVH::IntFloor` follows the x86-64 truncation: NaN and out-of-range values give
  `INT_MIN`, and -inf wraps to `INT_MAX`. The float correction compares in single
  precision.
- Vec3/Vec4 division by a scalar multiplies by the reciprocal, and Vec2 divides.
  `BoxMinMax` uses `std::min`/`std::max`, while projections use `cwiseMin`/`cwiseMax`.
  The two resolve signed-zero ties differently.
- The quick and radix sorters apply the element order of libstdc++ `std::sort`
  and `std::partition`, so tied centroids end in the same order as in GCC.
- A new primitive set is not dirty. It has an invalid box and an empty tree until
  `MarkDirty`.
- `BVH_Transform(matrix)` keeps an identity inverse. A singular `SetTransform`
  leaves the unscaled cofactors, as `Inverted` writes them before it checks the
  determinant.
- `InitFromJson` keeps a parsed `MinPoint` when `MaxPoint` fails, and it widens
  the fourth dimension to `[lowest, max]`. `DumpJson` keys follow the macro text,
  for example `BegPrimitive(theNodeIndex)`.
- The binned builder splits coincident centroids at `(beg + end) / 2.f`, which is
  computed in float.

Deliberate Rust deviations: accessors that raise `Standard_OutOfRange` return
`Failure` errors with the index and the valid range. The quick sorter reads each
centroid once before it sorts; `Center` is a const query. Multi-threaded queue
builds serialize node splits behind a mutex, and their node numbering depends on
the schedule, as upstream. A failing build tool releases its busy slot. Upstream
tests that check constructors with `static_assert` use `const` items, and the
validity check runs at test time. Uninitialized ray-hit locals start as NaN.


### Planar gp algebra and transformations (API completion)

This work adds no upstream suite. It completes the public API of gp_XY, gp_Mat2d
(`Matrix2`), gp_Trsf2d (`Transform2`), gp_GTrsf2d (`GeneralTransform2`), gp_Dir2d,
gp_Vec2d, gp_Pnt2d and gp_Lin2d, and adds gp_Dir2d::D (`StandardDirection2`) with the
gp_Ax2d constructors that use it. `porting/parity.json` maps every public member.
Supplemental tests are in `gp_planar_behavior`, `gp_planar_primitive_behavior` and
`gp_trsf2d_behavior`. The planar gp probe sends 4,316 differential requests
(`gp2d_xy`, `gp2d_mat`, `gp2d_trsf`, `gp2d_gtrsf`, `gp2d_geom`), and all match the
native bits.

The port keeps these native results:

- `gp_Mat2d::Multiply` stores the new `[0][1]` before it reads the operand's `[0][1]`
  to compute `[1][1]`. `gp_Mat2d::Power`, `gp_Trsf2d::Power` and `gp_GTrsf2d::Power`
  square a temporary with `M.Multiply(M)`, so every power of 3 or more, or of -3 or
  less, mixes the new and the old coefficient. For example, a 0.3 rad rotation cubed
  is not the 0.9 rad rotation. `Matrix2::square_in_place_aliased` reproduces this. A
  Rust `multiply` receives a copy, so a user self-product has no such effect.
- The gp_Mat2d out-of-line index and determinant checks exist only with OCCT_DEBUG.
  A release build reads and writes column or row 2 for every index other than 1, and
  inverts a singular matrix to infinities. `occt-debug-checks` enables the raises.
- `gp_Trsf2d::IsNegative` tests the unscaled matrix, so a point mirror (identity
  matrix, scale -1) is not negative. `VectorialPart` scales only the diagonal of the
  scale and point-mirror forms. `PreMultiply` by a translation keeps the Rotation form.
- `gp_Trsf2d(const gp_Trsf&)` copies the 3D form, including forms that have no planar
  meaning. `SetValues` raises only for `|det| < Resolution`. If orthogonalization then
  raises, the scale, the Compound form and the divided matrix stay assigned.
- `gp_GTrsf2d::Power(n)` with a negative `n` and a similarity form inverts, then calls
  `gp_Trsf2d::Power(n)`, which inverts again. The result is `T^|n|` up to rounding.
- A gp_Dir2d built from components whose squared norm overflows is stored as a zero
  vector. `gp_Lin2d::Normal` is `noexcept`, so natively it calls `std::terminate` for
  such a direction. Rust returns the ConstructionError, and the corpus avoids that input.
- `gp_Vec2d::GetNormal` multiplies X by -1 and keeps the sign of a NaN. The copying
  `gp_Lin2d::Translated(P1,P2)` adds `P2 - P1`, but the in-place `Translate` adds `P2`
  and then subtracts `P1`.

Deliberate Rust deviations: gp_Trsf2d setters that replace the whole state are
constructors (`mirror_point`, `scaling`, `transformation_between`, ...) plus `set_*`
methods that call them. `gp_GTrsf2d::operator*` returns `Result`, because it raises
for a non-orthogonal similarity pair. gp_Mat2d has no `operator*=(gp_Mat2d)`, so
`Matrix2` has no `MulAssign<Matrix2>`.

### gp_Dir::D and the gp statics

gp_Dir::D is `StandardDirection3`. `Direction3::from_standard` and the gp_Ax1, gp_Ax2
and gp_Ax3 constructors that take it (`with_standard`, `from_standard` and
`with_standard_frame`) build exact axis-aligned frames. The gp class statics
(`gp::Resolution`, `gp::Origin`, `gp::DX` ... `gp::OY2d`) are the free functions
`gp::resolution()` ... `gp::oy2d()`, which return copies. 36 `gp_standard`
differential requests cover every main/X pair and the statics.

The port keeps these native results:

- The two-direction constructors take Y from the `crossStandardDir` table and do not
  check orthogonality. For a parallel or opposite pair the table returns a fixed
  fallback (for example `Z` for `(X, X)`), so the frame is not orthogonal.
- `gp::ZOX()` is `gp_Ax2(origin, D::Y)`, so its X direction is Z and its Y direction
  is X.
