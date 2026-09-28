// Differential coverage of the MathUtils core API (Core, Config, Convergence, Domain, Poly,
// Deriv, Bracket overloads, Utils helpers, Gauss/Kronrod wrappers, Random) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathUtils_Bracket.hxx>
#include <MathUtils_Config.hxx>
#include <MathUtils_Convergence.hxx>
#include <MathUtils_Core.hxx>
#include <MathUtils_Deriv.hxx>
#include <MathUtils_Domain.hxx>
#include <MathUtils_Gauss.hxx>
#include <MathUtils_GaussKronrodWeights.hxx>
#include <MathUtils_LineSearch.hxx>
#include <MathUtils_Poly.hxx>
#include <MathUtils_Random.hxx>
#include <vector>

// Python request text cannot carry inf/NaN through `operator>>`, so sentinels encode them.
static double mucore_get(std::istream& in) {
  double v; in >> v;
  if (v == 7.77e307) return std::numeric_limits<double>::infinity();
  if (v == -7.77e307) return -std::numeric_limits<double>::infinity();
  if (v == 6.66e307) return std::numeric_limits<double>::quiet_NaN();
  return v;
}
static int mucore_int(std::istream& in) { return static_cast<int>(mucore_get(in)); }
static void mucore_out(std::initializer_list<double> theValues) {
  for (double aValue : theValues) std::cout << bits(aValue) << ' ';
}
static void mucore_domain1(const MathUtils::Domain1D& d, double x, double t, double r) {
  mucore_out({d.Min, d.Max, d.Length(), d.Mid(), d.Clamp(x), d.Lerp(x), d.Normalize(x)});
  std::cout << d.IsValid() << d.Contains(x, t) << d.IsLarge(t) << d.IsFullPeriod(r, t)
            << d.IsFinite(r) << d.IsEqual(MathUtils::Domain1D(d.Max, d.Min), t) << ' ';
}
static void mucore_domain2(const MathUtils::Domain2D& d, double x, double y, double t, double r) {
  mucore_out({d.UMin, d.UMax, d.VMin, d.VMax, d.ULength(), d.VLength(), d.UMid(), d.VMid()});
  mucore_domain1(d.U(), x, t, r);
  mucore_domain1(d.V(), y, t, r);
  double u = x, v = y;
  d.Clamp(u, v);
  mucore_out({u, v});
  std::cout << d.IsValid() << d.Contains(x, y, t) << d.IsLarge(t) << d.IsUFullPeriod(r, t)
            << d.IsVFullPeriod(r, t) << d.IsFinite(r) << ' ';
}
static void mucore_configs(double a, double b, double t, uint32_t n) {
  MathUtils::Config c(t, n);
  MathUtils::BoundedConfig bc(a, b, t, n);
  MathUtils::NDimConfig nc(t, n, a < b);
  MathUtils::IntegConfig ic(t, n);
  MathUtils::LinConfig lc;
  lc.SingularityTolerance = t;
  std::cout << c.IsValid() << bc.IsValid() << nc.IsValid() << nc.UseBounds << ic.IsValid()
            << lc.IsValid() << ' ';
  mucore_out({c.StepMin, bc.StepMin, bc.LowerBound, bc.UpperBound, nc.RelativeTolerance});
}
static void mucore_bracket_options(double x, double a, double b, double r, uint32_t n) {
  MathUtils::MinBracketOptions o;
  o.MaxIterations = n;
  o.UseLimits = true;
  o.LeftLimit = a;
  o.RightLimit = b;
  o.HasFA = r > 0.0;
  o.FA = x;
  o.HasFB = r > 1.0;
  o.FB = r;
  mucore_out({MathUtils::Utils::Limited(x, o)});
  std::cout << MathUtils::Utils::IsValid(o) << ' ';
  o.UseLimits = false;
  mucore_out({MathUtils::Utils::Limited(x, o)});
  std::cout << MathUtils::Utils::IsValid(o) << ' ';
}

