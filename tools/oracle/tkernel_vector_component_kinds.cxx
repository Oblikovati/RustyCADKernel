// Const getters return values; mutable overloads expose the stored component.
#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <iostream>
#include <type_traits>
#include <utility>

#define CHECK_COMPONENT(Vector, component) \
  static_assert(std::is_same_v<decltype(std::declval<const Vector&>().component()), int>); \
  static_assert(std::is_same_v<decltype(std::declval<Vector&>().component()), int&>)
using Vec2 = NCollection_Vec2<int>;
using Vec3 = NCollection_Vec3<int>;
using Vec4 = NCollection_Vec4<int>;
CHECK_COMPONENT(Vec2, x); CHECK_COMPONENT(Vec2, y);
CHECK_COMPONENT(Vec3, x); CHECK_COMPONENT(Vec3, y); CHECK_COMPONENT(Vec3, z);
CHECK_COMPONENT(Vec4, x); CHECK_COMPONENT(Vec4, y); CHECK_COMPONENT(Vec4, z); CHECK_COMPONENT(Vec4, w);

int main()
{
  Vec2 planar(1, 2); Vec3 spatial(3, 4, 5); Vec4 homogeneous(6, 7, 8, 9);
  planar.x() = 11; planar.y() = 12;
  spatial.x() = 13; spatial.y() = 14; spatial.z() = 15;
  homogeneous.x() = 16; homogeneous.y() = 17; homogeneous.z() = 18; homogeneous.w() = 19;
  const Vec2& p = planar; const Vec3& s = spatial; const Vec4& h = homogeneous;
  std::cout << p.x() << ' ' << p.y() << '\n';
  std::cout << s.x() << ' ' << s.y() << ' ' << s.z() << '\n';
  std::cout << h.x() << ' ' << h.y() << ' ' << h.z() << ' ' << h.w() << '\n';
}
