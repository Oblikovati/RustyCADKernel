// MathRoot::Utils public helpers at pinned 3d097a0328e71b826377d4814ab05ec3c3d23871.
// Included after tkmath_modern_multiple_probe.cxx: reuses ModernMultipleFunction and its trace.
#include <MathRoot_All.hxx>
#include <MathRoot_Trig.hxx>
#include <MathRoot_Utils.hxx>
#include <cstdlib>

// strtod accepts the "nan"/"inf" tokens Python writes; operator>> does not.
static double mathroot_read(std::istream& input) {
  std::string token;
  input >> token;
  return std::strtod(token.c_str(), nullptr);
}

void mathroot_utils_case(std::istream& input) {
  const double a = mathroot_read(input), b = mathroot_read(input);
  const double c = mathroot_read(input), d = mathroot_read(input);
  int flag;
  size_t n;
  input >> flag >> n;
  namespace U = MathRoot::Utils;
  std::cout << U::IsValidBounds(a, b) << ' ' << U::HaveOppositeSigns(a, b) << ' '
            << bits(U::Midpoint(a, b)) << ' ';
  MathUtils::Config config;
  config.XTolerance = c;
  config.FTolerance = d;
  config.MaxIterations = static_cast<uint32_t>(n);
  std::cout << U::IsValidConfig(config) << ' ';
  size_t count = 77;
  const bool counted = U::ComputePeriodicRootCount(a, b, c, d, flag != 0, count);
  std::cout << counted << ' ' << count << ' ' << bits(U::EffectiveXTolerance(a, b, c)) << ' ';
  MathRoot::MultipleConfig multiple;
  multiple.NbSamples = n;
  multiple.XTolerance = c;
  multiple.FTolerance = d;
  multiple.NullTolerance = d;
  multiple.MaxIterations = static_cast<uint32_t>(n);
  multiple.Offset = a;
  std::cout << U::IsValidMultipleInput(a, b, multiple) << ' ';
  double mapped = 91.0;
  const bool isMapped = U::MapPeriodicTrigRoot(a, b, c, d, mapped);
  std::cout << isMapped << ' ' << bits(mapped) << std::endl;
}

void mathroot_trig_roots_case(std::istream& input) {
  const double lower = mathroot_read(input), upper = mathroot_read(input);
  const double tolerance = mathroot_read(input);
  size_t count;
  input >> count;
  MathRoot::TrigResult result;
  for (size_t i = 0; i < count; ++i) {
    const double root = mathroot_read(input);
    if (i % 2 == 0)
      MathRoot::Utils::AddPeriodicTrigRoot(result, root, lower, upper, tolerance);
    else
      MathRoot::Utils::AddDistinctTrigRoot(result, root, tolerance);
  }
  std::cout << result.NbRoots << ' ';
  for (size_t i = 0; i < result.NbRoots; ++i)
    std::cout << bits(result.Roots[i]) << ' ';
  std::cout << std::endl;
}

void mathroot_sort_case(std::istream& input) {
  const double eps = mathroot_read(input);
  size_t count;
  input >> count;
  MathRoot::MultipleResult result;
  for (size_t i = 0; i < count; ++i) {
    const double root = mathroot_read(input);
    const double value = mathroot_read(input);
    MathRoot::Utils::AddRoot(result, eps, root, value);
  }
  MathRoot::Utils::SortRoots(result);
  std::cout << result.NbRoots() << ' ';
  for (size_t i = 0; i < result.NbRoots(); ++i)
    std::cout << bits(result.Roots.Value(i)) << ' ' << bits(result.Values.Value(i)) << ' ';
  std::cout << std::endl;
}

