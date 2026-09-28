// MathOpt::Utils, PolishCoordinateWise and modern Uzawa oracle at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathOpt_Utils.hxx>
#include <MathOpt_Newton.hxx>
#include <MathOpt_FRPR.hxx>
#include <MathOpt_PSO.hxx>
#include <MathOpt_Uzawa.hxx>
// Sentinels carry NaN and infinities through the text corpus (operator>> rejects "nan"/"inf").
double mo_decode(double value) {
  if (value == 7777.25) return std::numeric_limits<double>::quiet_NaN();
  if (value == 8888.25) return std::numeric_limits<double>::infinity();
  if (value == -8888.25) return -std::numeric_limits<double>::infinity();
  return value;
}
double mo_read(std::istream& input) {double value;input>>value;return mo_decode(value);}
size_t mo_count(std::istream& input) {double value;input>>value;return size_t(value);}
math_Vector mo_vector(std::istream& input, size_t n, int lower) {
  math_Vector result(lower, lower + int(n) - 1);
  for (size_t i = 0; i < n; ++i) result.ChangeAt(i) = mo_read(input);
  return result;
}
void mo_vec_out(const math_Vector& v) {for (size_t i = 0; i < v.Size(); ++i) std::cout << bits(v.At(i)) << ' ';}
void mo_mat_out(const math_Matrix& m) {
  std::cout << m.RowSize() << ' ' << m.ColSize() << ' ';
  for (size_t i = 0; i < m.RowSize(); ++i) for (size_t j = 0; j < m.ColSize(); ++j) std::cout << bits(m.At(i, j)) << ' ';
}
// Quadratic bowl with an x0*x1 coupling; `fail` refuses the fail-th callback of any kind and
// modes inject NaN/infinite values, gradients and Hessians.
struct MathOptApiObjective {
  int mode; size_t fail; double center, coupling; std::vector<double> calls;
  bool Record(double kind, const math_Vector& x) {
    calls.push_back(kind); for (size_t i = 0; i < x.Size(); ++i) calls.push_back(x.At(i));
    return ++count != fail;
  }
  size_t count = 0;
  bool Value(const math_Vector& x, double& f) {
    if (!Record(0, x)) return false;
    f = 0.0;
    for (size_t i = 0; i < x.Size(); ++i) {const double d = x.At(i) - center; f += d * d;}
    if (x.Size() >= 2) f += coupling * x.At(0) * x.At(1);
    if (mode == 1) f = std::numeric_limits<double>::quiet_NaN();
    if (mode == 2 && x.At(0) > center) f = std::numeric_limits<double>::infinity();
    return true;
  }
  bool Value(double x, double& f) {
    calls.push_back(3); calls.push_back(x);
    if (++count == fail) return false;
    f = (x - center) * (x - center);
    if (mode == 1) f = std::numeric_limits<double>::quiet_NaN();
    return true;
  }
  bool Gradient(const math_Vector& x, math_Vector& g) {
    if (!Record(1, x)) return false;
    for (size_t i = 0; i < x.Size(); ++i) g.ChangeAt(i) = 2.0 * (x.At(i) - center);
    if (x.Size() >= 2) {g.ChangeAt(0) += coupling * x.At(1); g.ChangeAt(1) += coupling * x.At(0);}
    if (mode == 3) g.ChangeAt(0) = std::numeric_limits<double>::quiet_NaN();
    return true;
  }
  bool Hessian(const math_Vector& x, math_Matrix& h) {
    if (!Record(2, x)) return false;
    for (size_t i = 0; i < h.RowSize(); ++i) for (size_t j = 0; j < h.ColSize(); ++j) h.ChangeAt(i, j) = i == j ? 2.0 : coupling;
    if (mode == 4) h.ChangeAt(0, 0) = std::numeric_limits<double>::infinity();
    return true;
  }
  void Flush() {std::cout << calls.size() << ' '; for (double c : calls) std::cout << bits(c) << ' '; calls.clear();}
};
MathOptApiObjective mo_objective(std::istream& input) {
  MathOptApiObjective f{0, 0, 0.0, 0.0, {}};
  f.mode = int(mo_count(input)); f.fail = mo_count(input); f.center = mo_read(input); f.coupling = mo_read(input);
  return f;
}
void mathopt_checks_case(std::istream& input) {
  MathUtils::Config config;
  config.MaxIterations = uint32_t(mo_count(input)); config.Tolerance = mo_read(input);
  config.XTolerance = mo_read(input); config.FTolerance = mo_read(input);
  config.RelativeTolerance = mo_read(input); config.StepMin = mo_read(input);
  const size_t n = mo_count(input);
  const math_Vector a = mo_vector(input, n, 3), b = mo_vector(input, n, -2);
  math_Matrix m(2, 3, -1, 0);
  for (size_t i = 0; i < 2; ++i) for (size_t j = 0; j < 2; ++j) m.ChangeAt(i, j) = mo_read(input);
  math_Vector longer(0, int(n)); longer.Init(1.0);
  std::cout << MathOpt::Utils::IsValidConfig(config) << ' ' << MathOpt::Utils::IsFinite(a) << ' '
            << MathOpt::Utils::IsFinite(m) << ' ' << MathOpt::Utils::IsValidBounds(a, b) << ' '
            << MathOpt::Utils::IsValidBounds(a, longer) << ' ' << bits(MathOpt::Utils::Norm(a)) << ' '
            << bits(MathOpt::Utils::Norm(b)) << '\n';
}
void mathopt_line_case(std::istream& input) {
  MathUtils::LineSearchResult first, second;
  first.Status = MathUtils::Status(mo_count(input)); second.Status = MathUtils::Status(mo_count(input));
  std::cout << MathOpt::Utils::IsLineSearchEvaluationError(first) << ' '
            << int(MathOpt::Utils::LineSearchFailureStatus(first)) << ' '
            << int(MathOpt::Utils::LineSearchFailureStatus(first, second)) << ' '
            << bits(MathOpt::Utils::THE_FRPR_WOLFE_CURVATURE) << '\n';
}
void mathopt_eval_case(std::istream& input) {
  MathOptApiObjective f = mo_objective(input);
  const size_t n = mo_count(input);
  const math_Vector x = mo_vector(input, n, 1), lower = mo_vector(input, n, 0), upper = mo_vector(input, n, 5);
  double value = 91; math_Vector g(2, 1 + int(n)); g.Init(91); math_Matrix h(1, int(n), 1, int(n), 91.0);
  std::cout << int(MathOpt::Utils::ValueStatus(f, x.At(0), value)) << ' ' << bits(value) << ' ';
  std::cout << int(MathOpt::Utils::ValueStatus(f, x, value)) << ' ' << bits(value) << ' ';
  std::cout << MathOpt::Utils::Value(f, x, value) << ' ' << bits(value) << ' ';
  std::cout << int(MathOpt::Utils::GradientStatus(f, x, g)) << ' '; mo_vec_out(g);
  std::cout << MathOpt::Utils::Gradient(f, x, g) << ' ';
  std::cout << int(MathOpt::Utils::HessianStatus(f, x, h)) << ' '; mo_mat_out(h);
  std::cout << MathOpt::Utils::Hessian(f, x, h) << ' ';
  MathOpt::Utils::CheckedFunction<MathOptApiObjective> checked(f);
  math_Vector bad(1, int(n)); bad.Init(std::numeric_limits<double>::quiet_NaN());
  std::cout << checked.Value(x, value) << ' ' << checked.Value(bad, value) << ' ' << checked.Value(x, value) << ' '
            << checked.HadFailure << ' ' << checked.HasValidEvaluation << ' ' << int(checked.FailureStatus) << ' '
            << checked.Original().Value(x, value) << ' ';
  MathOpt::Utils::BoundedFunction<MathOptApiObjective> bounded(f, lower, upper);
  std::cout << bounded.Value(x, value) << ' ' << bits(value) << ' ';
  f.Flush(); std::cout << '\n';
}
void mathopt_stationary_case(std::istream& input) {
  MathOptApiObjective f = mo_objective(input);
  const size_t n = mo_count(input);
  const math_Vector x = mo_vector(input, n, 2);
  const double fx = mo_read(input);
  MathUtils::Config config; config.XTolerance = mo_read(input); config.FTolerance = mo_read(input);
  MathUtils::Status status = MathUtils::Status::MaxIterations;
  std::cout << MathOpt::Utils::IsStationary(f, x, fx, config, status) << ' ' << int(status) << ' ';
  f.Flush(); std::cout << '\n';
}
void mathopt_backtrack_case(std::istream& input) {
  MathOptApiObjective f = mo_objective(input);
  const size_t n = mo_count(input);
  const math_Vector x = mo_vector(input, n, 1), d = mo_vector(input, n, 1), g = mo_vector(input, n, 1);
  const double fx = mo_read(input), step = mo_read(input), stepMin = mo_read(input);
  const MathUtils::LineSearchResult r = MathOpt::Utils::Backtrack(f, x, d, g, fx, step, stepMin);
  std::cout << int(r.Status) << ' ' << r.IsValid << ' ' << bits(r.Alpha) << ' ' << bits(r.FNew) << ' ' << r.NbEvals << ' ';
  f.Flush(); std::cout << '\n';
}
void mathopt_regularize_case(std::istream& input) {
  const size_t n = mo_count(input);
  math_Matrix h(1, int(n), 0, int(n) - 1);
  for (size_t i = 0; i < n; ++i) for (size_t j = 0; j < n; ++j) h.ChangeAt(i, j) = mo_read(input);
  const double minimum = mo_read(input);
  std::cout << MathOpt::Utils::RegularizeHessian(h, minimum) << ' '; mo_mat_out(h); std::cout << '\n';
}
void mathopt_polish_case(std::istream& input) {
  MathOptApiObjective f = mo_objective(input);
  const size_t n = mo_count(input);
  math_Vector x = mo_vector(input, n, 1);
  const math_Vector lower = mo_vector(input, n, 1), upper = mo_vector(input, n, 1);
  double value = mo_read(input); const double tolerance = mo_read(input); const size_t budget = mo_count(input);
  size_t evaluations = 91;
  MathOpt::PolishCoordinateWise(f, x, value, lower, upper, tolerance, budget, evaluations);
  mo_vec_out(x); std::cout << bits(value) << ' ' << evaluations << ' '; f.Flush(); std::cout << '\n';
}
void mo_optional_out(const std::optional<math_Vector>& v) {std::cout << v.has_value() << ' '; if (v) {std::cout << v->Size() << ' '; mo_vec_out(*v);}}
void mathopt_uzawa_case(std::istream& input) {
  const size_t rows = mo_count(input), cols = mo_count(input), nce = mo_count(input), nci = mo_count(input);
  MathOpt::UzawaConfig config; config.EpsLix = mo_read(input); config.EpsLic = mo_read(input);
  config.MaxIterations = uint32_t(mo_count(input)); const bool equality = mo_count(input) != 0;
  math_Matrix c(3, 2 + int(rows), -1, int(cols) - 2);
  for (size_t i = 0; i < rows; ++i) for (size_t j = 0; j < cols; ++j) c.ChangeAt(i, j) = mo_read(input);
  const math_Vector s = mo_vector(input, rows, 2), x0 = mo_vector(input, cols, -1);
  const MathOpt::UzawaResult r = equality ? MathOpt::UzawaEquality(c, s, x0, config) : MathOpt::Uzawa(c, s, x0, nce, nci, config);
  std::cout << int(r.Status) << ' ' << r.IsDone() << ' ' << bool(r) << ' ' << r.NbIterations << ' ';
  mo_optional_out(r.Solution); mo_optional_out(r.Dual); mo_optional_out(r.Error); mo_optional_out(r.InitialError);
  std::cout << r.InverseCTC.has_value() << ' '; if (r.InverseCTC) mo_mat_out(*r.InverseCTC);
  std::cout << '\n';
}
