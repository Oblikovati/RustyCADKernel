// MathLin::Utils QL helpers, MathLin_Utils, spectral wrappers and MathInteg constants/TanhSinhSingular
// at pinned 3d097a0328e71b826377d4814ab05ec3c3d23871. Included after tkmath_modern_integ_probe.cxx and
// tkmath_modern_eigen_probe.cxx (reuses ModernIntegrand, integ_result and modern_eigen_vectors).
#include <MathInteg_DoubleExp.hxx>
#include <MathInteg_Gauss.hxx>
#include <MathInteg_Kronrod.hxx>
#include <MathInteg_Multiple.hxx>
#include <MathInteg_Set.hxx>
#include <MathLin_EigenSearch.hxx>
#include <MathLin_Jacobi.hxx>
#include <MathLin_Utils.hxx>

// Applies the request's special value (mode 1..4) to one slot of a zero-based vector.
void integ_lin_special(math_Vector& v, int mode, size_t slot) {
 const double values[] = {0.0, std::numeric_limits<double>::quiet_NaN(), std::numeric_limits<double>::infinity(),
                          -std::numeric_limits<double>::infinity(), 0.0};
 if (mode >= 1 && mode <= 4 && slot < v.Size()) v.ChangeAt(slot) = mode == 4 ? -0.0 : values[mode];
}
void integ_lin_vector(const math_Vector& v) {for (size_t i = 0; i < v.Size(); ++i) std::cout << bits(v.At(i)) << ' ';}
void integ_lin_matrix(const math_Matrix& m) {
 for (size_t i = 0; i < m.RowSize(); ++i) for (size_t j = 0; j < m.ColSize(); ++j) std::cout << bits(m.At(i, j)) << ' ';
}

// Request: n mode slot shift_scale d[n] e[n]. Drives every MathLin::Utils QL helper directly.
void integ_lin_tridiag_case(std::istream& input) {
 int n, mode, slot; double shiftScale; input >> n >> mode >> slot >> shiftScale;
 math_Vector d(static_cast<size_t>(n)), e(static_cast<size_t>(n));
 for (int i = 0; i < n; ++i) input >> d.ChangeAt(i);
 for (int i = 0; i < n; ++i) input >> e.ChangeAt(i);
 integ_lin_special(slot % 2 == 0 ? d : e, mode, size_t(slot / 2));
 for (size_t start = 0; start < size_t(n); ++start) {
  const size_t end = MathLin::Utils::FindSubmatrixEnd(d, e, start, size_t(n));
  std::cout << end << ' ';
  if (end == start) continue;
  const double shift = MathLin::Utils::ComputeWilkinsonShift(d, e, start, end);
  std::cout << bits(shift) << ' ';
  math_Vector dd = d, ee = e; math_Matrix v(size_t(n), size_t(n), 0.0);
  for (size_t i = 0; i < size_t(n); ++i) v.ChangeAt(i, i) = 1.0;
  std::cout << MathLin::Utils::PerformQLStep(dd, ee, v, start, end, shift * shiftScale, size_t(n)) << ' ';
  integ_lin_vector(dd); integ_lin_vector(ee); integ_lin_matrix(v);
 }
 math_Vector sub(static_cast<size_t>(n)); for (int i = 0; i < n; ++i) sub.ChangeAt(i) = i + 1 < n ? e.At(i) : 0.0;
 auto r = MathLin::EigenTridiagonal(d, sub, 30);
 std::cout << int(r.Status) << ' ';
 if (r.EigenValues && r.EigenVectors) {
  std::cout << MathLin::Utils::ValidateEigenpairs(d, sub, *r.EigenValues, *r.EigenVectors) << ' ';
  math_Vector shifted = *r.EigenValues; shifted.ChangeAt(0) += shiftScale;
  std::cout << MathLin::Utils::ValidateEigenpairs(d, sub, shifted, *r.EigenVectors) << ' ';
  for (size_t k = 0; k < size_t(n); ++k) integ_lin_vector(MathLin::GetEigenVector(r, k));
 } else {
  math_Matrix identity(size_t(n), size_t(n), 0.0); for (size_t i = 0; i < size_t(n); ++i) identity.ChangeAt(i, i) = 1.0;
  std::cout << MathLin::Utils::ValidateEigenpairs(d, sub, d, identity) << ' ';
  // Without eigenvectors GetEigenVector returns an uninitialized one-element vector: size only.
  std::cout << MathLin::GetEigenVector(r, 0).Size() << ' ';
 }
 std::cout << std::endl;
}

// Special tolerance codes: 1 = NaN, 2 = +inf (stream extraction cannot read "nan"/"inf" text).
double integ_lin_code(int code, double value) {
 return code == 1 ? std::numeric_limits<double>::quiet_NaN() : code == 2 ? std::numeric_limits<double>::infinity() : value;
}

