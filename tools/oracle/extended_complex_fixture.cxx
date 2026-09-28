// Linux/GCC std::complex<long double> oracle (libgcc __mulxc3/__divxc3, glibc hypotl).
// Build: c++ -std=c++17 -O2 extended_complex_fixture.cxx -o fixture
// Output per line: a b c d |a+ib| re((a+ib)(c+id)) im(..) re((a+ib)/(c+id)) im(..),
// each as x87 exponent word and significand in hex.
#include <complex>
#include <cstdint>
#include <cstring>
#include <iostream>
long double make(std::uint16_t exponent, std::uint64_t significand) {
  unsigned char bytes[sizeof(long double)] = {};
  std::memcpy(bytes, &significand, 8);
  std::memcpy(bytes + 8, &exponent, 2);
  long double value;
  std::memcpy(&value, bytes, sizeof(value));
  return value;
}
void emit(long double value) {
  unsigned char bytes[sizeof(long double)];
  std::memcpy(bytes, &value, sizeof(value));
  std::uint64_t significand;
  std::uint16_t exponent;
  std::memcpy(&significand, bytes, 8);
  std::memcpy(&exponent, bytes + 8, 2);
  std::cout << std::hex << exponent << ' ' << significand << ' ';
}
std::uint64_t next(std::uint64_t& seed) {
  seed = seed * 6364136223846793005ULL + 1442695040888963407ULL;
  return seed;
}
// Mostly moderate exponents so products stay meaningful, plus zeros, subnormals,
// extremes, infinities and NaNs with a random sign.
long double sample(std::uint64_t& seed) {
  const std::uint64_t r = next(seed);
  const std::uint16_t sign = (r >> 7) & 1 ? 0x8000 : 0;
  const std::uint64_t significand = next(seed) | (1ULL << 63);
  switch (r % 16) {
    case 0: return make(sign, 0);
    case 1: return make(sign, significand >> 1);
    case 2: return make(sign | 0x7fff, 1ULL << 63);
    case 3: return make(sign | 0x7fff, 3ULL << 62);
    case 4: return make(sign | std::uint16_t(0x7ff0 + (r >> 20) % 15), significand);
    case 5: return make(sign | std::uint16_t(1 + (r >> 20) % 64), significand);
    default: return make(sign | std::uint16_t(16383 - 40 + (r >> 20) % 80), significand);
  }
}
int main() {
  static_assert(sizeof(long double) == 16 && __LDBL_MANT_DIG__ == 64);
  std::uint64_t seed = 20260928;
  for (int i = 0; i < 1024; ++i) {
    volatile long double a = sample(seed), b = sample(seed), c = sample(seed), d = sample(seed);
    const std::complex<long double> z(a, b), w(c, d);
    const std::complex<long double> product = z * w, quotient = z / w;
    for (long double value : {(long double)a, (long double)b, (long double)c, (long double)d,
                              std::abs(z), product.real(), product.imag(), quotient.real(),
                              quotient.imag()})
      emit(value);
    std::cout << '\n';
  }
}