// mathutils_core_scalar x a b t r n
void mathutils_core_scalar_case(std::istream& input) {
  const double x = mucore_get(input), a = mucore_get(input), b = mucore_get(input);
  const double t = mucore_get(input), r = mucore_get(input);
  const int n = mucore_int(input);
  const uint32_t aN = static_cast<uint32_t>(n);
  mucore_out({MathUtils::Clamp(x, a, b), MathUtils::SafeDiv(a, b, x), MathUtils::SignTransfer(a, b),
              MathUtils::Sqr(x), MathUtils::Cube(x), MathUtils::CubeRoot(x)});
  std::cout << MathUtils::IsZero(x, t) << MathUtils::IsEqual(a, b, t)
            << MathUtils::IsWithinTolerance(a, b, t, r) << MathUtils::IsFinite(x) << ' '
            << MathUtils::Sign(x) << ' ';
  const MathUtils::Config c(t, aN);
  std::cout << MathUtils::IsXConverged(a, b, t) << MathUtils::IsFConverged(x, t)
            << MathUtils::IsConverged(a, b, x, c) << MathUtils::IsMinConverged(a, b, x, r, c)
            << MathUtils::Utils::OppositeSigns(a, b) << ' ';
  double aMinus = -7.0, aPlus = -7.0;
  std::cout << MathUtils::Utils::Perturbations(x, t, aMinus, aPlus) << ' ';
  mucore_out({aMinus, aPlus});
  mucore_configs(a, b, t, aN);
  mucore_bracket_options(x, a, b, r, aN);
  mucore_domain1(MathUtils::Domain1D(a, b), x, t, r);
  mucore_domain2(MathUtils::Domain2D(a, b, x, r), a, x, t, r);
  mucore_domain2(MathUtils::Domain2D(MathUtils::Domain1D(b, a), MathUtils::Domain1D(r, x)), b, r, t, x);
  std::cout << '\n';
}

// mathutils_core_defaults (no values): default members and namespace constants.
void mathutils_core_defaults_case(std::istream&) {
  const MathUtils::Config c;
  const MathUtils::BoundedConfig bc;
  const MathUtils::NDimConfig nc;
  const MathUtils::IntegConfig ic;
  const MathUtils::LinConfig lc;
  const MathUtils::MinBracketOptions mo;
  const MathUtils::LineSearchResult lr;
  const MathUtils::Domain1D d1;
  const MathUtils::Domain2D d2;
  std::cout << c.MaxIterations << ' ' << c.IsValid() << bc.IsValid() << nc.IsValid() << nc.UseBounds
            << ' ' << ic.InitialOrder << ' ' << ic.MaxOrder << ' ' << ic.MaxIterations << ' '
            << ic.IsValid() << lc.UsePivoting << lc.IsValid() << ' ' << mo.MaxIterations << ' '
            << mo.UseLimits << mo.HasFA << mo.HasFB << ' ' << int(lr.Status) << lr.IsValid << ' '
            << lr.NbEvals << ' ' << MathUtils::THE_NEWTON_MAX_ITER << ' '
            << MathUtils::THE_NEWTON2D_STAGNATION_COUNT << ' ' << MathUtils::THE_NEWTON2D_LINE_SEARCH_MAX
            << ' ';
  mucore_out({c.Tolerance, c.XTolerance, c.FTolerance, c.RelativeTolerance, c.StepMin,
              bc.LowerBound, bc.UpperBound, nc.Tolerance, ic.Tolerance, lc.SingularityTolerance,
              mo.LeftLimit, mo.RightLimit, mo.FA, mo.FB, lr.Alpha, lr.FNew, d1.Min, d1.Max,
              d2.UMin, d2.UMax, d2.VMin, d2.VMax});
  mucore_out({MathUtils::THE_EPSILON, MathUtils::THE_ZERO_TOL, MathUtils::THE_PI, MathUtils::THE_2PI,
              MathUtils::THE_EULER_NUMBER, MathUtils::THE_GOLDEN_RATIO, MathUtils::THE_GOLDEN_SECTION,
              MathUtils::THE_NEWTON_FTOL_SQ, MathUtils::THE_NEWTON_STEP_TOL_FACTOR,
              MathUtils::THE_NEWTON2D_SINGULAR_DET, MathUtils::THE_NEWTON2D_CRITICAL_GRAD_SQ,
              MathUtils::THE_NEWTON2D_MAX_STEP_RATIO, MathUtils::THE_NEWTON2D_DOMAIN_EXT,
              MathUtils::THE_NEWTON2D_STAGNATION_RATIO, MathUtils::THE_NEWTON2D_SKIP_LINESEARCH_SQ,
              MathUtils::THE_NEWTON2D_STAGNATION_RELAX, MathUtils::THE_NEWTON2D_BACKTRACK_TRIGGER,
              MathUtils::THE_NEWTON2D_BACKTRACK_ACCEPT, MathUtils::THE_NEWTON2D_MAXITER_RELAX,
              MathUtils::THE_HESSIAN_DEGENERACY_REL, MathUtils::THE_HESSIAN_DEGENERACY_ABS,
              MathUtils::THE_ARMIJO_C1, MathUtils::THE_EXACT_LINE_SEARCH_ALPHA_MAX});
  std::cout << '\n';
}

