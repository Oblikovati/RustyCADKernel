# TKMath port status

**In progress; toolkit acceptance is not complete.** The source inventory contains
120 GoogleTest files and 2023 declaration candidates. 114 complete source suites
currently account for 2,002 mapped declarations, with matching native and Rust passes.

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
| BVH_BinnedBuilder_Test.cxx | bvh_binned_builder | 14 |
| BVH_Triangulation_Test.cxx | bvh_triangulation | 16 |
| BVH_LinearBuilder_Test.cxx | bvh_linear_builder | 15 |
| BVH_SpatialMedianBuilder_Test.cxx | bvh_spatial_median_builder | 15 |
| BVH_SweepPlaneBuilder_Test.cxx | bvh_sweep_plane_builder | 10 |
| BVH_Traverse_Test.cxx | bvh_traverse | 21 |
| BSplCLib_Test.cxx | bsplclib | 5 |
| BSplCLib_Cache_Test.cxx | bsplclib_cache | 7 |
| BSplSLib_Test.cxx | bsplslib | 5 |
| BSplSLib_Cache_Test.cxx | bsplslib_cache | 8 |
| Convert_CircleToBSplineCurve_Test.cxx | convert_circle_to_bspline_curve | 6 |
| Convert_EllipseToBSplineCurve_Test.cxx | convert_ellipse_to_bspline_curve | 4 |
| Convert_HyperbolaToBSplineCurve_Test.cxx | convert_hyperbola_to_bspline_curve | 3 |
| Convert_ParabolaToBSplineCurve_Test.cxx | convert_parabola_to_bspline_curve | 3 |
| Convert_CompBezierCurvesToBSplineCurve_Test.cxx | convert_comp_bezier_curves_to_bspline_curve | 7 |
| Convert_CompPolynomialToPoles_Test.cxx | convert_comp_polynomial_to_poles | 4 |
| Convert_GridPolynomialToPoles_Test.cxx | convert_grid_polynomial_to_poles | 2 |
| Convert_SphereToBSplineSurface_Test.cxx | convert_sphere_to_bspline_surface | 6 |
| Convert_TorusToBSplineSurface_Test.cxx | convert_torus_to_bspline_surface | 7 |
| Convert_CylinderToBSplineSurface_Test.cxx | convert_cylinder_to_bspline_surface | 5 |
| Convert_ConeToBSplineSurface_Test.cxx | convert_cone_to_bspline_surface | 4 |

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

### BVH traversal, builders and distance field (phase 2)

All 91 declarations of the six phase 2 suites (binned builder, triangulation,
linear, spatial-median and sweep-plane builders, traversal) pass in both kernels.
`BVH_ShapeSelect_Test`, `BVH_PairDistance_Test` and `BVH_DistanceField_Test` are
not mapped, because they mesh shapes with BRepPrimAPI and BRepMesh. Their BVH
classes are ported. The 912 phase 2 differential requests (30,090 in the whole
corpus) match exactly. They compare whole tree layouts of every builder, the
element order, and the logs of each `RejectNode` metric and `Accept` call of
single and pair traversals. They also compare `BVH_Distance`, `BVH_PairDistance`
and complete distance-field voxel grids, in float and double.

The port keeps these native behaviors:

- `BVH_LinearBuilder` returns before `Clear()` for an empty set, so the old tree
  stays. It does not apply the depth limit, leaves every node level at 0 and sets
  the depth to the tree height. The spatial-median builder is the binned builder
  with two bins.
- The single traversal never tests the root with `RejectNode`. When both children
  pass, it stacks the worse child. The pair traversal tests two leaves before
  their element pairs. It sorts the child pairs with a stable insertion by
  `IsMetricBetter`. Stack overflow raises `Standard_ProgramError`, also in release
  builds.
- The sweep-plane builder sorts the node again unless the best axis is the last
  one it tried. So the result can depend on the sort of an already sorted range.
  It does not split a node that is at most `THE_NODE_MIN_SIZE` thick on all axes.
- `BVH_Geometry` starts clean, with its own dirty flag: until `MarkDirty`, its box
  is invalid and its tree empty. A distance field of such a geometry has infinite
  corners, and every voxel is the root of the scalar maximum.
- `BVH::DirectionToNearestPoint` accepts zero dot products on the edges (`>=`),
  unlike `BVH_Tools`. The interior case multiplies by a reciprocal norm.
  `SquareDistanceToObject` gives the scalar maximum for an object that is not a
  triangulation. `BVH_BoxSet::Center` is the box center.
- The distance field sizes are truncated through `static_cast<int>`. A NaN
  becomes `INT_MIN` and then 16. Every margin uses the division order
  `2 * side / (dimension - 4)`.

Deliberate Rust deviations: set-based `Select()` moves the tree out of its set
for the traversal and moves it back, because `Accept` reads the same set. So a
set cannot be paired with itself through the set-based path. Use
`select_trees(Some(&tree), Some(&tree))` for that. `Accept` returns a
`Result`. Object sets own their objects (`Option<Box<dyn BvhObject>>`, with `None`
as a null handle), and a null handle reports `Standard_NullObject` where upstream
crashes. `BvhObject::triangulation_mut` stands in for the `dynamic_cast`. A leaf
size below 1 that makes the LBVH middle split recurse forever fails with
`Standard_StackOverflow`. The distance field zero-fills its voxels, rejects
negative sizes as `std::bad_array_new_length` and checks voxel offsets. Parallel
LBVH bounds and parallel distance-field slices run in order. Each task writes
only its own nodes or voxels, so the results are the same. Upstream even races on
lazy tree builds in parallel slices.


### BSplCLib B-spline curve library

The whole public API of BSplCLib, BSplCLib_Cache, BSplCLib_CacheParams and
BSplCLib_EvaluatorFunction is ported (185 API rows). The 12 declarations of
BSplCLib_Test and BSplCLib_Cache_Test pass in both kernels. Another 8,192
differential requests match with raw bits; the numeric gate now passes 30,839
cases. They cover knot management (Hunt, LocateParameter overloads, flat
sequences, forms, reparametrization, reversal), the Boor, anti-Boor, derivative
and Bohm schemes for dimensions 1 to 6, local knot and pole extraction, insertion,
removal, degree elevation, unperiodization and trimming on flat, 2D and 3D poles
(rational and periodic included), D0 to DN in 1D, 2D and 3D up to degree 25,
flat-knot Eval, the basis, the banded LU and interpolation, both BuildCache forms,
CacheD0 to CacheD3, the Bezier syntaxes, BSplCLib_Cache, MovePoint(AndTangent),
Resolution, FunctionMultiply/Reparameterise, TangExtendToConstraint,
MergeBSplineKnots and Intervals.

The native dimension 1 to 4 specialisations of Eval, Bohm and Resolution perform
the same per-component operations as the general loops, so one loop reproduces
them. One exception is kept: Bohm divides by the knot difference in dimension 1
and multiplies by its reciprocal elsewhere, which rounds differently and gives
+0.0 instead of -0.0 for coincident knots. Other preserved quirks: InsertKnots
writes the first new pole of each scheme at `NewKnots.Lower() + (index+1)*Dimension`;
IncreaseDegree reads its working poles from `NewPoles.Lower()`; the rational
flat-knot Eval clears `WeightsResults[row*ArrayDimension]` and accumulates into
`WeightsResults[row]`, so weight derivatives add to the caller's values;
EvalBsplineBasis locates a periodic span but evaluates at the unwrapped
parameter; MovePointAndTangent compares U with flat knots at Schoenberg indices; CacheD2
clears the missing derivative rows after scaling, CacheD3 before; the singular
gp_Mat2d inversion in MovePointAndTangent is unchecked, as in the release build.
`bsplclib_behavior.rs` pins them with regression tests.