// Request: n mode slot tol tolcode dim values[n]. MathLin_Utils norm accumulation, tolerance floor, finiteness.
void integ_lin_norm_case(std::istream& input) {
 int n, mode, slot, code, dim; double tol; input >> n >> mode >> slot >> tol >> code >> dim;
 tol = integ_lin_code(code, tol);
 math_Vector values(static_cast<size_t>(n)); for (int i = 0; i < n; ++i) input >> values.ChangeAt(i);
 integ_lin_special(values, mode, size_t(slot));
 double scale = 0.0, sum = 1.0;
 for (size_t i = 0; i < values.Size(); ++i) {
  std::cout << MathLin::Utils::AccumulateNorm(values.At(i), scale, sum) << ' ' << bits(scale) << ' ' << bits(sum) << ' ';
  const std::optional<double> norm = MathLin::Utils::Norm(scale, sum);
  std::cout << norm.has_value() << ' ' << bits(norm.value_or(91.0)) << ' ';
 }
 const std::optional<double> raw = MathLin::Utils::Norm(values.At(0), n > 1 ? values.At(1) : 1.0);
 std::cout << raw.has_value() << ' ' << bits(raw.value_or(91.0)) << ' ';
 std::cout << bits(MathLin::Utils::RelativeTolerance(tol, size_t(dim))) << ' ';
 math_Matrix m(size_t(1), values.Size(), 0.0); for (size_t i = 0; i < values.Size(); ++i) m.ChangeAt(0, i) = values.At(i);
 std::cout << MathLin::Utils::IsFinite(values) << ' ' << MathLin::Utils::IsFinite(m) << std::endl;
}

// Request: n lower mode entries[n*n]. EigenValues/SpectralDecomposition/MatrixSqrt/MatrixInvSqrt.
void integ_lin_spectral_case(std::istream& input) {
 int n, lower, mode; input >> n >> lower >> mode;
 math_Matrix a(lower, lower + n - 1, lower + 3, lower + n + 2, 0.0);
 for (int i = 0; i < n; ++i) for (int j = 0; j < n; ++j) input >> a.ChangeAt(size_t(i), size_t(j));
 if (mode == 1) a.ChangeAt(0, 0) = std::numeric_limits<double>::quiet_NaN();
 for (bool sort : {false, true}) {
  auto r = MathLin::EigenValues(a, sort); std::cout << int(r.Status) << ' ' << r.NbIterations << ' ';
  modern_eigen_vectors(r.EigenValues, r.EigenVectors);
 }
 auto s = MathLin::SpectralDecomposition(a); std::cout << int(s.Status) << ' ' << s.NbIterations << ' ';
 modern_eigen_vectors(s.EigenValues, s.EigenVectors);
 modern_eigen_vectors({}, MathLin::MatrixSqrt(a));
 modern_eigen_vectors({}, MathLin::MatrixInvSqrt(a));
 std::cout << std::endl;
}

// Request: mode lower upper uppercode tolerance fail. TanhSinhSingular plus every MathInteg/MathLin constant.
void integ_lin_singular_case(std::istream& input) {
 int mode, fail, code; double lower, upper, tolerance; input >> mode >> lower >> upper >> code >> tolerance >> fail;
 upper = integ_lin_code(code, upper);
 ModernIntegrand f{mode, fail, {}};
 integ_result(MathInteg::TanhSinhSingular(f, lower, upper, tolerance), f);
 integ_result(MathInteg::TanhSinhSingular(f, lower, upper), f);
 std::cout << MathInteg::THE_GAUSS_MAX_ORDER << ' ' << MathInteg::THE_SET_GAUSS_MAX_ORDER << ' '
           << MathInteg::THE_MULTIPLE_GAUSS_MAX_ORDER << ' ' << MathInteg::THE_KRONROD_MAX_GAUSS_ORDER << ' '
           << MathInteg::THE_KRONROD_AUTO_INITIAL_ORDER << ' ' << MathInteg::THE_KRONROD_AUTO_ORDER_STEP << ' '
           << MathInteg::THE_KRONROD_AUTO_MAX_ITERATIONS << ' ' << MathInteg::THE_DOUBLE_EXP_DEFAULT_MAX_POINTS_PER_LEVEL << ' '
           << bits(MathInteg::THE_DOUBLE_EXP_MAX_TRANSFORM_ARGUMENT) << ' ' << bits(MathInteg::THE_EXP_SINH_MIN_TRANSFORM_ARGUMENT) << ' '
           << bits(MathLin::Utils::THE_TRIDIAGONAL_DEFLATION_TOLERANCE) << ' '
           << bits(MathLin::Utils::THE_TRIDIAGONAL_RESIDUAL_TOLERANCE_FACTOR) << ' ';
 MathInteg::KronrodConfig kronrod; MathInteg::DoubleExpConfig doubleExp; MathInteg::MultipleConfig multiple;
 std::cout << kronrod.NbGaussPoints << ' ' << kronrod.Adaptive << ' ' << bits(kronrod.Tolerance) << ' ' << kronrod.MaxIterations << ' '
           << doubleExp.NbLevels << ' ' << bits(doubleExp.StepFactor) << ' ' << doubleExp.MaxPointsPerLevel << ' '
           << multiple.MaxOrder << std::endl;
}
