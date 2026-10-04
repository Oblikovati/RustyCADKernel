// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871, Standard_CString C-locale forwarding.
#include <Standard_CString.hxx>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>

static void printCase(std::uint64_t bits, const char* format)
{
  double value;
  std::memcpy(&value, &bits, sizeof(value));
  char output[512];
  const int count = Sprintf(output, format, value);
  std::cout << std::hex << std::setfill('0') << std::setw(16) << bits
            << '|' << format << '|' << std::dec << count << '|';
  for (int i = 0; i < count; ++i)
    std::cout << std::hex << std::setw(2) << static_cast<unsigned>(static_cast<unsigned char>(output[i]));
  std::cout << '\n';
}

static void printValue(std::uint64_t bits)
{
  const char* formats[] = {"%a", "%A", "%#a", "%.0a", "%#.0A", "%.1a", "%.2a",
    "%.6a", "%.12a", "%.13a", "%.20a", "%+028.3a", "% 028.3A", "%-28.4a", "%28a", "%la"};
  for (const char* format : formats)
    printCase(bits, format);
}

int main()
{
  const std::uint64_t edges[] = {0, 0x8000000000000000ULL, 1, 0x8000000000000001ULL,
    0x000fffffffffffffULL, 0x0010000000000000ULL, 0x3ff0000000000000ULL,
    0x3ff8000000000000ULL, 0x3ff0800000000000ULL, 0x3ff1800000000000ULL,
    0x3fffffffffffffffULL, 0x7fefffffffffffffULL, 0xffefffffffffffffULL,
    0x7ff0000000000000ULL, 0xfff0000000000000ULL, 0x7ff8000000000000ULL,
    0xfff8000000000000ULL, 0x7ff0000000000001ULL};
  for (std::uint64_t bits : edges)
    printValue(bits);
  std::uint64_t seed = 0x719034da8761bcf2ULL;
  for (int i = 0; i < 64; ++i) {
    seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
    printValue(seed);
  }
}