Deviations are limited to undefined native behaviour. The flat-knot Eval with
extrapolation evaluates a Taylor polynomial of `Degree` from a buffer of
`LocalRequest+1` terms, reading uninitialized stack; the port reads zeros, the
intended lower-degree expansion (stale pole terms read as weight terms in the
rational variant are kept). D1 on degree 0 reads an uninitialized derivative;
the port returns zero. FunctionMultiply/Reparameterise pass one `int` where Eval
reads two extrapolation modes; the port uses the B-spline degree for both.
BuildBSpMatrix rejects contact orders outside 0..Degree, whose rows natively come
from an unwritten buffer. Raw reads and writes outside a buffer, knot array or
view return Standard_OutOfRange naming the index: for example an evaluation hint
equal to the last u-knot of a non-periodic curve, which LocateParameter keeps and
BuildKnots then walks past the multiplicities, or flat knots of an unclamped curve
whose span search runs through equal knots past the last span. A periodic Eval
whose period cannot advance fails instead of looping forever. The corpus
never enters these paths. `BSplCLib::DN(U, N, Poles, Weights, P, VN)` for Bezier
poles is declared without a definition in the pinned sources and is not ported.

`NCollection_Array1::from_shared` and `Array2::from_shared` add the native
read-only buffer constructor, needed by UnitWeights: the view does not own its
storage and detaches a private copy on the first write.

### BSplSLib B-spline surface library

The whole public API of BSplSLib, BSplSLib_Cache and BSplSLib_EvaluatorFunction
is ported (48 API rows) on top of BSplCLib, PLib and math. The 13 declarations of
BSplSLib_Test and BSplSLib_Cache_Test pass in both kernels. Another 3,840
differential requests match with raw bits; the numeric gate now passes 34,679
cases. They cover D0 to D3, DN and HomogeneousD0/D1 up to degree 25 (and the
refused degree 26) with multiplicities or flat knots, index hints, periodic,
rational, partly rational and locally polynomial patches and non-1 grid bounds;
RationalDerivative on raw buffers; Iso, IsRational and both Reverse overloads;
SetPoles/GetPoles in both directions; InsertKnots, RemoveKnot, IncreaseDegree and
Unperiodize along U and V; both BuildCache forms, CacheD0 to CacheD2,
PolesCoefficients and CoefsD0 to CoefsD2; BSplSLib_Cache with its local
evaluations; MovePoint, Resolution, both Interpolate overloads and
FunctionMultiply (including failing evaluations). Native argument groups become
`SurfaceSpan`, `CacheSource`, `SurfaceCacheSpan`, `ResolutionDirection`,
`InterpolationGrid`, `MultiplySource` and `InterpolationKnots`; output references
become returned values.

Preserved quirks: IsRational takes its default tolerance from
`Epsilon(Weights(I1, I2))`, reading column I2; CacheD0 to CacheD2 read the cache
from `Poles(LowerCol, LowerRow)` and `Weights(LowerCol, LowerRow)`, with swapped
bounds that coincide for 1-based grids; the rational CacheD2 with a maximal
degree below 2 reads the missing weight derivative rows from the evaluation
buffer where pole rows were left; CoefsD0 to CoefsD2 pass `RowLength() - 1` as
the U degree and `ColLength() - 1` as the V degree; the V pass of the rational
Resolution takes the neighbour row from the U range and the column from the V
range; PrepareEval uses an index hint whenever it lies inside the knot array,
without locating or normalizing the parameter, and only its V-first
non-rational pole copy wraps a start index past the last row; the second copy
loop of Reverse starts one row or column after the wrapped `Last`; SetPoles with weights reads
and stores the weight first; FunctionMultiply always evaluates the surface as
rational and non-periodic, so it needs weights; BSplSLib_Cache parametrizes a
span by its middle and half length, and its local evaluations divide by the half
spans. `bsplslib_behavior.rs` pins the main ones with regression tests.

Deviations are limited to undefined or unchecked native behaviour. Rational
evaluations, Iso and FunctionMultiply without the weights they dereference are
errors. Degrees above MaxDegree raise Standard_OutOfRange as natively. A
non-periodic direction whose FirstUKnotIndex is not below its LastUKnotIndex
(unclamped ends at a high degree) has no valid span: the native local knot walk
leaves entries of its stack buffer unwritten, and the port reads zeros; the
corpus draws only directions with a valid span. In the reference build the
transposition in PolesCoefficients (fewer U than V poles) writes `CachePoles`
without the NCollection bound check, so a `[max][min]` target is filled through
row-major offsets; the port raises the checked Standard_OutOfRange, and the
coefficients take the shape of the poles in the corpus. UnitWeights rejects
counts whose product is negative.


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

### Spatial gp transformations and quaternions (API completion)

This work adds no upstream suite. It completes the public API of gp_XYZ (`Xyz`),
gp_Mat (`Matrix3`), gp_Trsf (`Transform3`), gp_GTrsf (`GeneralTransform3`),
gp_Quaternion, gp_QuaternionNLerp and gp_QuaternionSLerp. `porting/parity.json` maps
every public member. Supplemental tests are in `gp_trsf3d_behavior` and
`gp_quaternion_api_behavior`. The spatial gp probe sends 3,100 differential requests
(`gp3d_trsf`, `gp3d_gtrsf`, `gp3d_quat`). They cover every gp_Trsf form, the
ConstructionError and OutOfRange raises, and all match the native bits.

The port keeps these native results:

- `gp_Trsf::IsNegative` tests the sign of the scale factor. `gp_Trsf2d::IsNegative`
  tests the determinant of the unscaled matrix. Thus a 3D point mirror is negative and
  a planar point mirror is not.
- `SetMirror(gp_Ax1)` stores the half-turn about the axis with scale 1 and form
  Ax1Mirror. `gp_Trsf(const gp_Trsf2d&)` converts a planar axis mirror (scale -1) to
  this form by negating the matrix, so its zero entries become -0.
- `SetScale` writes the form, the scale and the offset before it raises for
  `|s| <= Resolution`. `SetValues` raises only for `|det| < Resolution`; if the
  orthogonalization then raises, the scale, the Compound form and the divided matrix
  stay assigned.
- `SetTranslationPart` treats a squared length below Resolution as null.
  `SetScaleFactor` and `SetRotationPart` re-derive the form with the native switch
  tables, for example a scale of -1 on an identity gives PntMirror.
- `InitFromJson` applies the parsed `Location` through `SetTranslation` before it reads
  the matrix, so a later failure leaves a translation form. Native code stores a
  `shape` outside gp_TrsfForm as-is; Rust returns an OutOfRange error.
- `gp_GTrsf::Power(n)` does not invert first for `n < -1`. A general (Other) form
  returns the power of `|n|`. A similarity form calls `gp_Trsf::Power(n)`, which
  inverts. `gp_GTrsf::Multiply` and `PreMultiply` never raise, unlike
  `gp_GTrsf2d::Multiply`.
- `gp_GTrsf::SetForm` divides by the signed cube root of the determinant and compares
  `N^T N - I` with the literal 1e-12. It does not change the scale, so a scaled
  orthogonal matrix becomes Compound with scale 1.
- `gp_GTrsf::SetValue` folds the scale into the matrix before it writes a vectorial
  cell and sets the form to Other; a write to column 4 keeps the form.
- `gp_QuaternionSLerp::InitFromUnit` clamps the cosine to 0.9999, so equal ends still
  have a finite `1 / sin(omega)`. `gp_Quaternion::Inverted` has no zero check.

Deliberate Rust deviations: `gp_Trsf::DumpJson` has no depth argument (the native one
is not used). The `const gp_Trsf&` and `const gp_GTrsf&` operands of the Rust
`multiply` and `pre_multiply` are copies, so a self-product has no aliasing effect.
gp_XYZ, gp_Mat and gp_Quaternion operators are the `std::ops` traits.

### Convert conics, quadrics and polynomial pieces to B-splines

Every class of the Convert package is ported (92 API rows, plus the two-frame
`gp_Trsf::SetTransformation` and `gp_Trsf2d::SetTransformation` overloads the
conversions need): the conic root class with the circle, ellipse, hyperbola and
parabola conversions, both public BuildCosAndSin overloads and
BuildPolynomialCosAndSin, the elementary surface root class with the sphere, torus,
cylinder and cone conversions, the composite Bezier template in 2D and 3D, and the
composite polynomial curve and grid conversions. The 51 declarations of the 11
Convert suites pass in both kernels. Another 3,595 differential requests match
with raw bits; the numeric gate now passes 45,717 cases. They cover every
Convert_ParameterisationType on full, trimmed, tiny, empty, negative and oversized
arcs, in canonical, rotated, offset and indirect frames; hyperbola and parabola arcs
with reversed and degenerate bounds and zero radii; whole, patch and band surfaces
with direct and indirect frames, boundary openings and reversed ranges; Bezier chains
with C1, antiparallel, C0 and coincident junctions, mixed degrees, several angular
tolerances and non-1 lower bounds; composite polynomials with every constructor,
per-span coefficient counts, continuities up to the degree and reversed domains;
single and gridded polynomial patches; and the argument errors of each class.

