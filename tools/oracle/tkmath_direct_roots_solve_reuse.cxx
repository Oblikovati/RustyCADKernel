// Exposes OCCT's protected overloads to verify their in-place state transitions.
#include <cstdint>
#include <cstring>
#include <iostream>
#include <math_DirectPolynomialRoots.hxx>

struct DirectRootsProbe : math_DirectPolynomialRoots {
  using math_DirectPolynomialRoots::math_DirectPolynomialRoots;
  using math_DirectPolynomialRoots::Solve;
};

static std::uint64_t bits(double value) {
  std::uint64_t result;
  std::memcpy(&result, &value, sizeof(result));
  return result;
}

static void report(const char* label, const DirectRootsProbe& roots) {
  std::cout << label << ' ' << roots.IsDone() << ' ' << roots.InfiniteRoots();
  try {
    const int count = roots.NbSolutions();
    std::cout << ' ' << count;
    for (int index = 1; index <= count; ++index) std::cout << ' ' << bits(roots.Value(index));
  } catch (const Standard_Failure&) {
    std::cout << " failure";
  }
  std::cout << '\n';
}

static void quartic_sequence() {
  DirectRootsProbe quartic(1.0, 0.0, -5.0, 0.0, 4.0);
  quartic.Solve(1.0, 0.0, -5.0, 0.0, 4.0);
  report("quartic_success", quartic);
  quartic.Solve(1.0, 0.0, 1.0, 0.0, 1.0);
  report("quartic_no_real", quartic);
}

static void cubic_sequence() {
  DirectRootsProbe cubic(1.0, -6.0, 11.0, -6.0);
  cubic.Solve(1.0, -6.0, 11.0, -6.0);
  report("cubic_success", cubic);
  cubic.Solve(1.0, 1.0e60, 1.0, 1.0);
  report("cubic_overflow", cubic);
  cubic.Solve(1.0, -6.0, 11.0, -6.0);
  report("cubic_after_overflow", cubic);
}

static void quadratic_sequence() {
  DirectRootsProbe quadratic(1.0, -5.0, 6.0);
  quadratic.Solve(1.0, -5.0, 6.0);
  report("quadratic_success", quadratic);
  quadratic.Solve(1.0, 1.0, 1.0);
  report("quadratic_no_real", quadratic);
}

static void infinite_sequence() {
  DirectRootsProbe linear(1.0, -2.0);
  linear.Solve(0.0, 0.0);
  report("linear_infinite", linear);
  linear.Solve(1.0, -3.0);
  report("linear_after_infinite", linear);
  linear.Solve(1.0, -5.0, 6.0);
  report("quadratic_after_infinite", linear);
}

int main() {
  quartic_sequence();
  cubic_sequence();
  quadratic_sequence();
  infinite_sequence();
}