// mathutils_core_vector n m x[n] y[m] tol
void mathutils_core_vector_case(std::istream& input) {
  const int n = mucore_int(input), m = mucore_int(input);
  math_Vector xs(static_cast<size_t>(n)), ys(static_cast<size_t>(m));
  for (int i = 0; i < n; ++i) xs.ChangeAt(i) = mucore_get(input);
  for (int i = 0; i < m; ++i) ys.ChangeAt(i) = mucore_get(input);
  const double t = mucore_get(input);
  std::cout << MathUtils::IsVectorConverged(xs, ys, t) << MathUtils::IsGradientConverged(xs, t)
            << MathUtils::Utils::IsFiniteVector(xs) << ' ';
  mucore_out({MathUtils::InfinityNorm(xs), MathUtils::EuclideanNorm(xs), MathUtils::Norm(xs)});
  std::cout << '\n';
}

// mathutils_core_poly deg c0..c5 x root iter tol count r0..r3
void mathutils_core_poly_case(std::istream& input) {
  const int deg = mucore_int(input);
  double c[6]; for (double& v : c) v = mucore_get(input);
  const double x = mucore_get(input), root = mucore_get(input);
  const int iter = mucore_int(input);
  const double tol = mucore_get(input);
  const int count = mucore_int(input);
  double r[4]; for (double& v : r) v = mucore_get(input);
  double aValue = -3.0, aDeriv = -3.0;
  MathUtils::EvalPolyDeriv(c, deg, x, aValue, aDeriv);
  mucore_out({MathUtils::EvalPoly(c, deg, x), aValue, aDeriv, MathUtils::EvalPolyDesc(c, deg, x),
              MathUtils::RefinePolyRoot(c, deg, root, static_cast<uint32_t>(iter)),
              MathUtils::RefinePolyRootDesc(c, deg, root, static_cast<uint32_t>(iter))});
  MathUtils::SortRoots(r, static_cast<size_t>(count));
  mucore_out({r[0], r[1], r[2], r[3]});
  std::cout << MathUtils::RemoveDuplicateRoots(r, static_cast<size_t>(count), tol) << ' ';
  mucore_out({r[0], r[1], r[2], r[3]});
  double p, q, s, rr;
  MathUtils::DepressCubic(c[1], c[2], c[3], p, q, s);
  mucore_out({p, q, s});
  MathUtils::DepressQuartic(c[1], c[2], c[3], c[4], p, q, rr, s);
  mucore_out({p, q, rr, s});
  std::cout << '\n';
}

struct MuCoreScalar {
  int mode; double center; std::vector<double> calls;
  bool Value(double x, double& value) {
    calls.push_back(x);
    value = (x - center) * (x - center) * (x - center) + 0.5 * x * x;
    if (mode == 1 && x > center) return false;
    if (mode == 2 && x > center) value = std::numeric_limits<double>::quiet_NaN();
    if (mode == 3) value = 1.0 / (x - center);
    return true;
  }
};
struct MuCoreVector {
  int mode; double center; int nbCalls = 0;
  bool Value(const math_Vector& x, double& value) {
    ++nbCalls;
    value = (x.At(0) - center) * (x.At(0) - center) * x.At(1) + x.At(1) * x.At(1) * x.At(1);
    if (mode == 1 && x.At(0) > center) return false;
    if (mode == 2 && x.At(0) > center) value = std::numeric_limits<double>::quiet_NaN();
    return true;
  }
  bool Value(const math_Vector& x, math_Vector& f) {
    ++nbCalls;
    f.ChangeAt(0) = x.At(0) * x.At(1) - center;
    f.ChangeAt(1) = x.At(0) * x.At(0) + x.At(1);
    if (mode == 1 && x.At(0) > center) return false;
    if (mode == 2 && x.At(0) > center) f.ChangeAt(1) = std::numeric_limits<double>::infinity();
    return true;
  }
};