The OCCT subclasses only add constructors, so they become named constructors of
`ConicToBSplineCurve` (`from_circle`, `from_circle_arc`, ...) and
`ElementarySurfaceToBSplineSurface` (`from_sphere`, `from_sphere_patch`,
`from_sphere_band`, ...). The composite Bezier template is
`CompBezierCurvesToBSplineCurveBase<P>` with the two native aliases. Handle outputs
become returned values or `Option` copies, and the long argument lists of the
polynomial conversions become `PolynomialSpans` and `PolynomialGrid`.

Preserved quirks: the sphere band trimmed in U (`UTrim = true`) weights its meridian
middles with 0.5 although its two V spans open by PI / 4, so the result is not an
exact sphere; the periodic TgtThetaOver2 circle drops the closing pole of three spans
and gives every knot the multiplicity 2; for every negative QuasiAngular opening
`alpha / 2 < 1e-7` holds, so the Taylor value `b = -6/15` is used; the TgtThetaOver2
span count is `(int)std::trunc(1.2 * delta / PI) + 1` (INT_MIN for out-of-range values,
as `cvttsd2si`); the sphere patch accepts `V1 > V2`, where no span gives a single V
pole, a NaN knot and a multiplicity of 4, and fewer than no span raises
Standard_OutOfRange from `myVMults(0)`; a periodic cylinder or cone computes seven
pole rows and keeps six; the composite Bezier conversion assigns a curve that is
already at the maximal degree to its work array, and `NCollection_Array1::Assign`
copies the bounds, so a curve not based at 1 shifts the poles read after it and
fails with Standard_OutOfRange when it is the last curve; an empty sequence keeps
the previous degree; only the 3D instantiation guards the knot ratio against
`10 * Epsilon(Det)`; `GridPolynomialToPoles` knots keep the bounds of the true
breakpoints, while the breakpoints and domains are read from index 1. The ignored
pivot index of AlgorithmicCosAndSin and the unreachable inversion check of
CompPolynomialToPoles are kept (BSplCLib::Interpolate raises first).
`convert_behavior.rs` pins them with regression tests.

Deviations are limited to undefined native behaviour. A sphere band periodic in U
whose V opening is at least 5 PI / 3 makes the native ComputePoles write a seventh
meridian pole into a five-entry `std::array` (the oracle aborts with "stack smashing
detected"); the port raises Standard_OutOfRange before the write. KnotSequence reads
multiplicities through a raw pointer with the knot index, so gridded true breakpoints
not based at 1 read past the array; the port raises Standard_OutOfRange. A
coefficient array shorter than the spans read through `(double*)&Coefficients(i)` is
refused the same way. An arc whose span count turns negative makes the native code
allocate a wrapped size; the port raises Standard_OutOfMemory. The corpus never
enters these paths.

### Core gp primitives, frames and interpolators (API completion)

This work adds no upstream suite. It completes the public API of gp_Vec (`Vector3`),
gp_Dir (`Direction3`), gp_Pnt (`Point3`), gp_Lin (`Line3`), gp_Pln (`Plane3`), gp_Ax2,
gp_Ax3, gp_TrsfNLerp (`TransformNLerp`, the `NCollection_Lerp<gp_Trsf>` specialization)
and TKernel's NCollection_Lerp (`Lerp`). `tools/audit_api_rows.py` parses every public
member of every header in a package and lists members without an api row and rows that
match no member; for `src/FoundationClasses/TKMath/gp` it reports none. Supplemental
tests are in `gp_vec_api_behavior`, `gp_core_api_behavior`, `gp_trsf_nlerp_behavior` and
TKernel's `ncollection_lerp_supplemental`. The core gp probe sends 2,600 differential
requests (`gp3d_vec`, `gp3d_dir`, `gp3d_pnt`, `gp3d_lin`, `gp3d_pln`, `gp3d_frame`,
`gp3d_lerp`); they all match the native bits.

The port keeps these native results:

- gp_Dir setters renormalize through `SetCoord(x,y,z)`, which raises before it stores,
  so a failed set keeps the old direction. `SetCoord(i,v)` checks the index first.
  `Cross`/`^=`/`CrossCross` keep the unnormalized product when they raise; `^` and `^=`
  map to the fallible `crossed`/`cross` methods.
- `gp_Vec::Mirror(gp_Vec)` does nothing for a mirror vector whose magnitude is not above
  gp::Resolution (NaN included). `Mirror(gp_Ax2)` reverses a vector parallel to the
  normal and otherwise reflects about the normalized `N ^ (N ^ V)`, so it rounds.
  `IsEqual` evaluates `Angle` (which may raise gp_VectorWithNullMagnitude) only when both
  magnitudes exceed the linear tolerance and the lengths already match.
- `gp_Lin::Translated(P1,P2)` adds the precomputed vector `P2 - P1`, while
  `Translate(P1,P2)` adds P2 and then subtracts P1; the results can differ in the last
  ulp.
- `gp_Pln(A,B,C,D)` places the origin on the axis of the branch that selects the X
  direction, including ties; the line and plane distance overloads return 0 unless the
  line is parallel (normal within gp::Resolution) or the normals are parallel.
- `std::hash<gp_Pnt>` sums the truncating quotients of the six 32-bit words of the
  coordinates and hashes that sum with libstdc++ `std::hash<double>`
  (`PointHashPolicy`/`HashKey`); `std::equal_to<gp_Pnt>` accepts differences up to
  `Epsilon` of the second point, and a NaN difference compares equal.
- `NCollection_Lerp` evaluates `(1 - t) * start + t * end`. gp_TrsfNLerp returns the
  stored ends within Precision::Confusion of t = 0 and 1, otherwise sets rotation, then
  translation, then the scale factor, so an interpolated scale at or below gp::Resolution
  keeps `SetScaleFactor`'s ConstructionError (a default-constructed interpolator raises).
- gp_Ax22d constructors and X/Y setters raise gp_Dir2d::SetCoord's ConstructionError for
  a zero direction; the setters keep the native partial update.

Deliberate Rust deviations: gp_Pnt::ChangeCoord returns a write-back view
(`PointCoordinatesMut`) because the point stores a plain coordinate triple; the copy and
move constructors and assignments of gp_Dir are Rust copies. The 2D conics' noexcept
`Reverse` and hyperbola branches, which turn stored unit directions, use an infallible
quarter turn.

### Modern Math* namespaces (API completion)

The modern packages MathUtils, MathInteg, MathLin, MathOpt, MathRoot, MathSys and MathPoly
are C++ namespaces with no primary class. Their structs, `using` aliases and `constexpr`
constants are known identities for `tools/audit_api_rows.py`, and each has a bare
`Namespace::Name` row that points at the Rust type or constant. The audit reads conversion
operators as `operator bool` (it formerly read a member named `bool`), keeps nameless
multi-word builtins such as `long double` whole and spells array parameters
`const double[3][3]` and `const double(&)[N]`. For each of the seven packages the audit
reports no missing member and no stale row. The sections below list what each package needed. The new probes
add 10,475 requests to the numeric corpus (58,169 to 68,644 comparisons), and all of them
give the native bits.

### MathUtils core headers (API completion)

This work adds no upstream suite. It completes the public API of the MathUtils headers
Bracket, Config, Convergence, Core, Deriv, Domain, Gauss, GaussKronrodWeights, LineSearch,
Poly, Random and Types (171 api rows: namespace constants, structs, constructors, methods
and free functions). `MathUtils::Utils` is public as `math_utils::utils`
(`evaluate_finite`, `opposite_signs`, `limited`, `is_valid`, `perturbations`,
`is_finite_vector`); the bracketing and finite-difference algorithms call these functions,
so the public helpers are the implementation and not copies of it. The iteration-count
`BracketMinimum` overload is `bracket_minimum_iterations`. `GetKronrodPointsAndWeights` and
`GetOrderedGaussPointsAndWeights` are new. `Config::StepMin` and the `BoundedConfig` default
bounds now read `precision::computational()` and `precision::infinite()` and do not copy the
values. Supplemental tests are in `math_utils_core_api_behavior`. The core probe sends
2,652 differential requests (`mathutils_core_scalar`, `mathutils_core_defaults`,
`mathutils_core_vector`, `mathutils_core_poly`, `mathutils_core_deriv`,
`mathutils_core_gauss`, `mathutils_core_random`). They all match the native bits. The line
searches, `BracketRoot`/`BracketMinimum(options)`, `NumericalHessian`, `DotProduct` and the
norms were already covered by `utils_line`, `utils_core` and `utils_bracket`. The native
reader cannot parse `inf`/`nan`, so the requests encode them as sentinels that both probes
decode.

The port keeps these native results:

- `CubeRoot` uses glibc 2.39's `cbrt` (sysdeps/ieee754/dbl-64/s_cbrt.c: a polynomial seed,
  one Halley step and a 2^(k/3) factor), and this result is not correctly rounded.
  `cbrt(27)` is `3.0000000000000004`, `cbrt(0.125)` is `0.49999999999999994` and `cbrt(2e100)`
  ends in ...165. Rust's `f64::cbrt` resolves to the compiler-builtins libm (correctly
  rounded, ...164), and so does a direct `extern "C" cbrt` from a Rust binary. The kernel is
  therefore reproduced in `math_utils::glibc_cbrt`. Other Rust callers of `f64::cbrt`
  outside MathUtils can have the same one-ulp difference.
