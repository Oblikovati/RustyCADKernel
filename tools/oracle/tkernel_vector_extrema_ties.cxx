#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <cstdint>
#include <cstring>
#include <iostream>

static double payload_nan()
{
  const std::uint64_t bits = UINT64_C(0x7ff8000000001234);
  double value;
  std::memcpy(&value, &bits, sizeof(value));
  return value;
}

template <typename Vector> static void print_bits(const Vector& vector)
{
  for (int index = 0; index < Vector::Length(); ++index)
  {
    const double value = vector[index];
    std::uint64_t bits;
    std::memcpy(&bits, &value, sizeof(bits));
    std::cout << (index == 0 ? "" : " ") << std::hex << bits;
  }
  std::cout << '\n';
}

template <typename Vector> static void compare_ties(const Vector& left, const Vector& right)
{
  print_bits(left.cwiseMin(right));
  print_bits(left.cwiseMax(right));
  print_bits(right.cwiseMin(left));
  print_bits(right.cwiseMax(left));
}

int main()
{
  const double nan = payload_nan();
  compare_ties(NCollection_Vec2<double>(-0.0, 7.0), NCollection_Vec2<double>(0.0, nan));
  compare_ties(NCollection_Vec3<double>(-0.0, 7.0, nan), NCollection_Vec3<double>(0.0, nan, 7.0));
  compare_ties(NCollection_Vec4<double>(-0.0, 7.0, nan, 0.0),
               NCollection_Vec4<double>(0.0, nan, 7.0, -0.0));
}