// mathutils_core_deriv mode center x step fx x1 iterations
void mathutils_core_deriv_case(std::istream& input) {
  const int mode = mucore_int(input);
  const double center = mucore_get(input), x = mucore_get(input), step = mucore_get(input);
  const double fx = mucore_get(input), x1 = mucore_get(input);
  const int iterations = mucore_int(input);
  MuCoreScalar f{mode, center, {}};
  double d = -5.0;
  std::cout << MathUtils::CentralDifference(f, x, d, step) << ' '; mucore_out({d});
  std::cout << MathUtils::ForwardDifference(f, x, fx, d, step) << ' '; mucore_out({d});
  std::cout << MathUtils::SecondDerivative(f, x, fx, d, step) << ' '; mucore_out({d});
  MathUtils::Status aStatus = MathUtils::Status::OK;
  double v = -5.0;
  std::cout << MathUtils::Utils::EvaluateFinite(f, x, v, aStatus) << ' ' << int(aStatus) << ' ';
  mucore_out({v});
  const MathUtils::MinBracketResult m =
    MathUtils::BracketMinimum(f, x, x + step, static_cast<uint32_t>(iterations));
  std::cout << int(m.Status) << ' ' << m.IsValid << ' ' << m.NbIterations << ' ';
  mucore_out({m.A, m.B, m.C, m.Fa, m.Fb, m.Fc});
  std::cout << f.calls.size() << ' ';
  for (double c : f.calls) std::cout << bits(c) << ' ';
  MuCoreVector g{mode, center};
  math_Vector aPoint(3, 4); aPoint(3) = x; aPoint(4) = x1;
  math_Vector aGrad(0, 1, -9.0);
  std::cout << MathUtils::NumericalGradient(g, aPoint, aGrad, step) << ' ';
  mucore_out({aGrad(0), aGrad(1), aPoint(3), aPoint(4)});
  aGrad.Init(-9.0);
  std::cout << MathUtils::NumericalGradientAdaptive(g, aPoint, aGrad, step) << ' ';
  mucore_out({aGrad(0), aGrad(1), aPoint(3), aPoint(4)});
  math_Matrix aJac(1, 2, 5, 6, -9.0);
  std::cout << MathUtils::NumericalJacobian(g, aPoint, aJac, step) << ' ';
  mucore_out({aJac(1, 5), aJac(1, 6), aJac(2, 5), aJac(2, 6), aPoint(3), aPoint(4)});
  std::cout << g.nbCalls << '\n';
}

// mathutils_core_gauss order
void mathutils_core_gauss_case(std::istream& input) {
  const int order = mucore_int(input), size = mucore_int(input);
  const size_t aSize = static_cast<size_t>(size);
  math_Vector p(aSize), w(aSize);
  auto aPrint = [&](bool ok) {
    std::cout << ok << ' ';
    for (size_t i = 0; i < aSize; ++i) mucore_out({p.At(i), w.At(i)});
    p.Init(-1.0); w.Init(-1.0);
  };
  p.Init(-1.0); w.Init(-1.0);
  aPrint(MathUtils::GetGaussPointsAndWeights(order, p, w));
  aPrint(MathUtils::GetKronrodPointsAndWeights(order, p, w));
  aPrint(MathUtils::GetOrderedGaussPointsAndWeights(order, p, w));
  std::cout << '\n';
}

// mathutils_core_random seed reseed
void mathutils_core_random_case(std::istream& input) {
  const double aSeed = mucore_get(input), aReseed = mucore_get(input);
  MathUtils::RandomGenerator g(static_cast<uint64_t>(aSeed));
  for (int i = 0; i < 3; ++i) std::cout << g.NextInt() << ' ';
  mucore_out({g.NextReal(), g.NextReal()});
  g.SetSeed(static_cast<uint64_t>(aReseed));
  std::cout << g.NextInt() << ' ';
  mucore_out({g.NextReal()});
  MathUtils::RandomGenerator aDefault;
  std::cout << aDefault.NextInt() << '\n';
}