- `Utils::Limited` keeps `std::max(Left, std::min(Right, x))`. A NaN value becomes the
  right limit, and an unordered limit is returned as written. `Utils::Perturbations` writes
  both outputs before its final finiteness/ordering check, so a failed call leaves the
  overflowed values in place.
- `ComputeScaleFactor` uses the same `frexp` as `CubeRoot`. Zero, infinity and NaN return
  exponent 0.

Deliberate Rust differences: pointer+degree/count pairs of the Poly helpers are slices
(length = degree + 1 or count), and their out-parameters are returned tuples. Default
arguments are explicit parameters. `GetGaussPointsAndWeights` above order 61 sorts its
eigenpairs with a total fallback (`Equal`) for an unordered NaN pair instead of the UB of
`std::sort`. Legendre nodes are distinct, so the order does not change.

### MathUtils scalar and vector functors (API completion)

This work adds no upstream suite. It maps every public member of
MathUtils_FunctorScalar.hxx and MathUtils_FunctorVector.hxx (134 api rows: the functor
types, constructors, `Value`/`Values`/`Gradient` members and `Make*` helpers). Every member
was already ported; the rows check each port against the header. The C++ templates are
Rust generics: `math_utils::scalar` holds the scalar functors and
`math_utils::vector` holds the N-dimensional functors and `SystemLambda`. The lambda
wrappers `VectorLambda`/`VectorLambdaWithGradient` and their helpers are in `math_utils`.
`Value`, `Values` and `Gradient` are the `math::Function`,
`FunctionWithDerivative`, `MultipleVarFunction`, `MultipleVarFunctionWithGradient` and
`FunctionSet` trait methods. For `tools/audit_api_rows.py src/FoundationClasses/TKMath/MathUtils`
these two headers have no missing members and no stale rows. Supplemental tests are in
`math_utils_functor_api_behavior`. The new `mathutils_functor` probe sends 1,646 differential
requests (`mathutils_functor_poly` 400, `mathutils_functor_arith` 400,
`mathutils_functor_lambda` 206, `mathutils_functor_vlambda` 240, `mathutils_functor_nd` 400).
The requests use coefficient vectors of length 0 to 9 with shifted bounds, a Gaussian that
fails (zero sigma, NaN input) inside every combinator, a linear operand that becomes zero,
lambdas that fail after they write their outputs, and 0 to 6 coordinates with non-finite
values and non-square residual matrices. All requests give the native bits. The existing
`scalar_functor` and `vector_functor` ops still cover the fixed-size constructors.

The port keeps these native results:

- The binary combinators evaluate the first operand, then the second, and stop at the first
  failure without writing the output. `Composite` evaluates the inner function first. Only
  `Quotient` rejects non-finite operands and a zero denominator. `Sum`, `Difference`,
  `Product`, `Scaled`, `Shifted` and `Negated` return true for non-finite results.
- `ScalarLambdaWithDerivative::Value` calls the callable with a scratch derivative and keeps
  its status. The lambda wrappers do not validate. A callable that writes and then fails
  leaves the written values in the outputs.
- `VectorLambdaWithGradient::Values` and the `Values` member of every vector functor call
  `Value` first and call `Gradient` only when `Value` succeeds.
- `Polynomial` refuses non-finite `x` and an empty coefficient vector. `Degree()` is 0 for an
  empty polynomial, and `Coefficient` returns 0 past the end. `Rational` evaluates an empty
  numerator or denominator as 0, so an empty denominator always fails.
- `math_Vector`/`math_Matrix` arguments are read by zero-based offset (`At`), not by their
  declared bounds. The `math_Vector` constructors copy `Vector::values()` in that order.

Deliberate Rust deviations:

- `math::MultipleVarFunction` and `math::FunctionSet` report the number of variables. For this
  reason the `VectorLambda`, `VectorLambdaWithGradient`, `Sphere`, `Rastrigin`, `Ackley` and
  `SystemLambda` constructors and `make_vector*`/`make_system` take one more leading dimension
  argument. Evaluation never reads it. The native default constructors of `Sphere`,
  `Rastrigin` and `Ackley` therefore become `new(dimension, ...)` with explicit defaults.
  `Rosenbrock`, `Sine`, `Cosine`, `Exponential` and `Gaussian` implement `Default` with the
  native default arguments.
- `math::FunctionWithDerivative` requires a derivative-only method. The scalar functors
  implement it with `Values` and keep only the derivative.
- The `initializer_list` constructors take slices.

### MathInteg and MathLin (API completion)

This work adds no upstream suite. It completes the public API of the MathInteg
(`math_integ`) and MathLin (`math_lin`) namespaces: `tools/audit_api_rows.py` reports no
missing member and no stale row for `src/FoundationClasses/TKMath/MathInteg` and
`src/FoundationClasses/TKMath/MathLin` (96 rows). All solvers were already ported; the
work is:

- The MathInteg `THE_*` constants are public (`GAUSS_MAX_ORDER`, `SET_GAUSS_MAX_ORDER`,
  `MULTIPLE_GAUSS_MAX_ORDER`, `KRONROD_MAX_GAUSS_ORDER`, `KRONROD_AUTO_INITIAL_ORDER`,
  `KRONROD_AUTO_ORDER_STEP`, `KRONROD_AUTO_MAX_ITERATIONS`,
  `DOUBLE_EXP_DEFAULT_MAX_POINTS_PER_LEVEL`, `DOUBLE_EXP_MAX_TRANSFORM_ARGUMENT`,
  `EXP_SINH_MIN_TRANSFORM_ARGUMENT`). The rules, configs and transforms read them
  instead of repeating the literals.
- The QL helpers of `MathLin::Utils` in MathLin_EigenSearch.hxx are public in
  `math_lin::utils`: `find_submatrix_end`, `compute_wilkinson_shift`, `perform_ql_step`,
  `validate_eigenpairs`, `TRIDIAGONAL_DEFLATION_TOLERANCE` and
  `TRIDIAGONAL_RESIDUAL_TOLERANCE_FACTOR`. `eigen_tridiagonal` now calls them, so the
  private copies in `eigen.rs` are gone.

Supplemental tests are in `math_integ_api_behavior` and `math_lin_api_behavior`. The
`integ_lin` probe sends 770 differential requests: `integ_lin_tridiag` (300; every QL
helper, `EigenTridiagonal` and `GetEigenVector`, with NaN, infinite and -0.0 entries,
graded and deflated matrices), `integ_lin_norm` (200; `AccumulateNorm`, `Norm`,
`RelativeTolerance`, both `IsFinite` overloads), `integ_lin_spectral` (150; `EigenValues`,
`SpectralDecomposition`, `MatrixSqrt`, `MatrixInvSqrt`) and `integ_lin_singular` (120;
`TanhSinhSingular` with an explicit and a default tolerance, every constant and every
config default). They all match the native bits. The other members were already covered
by `modern_integ`, `modern_kronrod`, `modern_double_exp`, `modern_tensor`, `modern_eigen`,
`modern_lu` and `modern_factors`.

