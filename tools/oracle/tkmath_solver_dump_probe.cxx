// Solver Dump text, math_Status bookkeeping and IsSolutionReached/IsConverged overrides.
#include <math_BFGS.hxx>
#include <math_FRPR.hxx>
#include <math_Powell.hxx>
#include <math_NewtonMinimum.hxx>
#include <math_BrentMinimum.hxx>
#include <math_BissecNewton.hxx>
#include <math_BracketMinimum.hxx>
#include <math_BracketedRoot.hxx>
#include <math_FunctionRoot.hxx>
#include <math_NewtonFunctionRoot.hxx>
#include <math_FunctionRoots.hxx>
#include <math_FunctionAllRoots.hxx>
#include <math_FunctionSample.hxx>
#include <math_DirectPolynomialRoots.hxx>
#include <math_TrigonometricFunctionRoots.hxx>
#include <math_GaussSingleIntegration.hxx>
#include <math_KronrodSingleIntegration.hxx>
#include <math_GaussSetIntegration.hxx>
#include <math_FunctionSetRoot.hxx>
#include <math_NewtonFunctionSetRoot.hxx>
#include <math_Householder.hxx>
#include <math_Crout.hxx>
#include <math_Uzawa.hxx>
#include <math_ValueAndWeight.hxx>
#include <math_PSOParticlesPool.hxx>
#include <math_ComputeGaussPointsAndWeights.hxx>
#include <math_ComputeKronrodPointsAndWeights.hxx>
#include <math_FunctionSetWithDerivatives.hxx>
#include <math_MultipleVarFunctionWithHessian.hxx>
#include <algorithm>
namespace solver_dump_probe {
using recipes_probe::dump_call;
using vector_base_probe::guarded;
using vector_base_probe::reals;
// Separable quadratic (optionally quartic) objective; call `fail` returns false.
struct Bowl : math_MultipleVarFunctionWithHessian {
  int n, quartic, fail, calls = 0;
  std::vector<double> centers, weights;
  int NbVariables() const override { return n; }
  bool Count() { return ++calls != fail; }
  double Eval(const math_Vector& x) const {
    double r = 0;
    for (int i = 1; i <= n; ++i) { double d = x(i) - centers[i - 1]; r += weights[i - 1] * d * d + (quartic ? 0.1 * d * d * d * d : 0.0); }
    return r;
  }
  void Grad(const math_Vector& x, math_Vector& g) const {
    for (int i = 1; i <= n; ++i) { double d = x(i) - centers[i - 1]; g(i) = 2.0 * weights[i - 1] * d + (quartic ? 0.4 * d * d * d : 0.0); }
  }
  bool Value(const math_Vector& x, double& v) override { v = Eval(x); return Count(); }
  bool Gradient(const math_Vector& x, math_Vector& g) override { Grad(x, g); return Count(); }
  bool Values(const math_Vector& x, double& v, math_Vector& g) override { v = Eval(x); Grad(x, g); return Count(); }
  bool Values(const math_Vector& x, double& v, math_Vector& g, math_Matrix& h) override {
    v = Eval(x); Grad(x, g); h.Init(0.0);
    for (int i = 1; i <= n; ++i) { double d = x(i) - centers[i - 1]; h(i, i) = 2.0 * weights[i - 1] + (quartic ? 1.2 * d * d : 0.0); }
    return Count();
  }
};
// Overrides report convergence on the `cutoff`-th test and count every test.
struct CutBFGS : math_BFGS {
  int cutoff; mutable int tests = 0;
  CutBFGS(int n, double tol, int limit, int cut) : math_BFGS(n, tol, limit), cutoff(cut) {}
  bool IsSolutionReached(math_MultipleVarFunctionWithGradient&) const override { return ++tests >= cutoff; }
};
struct CutFRPR : math_FRPR {
  int cutoff, tests = 0;
  CutFRPR(const math_MultipleVarFunctionWithGradient& f, double tol, int limit, int cut) : math_FRPR(f, tol, limit), cutoff(cut) {}
  bool IsSolutionReached(math_MultipleVarFunctionWithGradient&) override { return ++tests >= cutoff; }
};
struct CutPowell : math_Powell {
  int cutoff, tests = 0;
  CutPowell(const math_MultipleVarFunction& f, double tol, int limit, int cut) : math_Powell(f, tol, limit), cutoff(cut) {}
  bool IsSolutionReached(math_MultipleVarFunction&) override { return ++tests >= cutoff; }
};
struct CutNewton : math_NewtonMinimum {
  int cutoff; mutable int tests = 0;
  CutNewton(const math_MultipleVarFunctionWithHessian& f, double tol, int limit, int cut) : math_NewtonMinimum(f, tol, limit), cutoff(cut) {}
  bool IsConverged() const override { return ++tests >= cutoff; }
};
template<class S> void minimizer_tail(const S& solver, int tests) {
  dump_call(solver); std::cout << tests << ' ';
}
void newton_dump(const math_NewtonMinimum& solver) {
  std::ostringstream text;
  try { solver.Dump(text); } catch (const Standard_Failure& e) { text << "#" << e.ExceptionType(); }
  std::string s = text.str(); for (char& ch : s) if (ch == '\n') ch = '|';
  std::cout << s << ' ' << int(solver.GetStatus()) << ' ';
}
// n quartic fail limit tol cutoff centers[n] weights[n] start[n]
void minimizers_case(std::istream& in) {
  Bowl f; int limit, cutoff; double tol; in >> f.n >> f.quartic >> f.fail >> limit >> tol >> cutoff;
  f.centers.resize(f.n); f.weights.resize(f.n); for (double& v : f.centers) in >> v; for (double& v : f.weights) in >> v;
  math_Vector start(1, f.n); for (int i = 1; i <= f.n; ++i) in >> start(i);
  math_Matrix directions(1, f.n, 1, f.n, 0.0); for (int i = 1; i <= f.n; ++i) directions(i, i) = 1.0;
  { f.calls = 0; math_BFGS s(f.n, tol, limit); s.Perform(f, start); dump_call(s); }
  { f.calls = 0; CutBFGS s(f.n, tol, limit, cutoff); s.Perform(f, start); minimizer_tail(s, s.tests); }
  { f.calls = 0; math_FRPR s(f, tol, limit); s.Perform(f, start); dump_call(s); }
  { f.calls = 0; CutFRPR s(f, tol, limit, cutoff); s.Perform(f, start); minimizer_tail(s, s.tests); }
  { f.calls = 0; math_Powell s(f, tol, limit); s.Perform(f, start, directions); dump_call(s); }
  { f.calls = 0; CutPowell s(f, tol, limit, cutoff); s.Perform(f, start, directions); minimizer_tail(s, s.tests); }
  { f.calls = 0; math_NewtonMinimum s(f, tol, limit); s.Perform(f, start); newton_dump(s); }
  { f.calls = 0; CutNewton s(f, tol, limit, cutoff); s.Perform(f, start); newton_dump(s); std::cout << s.tests << ' '; }
  std::cout << '\n';
}
// f(x) = x^3 - c with derivative; g(x) = (x - c)^2 + s as a plain function.
struct Cubic : math_FunctionWithDerivative {
  double c; int fail, calls = 0;
  bool Value(double x, double& v) override { v = x * x * x - c; return ++calls != fail; }
  bool Derivative(double x, double& d) override { d = 3 * x * x; return ++calls != fail; }
  bool Values(double x, double& v, double& d) override { v = x * x * x - c; d = 3 * x * x; return ++calls != fail; }
};
struct Parabola : math_Function {
  double c, s; int fail, calls = 0;
  bool Value(double x, double& v) override { v = (x - c) * (x - c) + s; return ++calls != fail; }
};
struct CutBrent : math_BrentMinimum {
  int cutoff, tests = 0;
  CutBrent(double tol, int cut) : math_BrentMinimum(tol), cutoff(cut) {}
  bool IsSolutionReached(math_Function&) override { return ++tests >= cutoff; }
};
struct CutBissec : math_BissecNewton {
  int cutoff, tests = 0;
  CutBissec(double tol, int cut) : math_BissecNewton(tol), cutoff(cut) {}
  bool IsSolutionReached(math_FunctionWithDerivative&) override { return ++tests >= cutoff; }
};
// c s a b fail cutoff
void scalar_case(std::istream& in) {
  double c, s, a, b; int fail, cutoff; in >> c >> s >> a >> b >> fail >> cutoff;
  Cubic cubic; cubic.c = c; cubic.fail = fail; Parabola par; par.c = c; par.s = s; par.fail = fail;
  const double m = 0.5 * (a + b);
  { math_BrentMinimum r(1e-10); r.Perform(par, a, m, b); dump_call(r); }
  { par.calls = 0; CutBrent r(1e-10, cutoff); r.Perform(par, a, m, b); dump_call(r); std::cout << r.tests << ' '; }
  { math_BissecNewton r(1e-10); r.Perform(cubic, a, b, 50); dump_call(r); }
  { cubic.calls = 0; CutBissec r(1e-10, cutoff); r.Perform(cubic, a, b, 50); dump_call(r); std::cout << r.tests << ' '; }
  // math_FunctionSetRoot (behind math_FunctionRoot) ignores a failed F.Value(tsol, fsol) and
  // reads the uninitialized fsol, so the remaining solvers run on callbacks that never fail.
  cubic.fail = 0; par.fail = 0;
  { par.calls = 0; math_BracketMinimum r(par, a, b); dump_call(r); }
  { cubic.calls = 0; math_BracketedRoot r(cubic, a, b, 1e-10); dump_call(r); }
  { cubic.calls = 0; math_FunctionRoot r(cubic, m, 1e-10); dump_call(r); }
  { cubic.calls = 0; math_NewtonFunctionRoot r(cubic, m, 1e-10, 1e-10); dump_call(r); }
  { cubic.calls = 0; math_FunctionRoots r(cubic, a, b, 10); dump_call(r); }
  { cubic.calls = 0; math_FunctionSample sample(a, b, 10); math_FunctionAllRoots r(cubic, sample, 1e-8, 1e-8, 1e-8); dump_call(r); }
  { par.calls = 0; math_GaussSingleIntegration r(par, a, b, 5); dump_call(r); math_GaussSingleIntegration empty; dump_call(empty); }
  { math_KronrodSingleIntegration empty; std::cout << empty.IsDone() << ' '; }
  std::cout << '\n';
}
// a b c d e lower upper
void polynomial_dump_case(std::istream& in) {
  double a, b, c, d, e, lo, hi; in >> a >> b >> c >> d >> e >> lo >> hi;
  dump_call(math_DirectPolynomialRoots(a, b, c, d, e)); dump_call(math_DirectPolynomialRoots(b, c, d, e));
  dump_call(math_DirectPolynomialRoots(c, d, e)); dump_call(math_DirectPolynomialRoots(d, e));
  std::ostringstream stream; stream << math_DirectPolynomialRoots(c, d, e); std::string s = stream.str();
  for (char& ch : s) if (ch == '\n') ch = '|'; std::cout << s << ' ';
  dump_call(math_TrigonometricFunctionRoots(a, b, c, d, e, lo, hi)); dump_call(math_TrigonometricFunctionRoots(d, e, lo, hi));
  std::cout << '\n';
}
// F_i(x) = x_i^2 - c_i + coupling * x_{i+1}; n limit tol fail cutoff c[n] start[n]
struct Squares : math_FunctionSetWithDerivatives {
  int n, fail, calls = 0; double coupling; std::vector<double> c;
  int NbVariables() const override { return n; }
  int NbEquations() const override { return n; }
  void F(const math_Vector& x, math_Vector& r) const {
    for (int i = 1; i <= n; ++i) r(i) = x(i) * x(i) - c[i - 1] + (i < n ? coupling * x(i + 1) : 0.0);
  }
  void J(const math_Vector& x, math_Matrix& m) const {
    m.Init(0.0); for (int i = 1; i <= n; ++i) { m(i, i) = 2 * x(i); if (i < n) m(i, i + 1) = coupling; }
  }
  bool Value(const math_Vector& x, math_Vector& r) override { F(x, r); return ++calls != fail; }
  bool Derivatives(const math_Vector& x, math_Matrix& m) override { J(x, m); return ++calls != fail; }
  bool Values(const math_Vector& x, math_Vector& r, math_Matrix& m) override { F(x, r); J(x, m); return ++calls != fail; }
};
struct CutSetRoot : math_FunctionSetRoot {
  int cutoff, tests = 0;
  CutSetRoot(math_FunctionSetWithDerivatives& f, const math_Vector& tol, int limit, int cut) : math_FunctionSetRoot(f, tol, limit), cutoff(cut) {}
  bool IsSolutionReached(math_FunctionSetWithDerivatives&) override { return ++tests >= cutoff; }
};
struct CutNewtonSet : math_NewtonFunctionSetRoot {
  int cutoff, tests = 0;
  CutNewtonSet(math_FunctionSetWithDerivatives& f, const math_Vector& tol, double ftol, int limit, int cut) : math_NewtonFunctionSetRoot(f, tol, ftol, limit), cutoff(cut) {}
  bool IsSolutionReached(math_FunctionSetWithDerivatives&) override { return ++tests >= cutoff; }
};
// One variable, two equations: [t^2 - c, c*t]; variables != 1 raises NotImplemented.
struct Curve : math_FunctionSet {
  int vars; double c; int fail, calls = 0;
  int NbVariables() const override { return vars; }
  int NbEquations() const override { return 2; }
  bool Value(const math_Vector& x, math_Vector& r) override { r(1) = x(1) * x(1) - c; r(2) = c * x(1); return ++calls != fail; }
};
void sets_case(std::istream& in) {
  Squares f; int limit, cutoff; double tol; in >> f.n >> limit >> tol >> f.fail >> cutoff >> f.coupling;
  f.c.resize(f.n); for (double& v : f.c) in >> v; math_Vector start(1, f.n); for (int i = 1; i <= f.n; ++i) in >> start(i);
  math_Vector tolerances(1, f.n, tol);
  // A failed callback inside math_FunctionSetRoot's line search leaves fsol uninitialized
  // natively (undefined behaviour), so the general solver runs without injected failures.
  const int fail = f.fail; f.fail = 0;
  { f.calls = 0; math_FunctionSetRoot r(f, tolerances, limit); r.Perform(f, start); dump_call(r); std::ostringstream s; s << r; std::cout << s.str().size() << ' '; }
  { f.calls = 0; CutSetRoot r(f, tolerances, limit, cutoff); r.Perform(f, start); dump_call(r); std::cout << r.tests << ' '; }
  f.fail = fail;
  { f.calls = 0; math_NewtonFunctionSetRoot r(f, tolerances, tol, limit); r.Perform(f, start); dump_call(r); }
  { f.calls = 0; CutNewtonSet r(f, tolerances, tol, limit, cutoff); r.Perform(f, start); dump_call(r); std::cout << r.tests << ' '; }
  Curve curve; curve.vars = 1; curve.c = f.c[0]; curve.fail = f.fail;
  math_Vector lo(1, 1, start(1)), hi(1, 1, start(1) + 2.0); math_IntegerVector order(1, 1, limit % 12);
  { math_GaussSetIntegration r(curve, lo, hi, order); dump_call(r); if (r.IsDone()) reals(r.Value()); }
  curve.vars = 2; guarded([&] { math_GaussSetIntegration r(curve, lo, hi, order); });
  order(1) = 99; curve.vars = 1; guarded([&] { math_GaussSetIntegration r(curve, lo, hi, order); });
  math_Matrix a(1, f.n, 1, f.n); for (int i = 1; i <= f.n; ++i) for (int j = 1; j <= f.n; ++j) a(i, j) = i == j ? 2.0 + f.c[i - 1] : 0.5 * f.coupling;
  math_Vector rhs(1, f.n); for (int i = 1; i <= f.n; ++i) rhs(i) = f.c[i - 1];
  dump_call(math_Householder(a, rhs, 1e-20)); dump_call(math_Crout(a, 1e-20));
  math_Matrix cont(1, 1, 1, f.n, 1.0); math_Vector sec(1, 1, f.c[0]);
  dump_call(math_Uzawa(cont, sec, start, 1e-6, 1e-6, limit));
  std::cout << '\n';
}
// values[n] weights[n]; also PSO particles and quadrature orders.
void small_types_case(std::istream& in) {
  int n; in >> n; std::vector<math_ValueAndWeight> nodes;
  for (int i = 0; i < n; ++i) { double v, w; in >> v >> w; nodes.emplace_back(v, w); }
  std::cout << (nodes[0] < nodes[n - 1]) << ' ' << (nodes[n - 1] < nodes[0]) << ' ';
  std::sort(nodes.begin(), nodes.end());
  for (const auto& node : nodes) std::cout << bits(node.Value()) << ' ' << bits(node.Weight()) << ' ';
  math_ValueAndWeight zero; std::cout << bits(zero.Value()) << ' ' << bits(zero.Weight()) << ' ';
  PSO_Particle p, q; q.Distance = -1.0; std::cout << bits(p.Distance) << ' ' << bits(p.BestDistance) << ' ' << (q < p) << ' ' << (p < q) << ' ';
  guarded([&] { math_PSOParticlesPool pool(0, 2); std::cout << "empty "; });
  guarded([&] { math_PSOParticlesPool pool(2, 0); std::cout << "flat "; });
  for (int order : {0, 1, 2, 3}) {
    math_ComputeGaussPointsAndWeights g(order); std::cout << g.IsDone() << ' ' << g.Points().Length() << ' ';
    if (order == 0) continue;
    math_ComputeKronrodPointsAndWeights k(order); std::cout << k.IsDone() << ' ' << k.Points().Length() << ' ';
    if (k.IsDone()) reals(k.Points());
  }
  std::cout << '\n';
}
}
