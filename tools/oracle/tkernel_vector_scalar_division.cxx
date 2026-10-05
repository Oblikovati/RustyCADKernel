#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <cstdint>
#include <cstring>
#include <iostream>

struct DivisionTagged
{
  int value;
  constexpr DivisionTagged(int initial = 0) : value(initial) {}
  constexpr DivisionTagged& operator/=(DivisionTagged right)
  { value = 2000 + value * 10 + right.value; return *this; }
};

template<class Vector, size_t Dimension>
void print_values(const Vector& vector)
{
  for (size_t index = 0; index < Dimension; ++index)
    std::cout << (index == 0 ? "" : " ") << vector[index].value;
  std::cout << '\n';
}

void tagged_paths()
{
  NCollection_Vec3<DivisionTagged> first(DivisionTagged(2), DivisionTagged(4), DivisionTagged(6));
  print_values<decltype(first), 3>(first / DivisionTagged(3));
  first /= DivisionTagged(3);
  print_values<decltype(first), 3>(first);
  NCollection_Vec4<DivisionTagged> second(DivisionTagged(2));
  print_values<decltype(second), 4>(second / DivisionTagged(3));
  second /= DivisionTagged(3);
  print_values<decltype(second), 4>(second);
}

template<class Float, class Bits>
Bits float_bits(Float value)
{
  Bits bits;
  std::memcpy(&bits, &value, sizeof(value));
  return bits;
}

int main()
{
  tagged_paths();
  const auto first = NCollection_Vec3<float>(5.0f, -5.0f, 0.0f) / 3.0f;
  const auto second = NCollection_Vec4<double>(5.0, -5.0, 0.0, 1.0) / 3.0;
  std::cout << std::hex << float_bits<float, uint32_t>(first.x()) << '\n';
  std::cout << std::hex << float_bits<double, uint64_t>(second.x()) << '\n';
}