The port keeps these native results and makes these deliberate changes:

- `FindSubmatrixEnd` uses `std::max`, which keeps a NaN first operand. The Rust helper
  does the same (as do the `std::max` calls in `ValidateEigenpairs`), so a NaN diagonal entry gives a NaN deflation
  threshold and never deflates (`f64::max` would drop the NaN).
- `PerformQLStep` always returns true, also after its zero-radius early exit, which
  only clears the end coupling.
- The helpers index zero-based like `math_Vector::operator[]` and `math_Matrix::At`. An
  index that the native code would read out of bounds (undefined behavior) panics with
  the index and the length.
- `MathLin::EigenResult` is `math_lin::TridiagonalResult`. The different name keeps it
  apart from the `math_utils::EigenResult` that `Jacobi` returns.
- Without eigenvectors, `GetEigenVector` returns an uninitialized one-element native
  vector. Rust returns a zero one-element vector; the probe compares only its size.
- C++ default arguments (`TanhSinhSingular` tolerance `1.0e-10`, `KronrodAuto`
  `theMaxOrder = THE_KRONROD_MAX_GAUSS_ORDER`) are explicit Rust parameters.

### MathOpt (API completion)

This work adds no upstream suite. It completes the public API of the modern MathOpt
namespace: `tools/audit_api_rows.py src/FoundationClasses/TKMath/MathOpt` reports no
missing member and no stale row (76 rows). `MathOpt::Utils` is now the public module
`math_opt::utils`; its helpers were private duplicates inside the solvers and are now the
single implementation the solvers call (`is_valid_config`, `is_finite_vector`/`_matrix`,
`is_valid_bounds`, `norm`, `scalar_value_status`/`value_status`, `checked_value`,
`gradient_status`/`checked_gradient`, `hessian_status`/`checked_hessian`, `CheckedFunction`,
`BoundedFunction`, `is_line_search_evaluation_error`, `line_search_failure_status`,
`line_search_fallback_failure_status`, `is_stationary`, `backtrack`, `regularize_hessian`,
`FRPR_WOLFE_CURVATURE`). The modern `MathOpt::Uzawa`/`UzawaEquality` (`math_opt::uzawa`,
`uzawa_equality`, `UzawaResult`, `UzawaConfig`) is a new port, separate from the classic
`math::Uzawa`. Supplemental tests are in `math_opt_api_behavior`. The probe sends 2,171
differential requests (`mathopt_checks`, `mathopt_line`, `mathopt_eval`,
`mathopt_stationary`, `mathopt_backtrack`, `mathopt_regularize`, `mathopt_polish`,
`mathopt_uzawa`); they all match the native bits. The solvers themselves stay covered by
`modern_opt`, `modern_frpr`, `modern_newton`, `modern_lbfgs`, `scalar_opt`,
`modern_global` and `modern_swarm`.

The port keeps these native results:

- `BoundedFunction` clamps with `std::max(lower, std::min(upper, x))`, so a NaN
  coordinate reaches the callback as the upper bound.
- `IsStationary` rejects a probe only when `value < fx - tol`; with a NaN `fx` no probe
  rejects. Its status output keeps the caller's value for an empty point and otherwise
  holds the last evaluation's status (`Ok` also when a probe found a decrease).
- `RegularizeHessian` folds the Jacobi eigenvalues with `std::min`, so a NaN after the
  first value is ignored and a NaN first value stays the minimum (no shift is applied).
  The earlier private copy used `f64::min`, which chose the non-NaN value.
- `Uzawa` stores `InitialError` before it can fail, so Singular and NumericalError
  results carry it. `InverseCTC` keeps zero-based bounds (native assignment into the
  zero-based `aCTCinv`). The iterative path projects multipliers with `std::max(0, v)`
  (NaN becomes 0) and returns `MaxIterations` without a finiteness check.

Deliberate Rust shapes: C++ overloads get distinct names (`scalar_value_status` for
`ValueStatus(Function&, double, double&)`, `line_search_fallback_failure_status` for the
two-result `LineSearchFailureStatus`), `Value`/`Gradient`/`Hessian` are
`checked_value`/`checked_gradient`/`checked_hessian`, and `CheckedFunction` and
`BoundedFunction` hold `&mut` borrows of the wrapped callback.

### MathRoot helpers and search engine (API completion)

This work adds no upstream suite. It completes the public API of the modern MathRoot
package: `tools/audit_api_rows.py src/FoundationClasses/TKMath/MathRoot` reports no member
without an api row and no stale row (66 rows). `MathRoot::Utils` is public as
`math_root::utils`: the scalar predicates (`IsValidConfig`, `IsValidBounds`,
`HaveOppositeSigns`, `Midpoint`), `ComputePeriodicRootCount`, the multi-root bookkeeping
(`IsValidMultipleInput`, `SortRoots`, `AddRoot`, `EffectiveXTolerance`, the `Evaluate*`
helpers), the functor adapters, `RefineBracketedRoot`, `FindAllRootsImpl`,
`FindAllRootsWithDerivativeImpl`, `FindMultipleRoots` and the trigonometric root
bookkeeping (`AddDistinctTrigRoot`, `MapPeriodicTrigRoot`, `AddPeriodicTrigRoot` and the
three refinement constants). These are the implementations the solvers run, not copies:
`find_all_roots` wires the four Value-only adapters into `find_all_roots_impl` as
MathRoot_Multiple.hxx does, `find_all_roots_with_derivative` calls
`find_all_roots_with_derivative_impl`, the null-interval search calls
`find_multiple_roots`, `trigonometric` inserts through `add_periodic_trig_root`, and Brent,
Bisection, Newton and Secant use the shared predicates. The private duplicates they
replace (inlined sign tests, midpoints, a second config validator, the old value-only
search and bracket polishing) are deleted, and the trigonometric infinite-bound checks now
read `Precision::IsPositiveInfinite`/`IsNegativeInfinite` and `Precision::Computational`
instead of literals. Supplemental tests are in `math_root_api_behavior`. The MathRoot API
probe sends 1,616 differential requests (`mathroot_utils`, `mathroot_trig_roots`,
`mathroot_sort`, `mathroot_search`); they all match the native bits, and the existing
`modern_root`, `modern_multiple` and `modern_trig` requests still match after the refactor.

The port keeps these native results:

- `EffectiveXTolerance`, the extremum probe bounds of `FindAllRootsWithDerivativeImpl` and
  the trigonometric angle tolerance use `std::max`/`std::min` semantics (the first operand
  wins unless the comparison holds), so a NaN bound yields the plain X tolerance and signed
  zeros keep their sign; `f64::max` would differ.
- `AddRoot` compares `|root - old| < eps`, so NaN roots and a NaN tolerance never count as
  duplicates; `SortRoots` is the native insertion sort, including its NaN ordering.
- The sampling adapter stores `f - offset` before checking it, and the Brent value wrapper
  returns an unshifted-then-shifted value without a finiteness check, as natively; the
  derivative wrapper replaces the derivative by a non-finite `f` so Brent reports it.
- `AddDistinctTrigRoot` silently drops a fifth root once the four slots are full.

Deliberate Rust deviations: the C++ adapters alias one `Function&` (and one sample
`math_Vector`) from several structs at once; Rust shares them through
`MultipleFunctionCell` (`RefCell<&mut F>`) and `RefCell<Vector>`, borrowing only for each
call. `operator()` becomes the trait methods `MultipleSampleCallback::sample_at`,
`MultipleSampleLookup::value_at`, `MultipleRootValueCallback::root_value` and
`MultipleIntervalHandler::handle_interval`, and the `mutable Status` members become `Cell`
fields read through `status()`. C++ default arguments become explicit parameters.

### MathSys namespace (API completion)

