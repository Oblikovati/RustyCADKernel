// Different value/assignment operators expose the native dimension-specific path.
#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <cstdint>
#include <iostream>

struct IntegerElement {
  std::int64_t value;
  constexpr IntegerElement(std::int64_t input = 0) : value(input) {}
  constexpr IntegerElement operator*(IntegerElement right) const {
    return IntegerElement(value * right.value);
  }
  constexpr IntegerElement& operator*=(IntegerElement right) {
    value = value * right.value + 1;
    return *this;
  }
};
std::ostream& operator<<(std::ostream& stream, IntegerElement element) {
  return stream << element.value;
}
template<class Vector> void print(const Vector& vector) {
  for (int component = 0; component < Vector::Length(); ++component) {
    if (component != 0) std::cout << ' ';
    std::cout << vector[component];
  }
  std::cout << '\n';
}
template<class Element> void checkScalarOperators() {
  auto planar = NCollection_Vec2<Element>(Element(-3), Element(7)) * Element(-2);
  print(planar); planar *= Element(3); print(planar);
  auto spatial = NCollection_Vec3<Element>(Element(1), Element(-2), Element(5)) * Element(4);
  print(spatial); spatial *= Element(-3); print(spatial);
  auto homogeneous = NCollection_Vec4<Element>(Element(-2), Element(3), Element(0), Element(9)) * Element(-2);
  print(homogeneous); homogeneous *= Element(2); print(homogeneous);
}
int main() {
  checkScalarOperators<std::int64_t>();
  checkScalarOperators<IntegerElement>();
}