static void mathroot_evaluations(ModernMultipleFunction& f, double x, double offset) {
  double value = 91.0, derivative = 92.0;
  const int plain = int(MathRoot::Utils::EvaluateValue(f, x, value));
  std::cout << plain << ' ' << bits(value) << ' ';
  const int shifted = int(MathRoot::Utils::EvaluateShiftedValue(f, x, offset, value));
  std::cout << shifted << ' ' << bits(value) << ' ';
  const int both = int(MathRoot::Utils::EvaluateShiftedValues(f, x, offset, value, derivative));
  std::cout << both << ' ' << bits(value) << ' ' << bits(derivative) << ' ';
  multiple_trace(f);
}

static void mathroot_adapters(ModernMultipleFunction& f, double x, double offset) {
  namespace U = MathRoot::Utils;
  math_Vector samples(3);
  samples.Init(93.0);
  U::MultipleSampleValueFn<ModernMultipleFunction> sample{f, samples, offset};
  const bool sampled = sample(1, x);
  U::MultipleGetValueFn lookup{samples};
  std::cout << sampled << ' ' << int(sample.Status) << ' ' << bits(lookup(1)) << ' ';
  U::MultipleGetRootValueFn<ModernMultipleFunction> rootValue{f};
  double value = 94.0;
  const bool valued = rootValue(x, value);
  std::cout << valued << ' ' << int(rootValue.Status) << ' ' << bits(value) << ' ';
  U::MultipleBrentValueWrapper<ModernMultipleFunction> brent{f, offset};
  value = 95.0;
  const bool shifted = brent.Value(x, value);
  std::cout << shifted << ' ' << bits(value) << ' ';
  U::MultipleDerivativeValueWrapper<ModernMultipleFunction> derivative{f};
  value = 96.0;
  const bool slope = derivative.Value(x, value);
  std::cout << slope << ' ' << bits(value) << ' ';
  multiple_trace(f);
}

void mathroot_search_case(std::istream& input) {
  int mode, fail, limit;
  size_t n;
  double target, lower, upper, xtol, ftol, ntol, offset;
  input >> mode >> target >> lower >> upper >> n >> xtol >> ftol >> ntol >> limit >> fail >> offset;
  ModernMultipleFunction f{mode, fail, target, {}};
  mathroot_evaluations(f, lower, offset);
  mathroot_adapters(f, upper, offset);
  double y1 = 0.0, y2 = 0.0;
  MathRoot::Utils::EvaluateShiftedValue(f, lower, offset, y1);
  MathRoot::Utils::EvaluateShiftedValue(f, upper, offset, y2);
  multiple_trace(f);
  MathRoot::MultipleResult refined;
  const int status = int(MathRoot::Utils::RefineBracketedRoot(
    f, offset, lower, y1, upper, y2, xtol, ftol, static_cast<uint32_t>(limit), xtol, refined));
  std::cout << status << ' ';
  multiple_result(refined, f);
  multiple_result(MathRoot::Utils::FindMultipleRoots(f, lower, upper, n, xtol, ftol, offset), f);
  MathRoot::MultipleConfig cfg;
  cfg.NbSamples = n;
  cfg.XTolerance = xtol;
  cfg.FTolerance = ftol;
  cfg.NullTolerance = ntol;
  cfg.MaxIterations = static_cast<uint32_t>(limit);
  cfg.Offset = offset;
  const size_t aNbSamples = std::max<size_t>(2 * n, 20);
  math_Vector samples(aNbSamples + 1);
  MathRoot::Utils::MultipleSampleValueFn<ModernMultipleFunction> sample{f, samples, offset};
  MathRoot::Utils::MultipleGetValueFn lookup{samples};
  MathRoot::Utils::MultipleBrentValueWrapper<ModernMultipleFunction> wrapper{f, offset};
  MathRoot::Utils::MultipleGetRootValueFn<ModernMultipleFunction> rootValue{f};
  multiple_result(MathRoot::Utils::FindAllRootsImpl(lower, upper, cfg, sample, lookup, wrapper,
                                                    rootValue, MathRoot::Utils::MultipleNoExtraHandler()),
                  f);
  multiple_result(MathRoot::Utils::FindAllRootsWithDerivativeImpl(f, lower, upper, cfg), f);
  std::cout << std::endl;
}
