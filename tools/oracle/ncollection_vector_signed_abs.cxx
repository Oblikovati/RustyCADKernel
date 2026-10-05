#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>

#include <cstdint>
#include <iostream>
#include <limits>

template <class Vector>
void emit(const char* name, const Vector& vector, int count)
{
  std::cout << name << '=';
  for (int index = 0; index < count; ++index)
  {
    if (index != 0)
    {
      std::cout << ',';
    }
    std::cout << static_cast<long long>(vector.GetData()[index]);
  }
  std::cout << '\n';
}

int main()
{
  emit("vec2_i8", NCollection_Vec2<std::int8_t>(-7, 9).cwiseAbs(), 2);
  emit("vec3_i8", NCollection_Vec3<std::int8_t>(-1, 2, -3).cwiseAbs(), 3);
  emit("vec4_i8", NCollection_Vec4<std::int8_t>(-4, 5, -6, 7).cwiseAbs(), 4);
  emit("vec2_i16", NCollection_Vec2<std::int16_t>(-123, 456).cwiseAbs(), 2);
  emit("vec3_i16", NCollection_Vec3<std::int16_t>(-123, 456, -789).cwiseAbs(), 3);
  emit("vec4_i16", NCollection_Vec4<std::int16_t>(-111, 222, -333, 444).cwiseAbs(), 4);
  emit("vec2_i64", NCollection_Vec2<std::int64_t>(-1234567890123LL, 4).cwiseAbs(), 2);
  emit("vec3_i64", NCollection_Vec3<std::int64_t>(-5, 6, -7).cwiseAbs(), 3);
  emit("vec4_i64", NCollection_Vec4<std::int64_t>(-8, 9, -10, 11).cwiseAbs(), 4);
  emit("vec2_long", NCollection_Vec2<long>(-12, 13).cwiseAbs(), 2);
  emit("vec3_long", NCollection_Vec3<long>(-14, 15, -16).cwiseAbs(), 3);
  emit("vec4_long", NCollection_Vec4<long>(-17, 18, -19, 20).cwiseAbs(), 4);
  emit("vec2_i8_min", NCollection_Vec2<std::int8_t>(std::numeric_limits<std::int8_t>::min(), 1).cwiseAbs(), 2);
  emit("vec2_i16_min", NCollection_Vec2<std::int16_t>(std::numeric_limits<std::int16_t>::min(), 1).cwiseAbs(), 2);
}