This work adds no upstream suite. It completes the public API of the MathSys package:
`SystemResult`, `LMConfig`, the generic `Newton`/`NewtonBounded`/`LevenbergMarquardt`
drivers, the fixed-size `Solve2D`/`Solve2DSymmetric`/`Solve3D`/`Solve4D` solvers with
their result, bound and option types, the curve/surface and surface/surface extrema
wrappers, and every member of the `MathSys::Utils` namespace. `MathSys::Utils` is public as
`math_sys::utils`; it re-exports each helper from the solver module that uses it
(`linear`, `newton`, `lm`, and the small-system modules). `MathSys::Utils::IsFinite` does
the same all-entries scan as MathLin's, so the port re-exports `math_lin::utils::is_finite_*`.
Before this work, the fixed-size solvers had private copies of the helpers. Those copies
(`merit.rs`, `NewtonBoundsN::clamp`/`max_step` and a private `solve2`) are deleted, and the
solvers now call the public helpers. `tools/audit_api_rows.py` reports 0 missing and
0 stale members for `src/FoundationClasses/TKMath/MathSys`.

The supplemental tests are in `math_sys_api_behavior`. The MathSys API probe sends 1,620
differential requests: `mathsys_small_linear` (480), `mathsys_small_box` (360),
`mathsys_linear_api` (360) and `mathsys_finish_api` (420). All of them match the native
bits. The inputs include NaN and infinite entries, inverted and inactive boxes,
rank-deficient and wide matrices, and each combination of the two optional bound pointers.
After the refactor, the 1,024 existing `modern_small`, `modern_sys` and `modern_lm`
requests also still match.

The port keeps these native results:

- The libstdc++ `std::clamp` in GCC 13 is `std::min(std::max(v, lo), hi)`. Thus
  `Clamp2D/3D/4D` return `hi` for an inverted box and keep a NaN coordinate. The port
  does not use `f64::clamp`, because that function panics when `lo > hi`.
- For NaN handling, the port keeps the `std::max` operand order. `std::max(a, b)` keeps a
  NaN `a`, and `std::max({...})` keeps the first maximal element, so a NaN survives only
  in the first position. This applies to the `MaxStep*D` nesting
  `max(1, max(d0, max(d1, ...)))` and to the row scales of `Solve2x2`, `Solve3x3`,
  `Solve4x4` and `SolveSymmetric2x2SVD`.
- `Solve3x3` refuses a pivot with `!(|p| > tol)`. `Solve4x4` refuses a pivot with
  `|p| <= tol`, so a NaN pivot gets through. The two routines also round differently:
  3x3 stores `F/s` and negates during back substitution, and it divides for each factor.
  4x4 stores `-F/s`, multiplies by the pivot inverse and zeroes the eliminated entry.
- All the fixed-size solves write their outputs before the finiteness test. A failed call
  can therefore leave non-finite values in `du`/`dv`/`delta`.
- `SolveSymmetric2x2SVD` fails for a diagonal matrix with `j11 < j22`, because it builds
  the eigenvector of the larger eigenvalue from the zero first column.
- `IsStepWithinTolerance` accepts a NaN difference, because it tests
  `|l - r| > tol` for rejection.
- `FinalizeSystemResult` and `FinishLM` copy the caller's solution vector with its lower
  bound. `FinishLM` keeps the gradient, the residual norm and the value when the gradient
  is not finite, but it does not store the Jacobian or the rank.
- The drivers apply bounds only when both bound pointers are given. The port maps each
  pointer to an `Option<&Vector>`, so one bound alone is ignored.
- `THE_CRITICAL_GRAD_SQ` is declared but not used upstream. It is exported unchanged.

The port deviates from the native code in these ways:

- The native default arguments are explicit Rust arguments: `theMaxIter = 100`,
  `SolveSymmetric2x2SVD` `theTol = 1e-15`, the rank tolerances and the default
  configurations.
- The template callbacks map to traits: `SmallSystem<N>` (closures work),
  `SymmetricFunction2D`, and `CurveEvaluatorD2`/`SurfaceEvaluatorD2`. The evaluator traits
  return a point-and-derivatives struct instead of writing through `D2` out parameters.
- The three identical dimension-specific helpers (`IsOptionsValid*`, `IsBoundsValid*D`,
  `MaxStep*D`, `Clamp*D`) keep their native names as wrappers around one const-generic
  body.
### Bnd package completion: OBB, BoundSortBox and Tools

Every file of the Bnd package is now ported (65 new API rows). `OrientedBoundingBox`
is Bnd_OBB with the DiTO construction of `ReBuild`: the closed forms for one and two
points, the 7-axis extremal-point search, the ditetrahedron of the default mode and
the all-triangle search of the optimal mode, which projects the points through a
linear BVH (30 elements per leaf) with the `OBB_ExtremePointsSelector` traversal.
`BoundSortBox` is Bnd_BoundSortBox with its voxel grid and large-box list, and
`box2_to_bvh`/`box3_to_bvh` are Bnd_Tools. The Bnd_Box and Bnd_Box2d members that
no legacy test calls are added too (`update_point`, the structured `limits()`, the
`x_min()`..`z_max()` getters, `Dump()` as `dump(&mut impl Write)`, the Box2d rays
and transformed IsOut overloads), as are the transformed `IsOut`/`IsIn` overloads of
Bnd_B2/Bnd_B3 (`is_out_transformed_box`, `is_in_transformed_box`). The Box2d
`limits()` now returns `BoxLimits2` in the native Xmin, Xmax, Ymin, Ymax field order;
before, it returned the `Get(x,y,X,Y)` order under the structured-Get name.

Bnd_OBB_Test.cxx and Bnd_BoundSortBox_Test.cxx link BRepPrimAPI/BRepBndLib, so their
native GoogleTest runs are not mapped and their 15 declarations are not counted
above. Every body is translated in `bnd_obb_supplemental.rs` and
`bnd_bound_sort_box_supplemental.rs` (named after the upstream case); the B-rep
inputs become plain data: the eight corners of the MakeBox cube for OCC30704 (the
native CheckPoints list repeats each vertex once per incident edge and face, which
changes nothing for an axis-aligned cube), and the cube and face boxes enlarged by
`Precision::Confusion()` for BUC60729. Another 2,909 differential requests
(`tools/tkmath_bnd_corpus.py`) match with raw bits; the numeric gate now passes
58,478 cases. They cover ReBuild on random, box-corner,
grid, collinear, coplanar, duplicate, elongated, spherical, tiny and huge clouds in
both modes, with and without tolerances, with array bounds not based at 1 and with
mismatched tolerance bounds; the OBB separating-axis, containment, Add, Enlarge,
SetVoid, Position and DumpJson results; BoundSortBox in all initialization modes up to
1,100 boxes (grid resolutions 8, 16 and 32), with void, open, degenerate and large
boxes, reporting every Compare result list in order; the new Box/Box2d members; and
the transformed B2/B3 relations in double and float for every transform form.

Preserved quirks: equally extreme points are resolved by the first point visited, in
list order in the default mode and in BVH traversal order in the optimal mode, so
ties change the chosen axes; `IsOut(gp_Pnt)` has no void guard; `Enlarge` also grows
a void box; `SetVoid` leaves null axes, so `Position()` of a void box raises
Standard_ConstructionError; the size checks of the constructor and the
`Set?Component` setters are `Standard_ASSERT_VOID`, compiled out of release builds, so
negative sizes are stored; the "use Precision::SquareConfusion()" test of BuildBox is
the literal `1.0e-14` on the product of the three squared axis norms. BoundSortBox
never resets its enclosing box in `Initialize(boxes)`, so repeated calls grow it; a
void enclosing box keeps the previous grid, coefficients and large-box list; a box
with an open side under a closed enclosing box maps that side to `INT_MIN`
(`cvttsd2si`), the ±1 voxel margin wraps and the clamped range is empty, so the box
never enters the grid and is never reported (the native result is the same); large
boxes are reported before grid boxes, which follow the Z slices of the query.
`bnd_obb_behavior.rs`, `bnd_sort_box_behavior.rs` and `bnd_members_behavior.rs` pin
them.

Deviations are limited to undefined native behaviour and ownership. ReBuild of an
empty list reads `First()` of an empty array natively; the port raises
Standard_OutOfRange, as it does for a NaN cloud that leaves a base triangle index at
`INT_MAX`. Add and Compare before any Initialize dereference a null handle natively;
the port raises Standard_NullObject for Add and returns an empty list for Compare.
The native class shares the caller's `NCollection_HArray1` handle, so `Add` writes
into the caller's array; the port owns the array and exposes it through `boxes()`.
The Bnd_Array1OfBox/Bnd_HArray1OfBox (and 2d) typedefs live in the deprecated
NCollection alias headers and carry no behavior of their own; this revision has no
Bnd_SeqOfBox and no Bnd_BoundSortBox2d.

