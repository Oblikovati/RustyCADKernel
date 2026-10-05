#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <cstdint>
#include <cstring>
#include <iostream>

template <typename Vector> static void scalar_integer_returns()
{
  Vector destination(12);
  bool same = &(destination *= 3) == &destination;
  std::cout << Vector::Length() << " mul " << same << ' ' << destination[0] << '\n';
  same = &(destination /= 3) == &destination;
  std::cout << Vector::Length() << " div " << same << ' ' << destination[0] << '\n';
}

template <typename Vector, typename Element, typename Bits> static void scalar_float_return()
{
  Vector destination(Element(5));
  const bool same = &(destination /= Element(3)) == &destination;
  const Element component = destination[0];
  Bits bits;
  std::memcpy(&bits, &component, sizeof(bits));
  std::cout << Vector::Length() << " f" << sizeof(Element) * 8 << ' ' << same << ' '
            << std::hex << bits << std::dec << '\n';
}

int main()
{
  scalar_integer_returns<NCollection_Vec2<int>>();
  scalar_integer_returns<NCollection_Vec3<int>>();
  scalar_integer_returns<NCollection_Vec4<int>>();
  scalar_float_return<NCollection_Vec3<float>, float, std::uint32_t>();
  scalar_float_return<NCollection_Vec3<double>, double, std::uint64_t>();
  scalar_float_return<NCollection_Vec4<float>, float, std::uint32_t>();
  scalar_float_return<NCollection_Vec4<double>, double, std::uint64_t>();
}