### BVH, Convert, BSplCLib and BSplSLib (API completion)

This work adds no upstream suite. `tools/audit_api_rows.py` now reports no missing
member and no stale row for the BVH, Convert, BSplCLib and BSplSLib packages. Most gaps
were ledger spellings: rows used informal parameter names (`min`, `cmax`, `4 corners`)
or wrote the template arguments of a primary template (`BVH_Box<T,N>::`,
`Convert_CompBezierCurvesToBSplineCurveBase<PointType,VecType>::`), which the ledger
spells as the bare class name. The rows now carry the exact header signatures.

The audit itself had gaps, fixed with tests in `tools/tests/test_audit_api_rows.py`:
out-of-class template definitions (`BVH_Box<T, N>::Area() const`) were counted as new
members; a class specialization's base clause leaked into its scope name;
`handle<B<T, N>>` and dependent `typename B<T, N>::V` parameters were cut at the first
`>`; and `DEFINE_STANDARD_RTTIEXT`, which expands to `public:` and has no semicolon,
swallowed the next declaration and its access label (in Poly this hid every public
member of Poly_Triangulation and Poly_PolygonOnTriangulation). Typedefs, `using`
aliases, data members, class names, enumerators and protected members are now known
identities: a row that names one is not stale, but none of them needs a row. A row
with status `excluded` must carry a `notes` reason.

Newly ported: `BVH::ToBndBox(const T&, const T&)` (`to_bnd_box_scalar`),
`BVH::MatrixOp<T, 4>::Inverse`/`Multiply` (`matrix_op_inverse`/`matrix_op_multiply`,
now also used by `BvhTransform`), and `BVH::SurfaceCalculator::Area` is public as
`surface_area`. The `bvh_tool_classes` probe sends 96 differential requests that cover
every `BVH` namespace tool class (ToBndBox, VecComp, UnitVector, MatrixOp,
SurfaceCalculator, BVH_AxisSelector, CenterAxis, BoxMinMax) for float and double in
2D, 3D and 4D, including tied, signed-zero, tiny and huge components and singular and
W = 0 matrices; they all match the native bits.

Excluded rows (status `excluded`, with the reason in `notes`): the destructors, the
`DEFINE_STANDARD_RTTIEXT` members, the defaulted constructors of the stateless abstract
`BVH_Set` and `BSplCLib/BSplSLib_EvaluatorFunction`, and the two Bezier overloads
`BSplCLib::DN(U, N, Poles, Weights, P, VN)`, which the pinned header declares but no
pinned source defines (libTKMath exports only the three B-spline `DN` overloads), so a
call cannot link.

### Classic math package (API completion)

This work adds no upstream suite. `tools/audit_api_rows.py` now reports no missing member
and no stale row for `src/FoundationClasses/TKMath/math` (515 rows: 499 implemented,
16 excluded). Supplemental tests are in `math_vector_base_behavior`,
`math_recipes_behavior` and `math_solver_api_behavior`. Three probes send 746 differential
requests: `int_vector`, `real_vector_extra` and `matrix_extra` (207), `recipes_lu`,
`recipes_svd`, `recipes_jacobi`, `recipes_dactcl` and `linear_dump` (358), and
`dump_minimizers`, `dump_scalar`, `dump_polynomial`, `dump_sets` and `small_types` (181).
The solver probes run each solver with its default convergence test and with an OCCT
subclass that overrides `IsSolutionReached`/`IsConverged`; they all match the native bits
and Dump text.

Structure: `math_VectorBase<TheItemType>` is the generic `VectorBase<'a, T: VectorItem>`
with the `Vector` (math_Vector) and `IntegerVector` (math_IntegerVector) aliases.
`math_Recipes` is the `math::recipes` module; math_Gauss, math_SVD, math_Jacobi,
math_GaussLeastSquare and math_NewtonFunctionSetRoot now call its kernels as they do
natively, and their private copies were deleted (the SVD kernel is general in its row
and column counts). The virtual `IsSolutionReached(F)` of BFGS, FRPR, Powell,
BrentMinimum, BissecNewton, FunctionSetRoot and NewtonFunctionSetRoot and
`NewtonMinimum::IsConverged` are replaced through `perform_with_criterion` closures,
called at the native call sites; accessors such as `current_minimum`,
`previous_minimum`, `current_step` and `tolerance` expose the protected state that
subclasses (the AppDef BFGS gradients, FairCurve_Newton) read. New ports:
math_GaussSetIntegration, math_ValueAndWeight (used by both Gauss/Kronrod node
generators), math_Recipes, the empty integration constructors, `PSO_Particle`'s
constructor and `operator<`, and every `Dump`/`operator<<`.

The port keeps these native results:

- `math_IntegerVector` follows the template's C++ expression rules: mixed double
  expressions convert back with truncation, so `Divided(d)` multiplies by the truncated
  reciprocal (zero unless `d` is +-1), `Normalize` truncates every element, and the
  matrix products truncate every partial sum.
- `math_Matrix::Transpose` rebases the columns onto the row lower bound before its swap
  loop but keeps the old upper column: with column bounds above the rows it raises
  Standard_OutOfRange part-way (bounds already rebased); with column bounds below the
  rows it swaps nothing and only exchanges the lower bounds.
- `math_Gauss::Dump` prints the permutation sign `D` as "Determinant of A". The Dump
  texts keep OCCT's inconsistent spacing (for example `math_GaussLeastSquare Status =
  not Done` but `math_GaussLeastSquare  Status = Done`), and doubles use the stream's
  `%g` formatting. `math_NewtonMinimum::Dump` reads `Location()`, so a solver that is
  not done raises StdFail_NotDone after the first status lines.
- math_BFGS/FRPR/Powell/NewtonMinimum/BissecNewton record math_Status at each native
  exit (for example FRPR reports math_FunctionError for a zero gradient norm and Powell
  for a failed extrapolated line search); the initial status is math_OK for BFGS and
  math_NotBracketed for the others.
- `math_ComputeGaussPointsAndWeights(0)` is a not-done result (the empty eigenproblem
  raises Standard_RangeError inside the constructor's try block).
- An empty `math_PSOParticlesPool` is valid; particles with zero dimensions raise
  Standard_OutOfRange when the pool takes pointers into its empty memory block.
- `Jacobi`'s EigenSort orders `d.Length()` entries, the last of equal maxima winning.

Undefined native behaviour is refused instead of reproduced: `math_Recipes.cxx` disables
range checks, so kernels refuse arrays that do not cover the 1-based indices they walk
(Standard_OutOfRange, or Standard_DimensionError for an SVD with fewer rows than
columns, which writes past the matrix natively); `math_ComputeKronrodPointsAndWeights(0)`
reads outside its coefficient arrays and negative quadrature orders or pool sizes crash
the native build, so these are refused; `GetBestParticle`/`GetWorstParticle` on an empty
pool dereference `end()` natively and raise Standard_OutOfRange here. math_FunctionSetRoot's
two-point line search ignores the status of `F.Value(tsol, fsol)` and then reads the
uninitialized `fsol` when the callback fails; the port reads 0, and the probes keep the
general solver on callbacks that never fail because the native result depends on stack
contents.

Deliberate Rust deviations: `Standard_OStream` is any `std::fmt::Write`; the zero-based
`operator[]` is `at`/`change_at` because Rust indexing is the declared-bound
`operator()`; `Array1()` returns a shared view; the untyped `void*` buffer constructors
take borrowed `f64` slices. Excluded rows: the destructors, and
`math_NewtonMinimum::IsConvex` and `math_NewtonFunctionSetRoot::StateNumber`, which the
headers declare but no pinned source defines.

**Progress and cancellation in math_Gauss and LU_Decompose.** The three rows that were
partial (the `Message_ProgressRange` argument of the math_Gauss constructor and of both
LU_Decompose overloads) are implemented now that TKernel ports the Message progress classes.
`Gauss::with_progress`, `lu_decompose_with_progress` and
`lu_decompose_with_scales_and_progress` take a `&ProgressRange`; the earlier functions keep
their signatures and pass the default (unbound) range. As natively, LU_Decompose opens a
`"math_Gauss LU_Decompose"` scope of `n` steps before the row scaling, polls the user break
before each column (the `j <= n && aPS.More()` loop condition), advances one step per
finished column through a temporary `Next()` range, and returns `math_Status_UserAborted`
(-1) with the partial factorization in place when the break stops the loop; the Gauss solver
is then not done. Every exit, the singular ones included, closes the scope and completes the
range. A non-square matrix is refused before the range is consumed (natively
`math_NotSquare` is raised first as well). `math_progress_behavior` tests a cancelling user
indicator, and the `progress_lu`/`progress_gauss` probes (149 requests in
`tools/tkmath_progress_corpus.py`) compare status, factors, pivots, determinant, poll count
and every Show position bit-for-bit with OCCT for sizes 1..6, every break point and singular
early exits.

### Bnd (API completion)

This work adds no upstream suite. `tools/audit_api_rows.py` now reports no missing
member and no stale row for the Bnd package (before: 144 missing, 7 stale). The
members were ported with the package, but the ledger had rows only for the members the
legacy suites do not call: 141 rows are added with the exact header signatures, for
every public member of Bnd_Box, Bnd_Box2d, Bnd_B2 and Bnd_B3 and for the
`Bnd_Range::IntersectStatus` enum. Stale spellings are corrected: the transformed
`IsOut`/`IsIn` rows of Bnd_B2/Bnd_B3 wrote the primary template with its argument
(`Bnd_B3<RealType>::`), which the ledger spells as the bare class name, and the
Bnd_BoundSortBox `Initialize` rows now name `occ::handle<NCollection_HArray1<Bnd_Box>>`.

The audit kept the name and extent of an array parameter, so
`Bnd_OBB::GetVertex(gp_Pnt theP[8])` matched no row. C++ adjusts an array parameter
to a pointer, and the audit now does the same (`gp_Pnt*`; `T p[2][3]` becomes
`T(*)[3]`), with a test in `tools/tests/test_audit_api_rows.py`.

Newly ported: the `std::array` overloads of `SetCenter`/`SetHSize` of Bnd_B2/Bnd_B3
(`set_center_array`, `set_half_size_array`), which copy the storage scalars without
the double-to-`RealType` conversion of the `gp_XY`/`gp_XYZ` overloads. The
`compact_arrays` probe sends 200 differential requests that cover, for float and
double in 2D and 3D, the array constructors, both setter overloads, `Center`/`HSize`,
`Add` of a point and of a box, and `Clear`, with float-inexact, zero, negative (void)
and huge half sizes (the float `SquareExtent` overflows to infinity natively too);
they all match the native bits, and the numeric gate now passes 61,374 cases.

### Poly and TopLoc packages (API completion)

This work adds no upstream suite: TKMath GTests hold only Poly_ArrayOfNodes and
TopLoc_Location suites, both already mapped. `tools/audit_api_rows.py` reports 0
missing and 0 stale rows for Poly and TopLoc (412 new rows). Every file of both
FILES.cmake is ported: Poly_Triangulation (UV nodes, single-precision normals, the
cached min-max range, the late-load interface), the three polygon classes,
Poly_Triangle, Poly_TriangulationParameters, Poly_MeshPurpose, the static Poly
functions, Poly_Connect, Poly_CoherentTriangulation with its node, link, triangle and
triangle-ring classes, Poly_MergeNodesTool and Poly_MakeLoops (2D and 3D); TopLoc_Datum3D,
TopLoc_ItemLocation, TopLoc_SListOfItemLocation and its node, the rest of
TopLoc_Location (Divided, Predivided, Powered, HashCode, dumps) and the map aliases.

Shared mutation follows docs/geom-design.md section 3: each transient Poly class keeps its
fields in one TKernel `standard::SharedState` (an `RwLock` that ignores poisoning, shared
with the Geom2d classes), every member takes `&self`, and a change through one `Handle` is
seen through all. Getters that return `const&` natively return copies; `Internal*`/`Change*`
lend the array to a closure under the write lock. The polygons share their parameters as a
TKernel `ncollection::HArray1OfReal` (`NCollection_HArray1<double>`). Poly_CoherentTriangulation,
Poly_Connect, Poly_MergeNodesTool and Poly_MakeLoops are algorithm objects with `&mut`
members. TKernel gains `standard::hash_bytes`, a public `next_map_prime`, and a
`PackedCursor` that reads a packed map live, as the native iterator does while
Poly_MakeLoops removes keys.

Evidence: 2,280 differential requests (`tools/tkmath_polymesh_corpus.py`,
`tools/tkmath_toploc_corpus.py`) match the native bits (the numeric gate now passes
63,454 cases), and
`poly_triangulation_behavior.rs`, `poly_polygon_io_behavior.rs`,
`poly_mesh_tools_behavior.rs` and `toploc_behavior.rs` pin the behaviour below.

Preserved quirks: a triangulation, `ResizeNodes` or `AddUVNodes` with zero nodes raises
Standard_RangeError (a zero-length Poly_ArrayOfNodes), so does `MergeNodesTool::Result`
without nodes; `Poly_Triangulation(Nodes, Triangles)` and both Poly_PolygonOnTriangulation
array constructors keep the lower bound of their input arrays; Poly_Polygon3D reads each
parameter at its node's index, and `Copy` allocates one parameter per node, so tables of
another length raise Standard_DimensionMismatch; `UpdateCachedMinMax` goes through `MinMax`
with an identity and so returns an existing cache unchanged; `ComputeNormals` sums
unnormalized single-precision triangle normals and gives (0, 0, 1) to a node without
area; `Poly::Intersect` writes the triangle at every improvement but the distance only on
success; `PointOnTriangle` on collinear sides reports the unclamped parameter of the
closer side; `Poly::Write` prints six significant digits, so a round trip rounds;
`Poly_Connect::Triangles` lists the neighbours across (n1,n2), (n2,n3), (n3,n1) (the
header comment says otherwise) and `Triangle(N)` is the last triangle that wrote node N.
A coherent node without UV keeps the infinite UV (`Precision::Infinite()`), and
`GetTriangulation` exports it because u² + v² exceeds the confusion; `ComputeLinks` without
`ClearLinks` leaves triangles pointing at cleared link slots, which
`NCollection_DynamicArray::Clear` keeps, so the port keeps the slots too. The merge map
keeps the NCollection_BaseMap bucket policy, head insertion and resize order, the DJB2 hash
of the float bytes (so -0 and +0 land in different buckets) and the cell hash of
NCollection_CellFilter, because with a tolerance the first match wins; the tolerance and the
angle are stored in single precision; an angle whose cosine is at most 0.01 skips the normal
computation and compares against the previous normal; a degenerate quad is kept, and without
an output a quad counts as one element. Poly_MakeLoops reads the node order of both the
stored links and the helper's links. TopLoc_Location::HashCode hashes datum addresses, so
the value is specific to one process; the probe checks the recurrence on synthetic ids and
against the native value on real addresses.

Deviations are limited to undefined or non-terminating native behaviour and ownership.
`Poly_ArrayOfNodes::Move` leaves the source empty (natively a non-owning alias of the same
bytes). The default Poly_CoherentNode normal is zero (natively uninitialized); an
out-of-range coherent index raises Standard_OutOfRange (natively unchecked); removing a
triangle from a free node returns false (natively a null dereference); a coherent
triangulation built from a null handle has deflection 0 (natively uninitialized). When
`RemoveDegenerated` would restart forever from a removed ring head, or `Perform` would
repeat a step that consumes no start link (seen when a helper still lists a link that
`ReplaceLink` swapped), the port raises Standard_ProgramError instead of looping.
`Poly::Read*` reject a token that libstdc++ extraction refuses (natively the fields stay
uninitialized). OSD_FileSystem is not ported, so a `DeferredTriangulation` owns its input;
the allocator arguments of the coherent classes are not kept (`Allocator` is an excluded
row). `Poly_Polygon3D::Parameters` without parameters and `TopLoc_Location::Powered(INT_MIN)`
on a multi-item chain raise Standard_NullObject and Standard_OutOfRange where native code
dereferences null or recurses forever. `NextLocation` of the identity is the identity itself.
