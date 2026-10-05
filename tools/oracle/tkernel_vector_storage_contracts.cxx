// GetData, ChangeData and pointer conversions share each vector's native array.
#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <iostream>

template <typename Vector, typename Component, int Count>
void storage_case(const char* name, Vector vector)
{
  const Vector& constant = vector;
  const Component* converted = constant;
  Component* writable = vector;
  const bool shared = constant.GetData() == converted;
  const bool exclusive = vector.ChangeData() == writable;
  const bool field = constant.GetData() == &vector.x();
  const auto stride = reinterpret_cast<const char*>(constant.GetData() + 1)
                      - reinterpret_cast<const char*>(constant.GetData());
  writable[Count - 1] = Component(7);
  std::cout << name << ":" << sizeof(Vector) << ":" << alignof(Vector) << ":"
            << stride << ":" << shared << ":" << exclusive << ":" << field
            << ":" << static_cast<int>(constant.GetData()[Count - 1]) << "\n";
}

struct alignas(32) AlignedComponent
{
  int value;
  explicit AlignedComponent(int input) : value(input) {}
  explicit operator int() const { return value; }
};

constexpr NCollection_Vec3<int> CONSTANT_VECTOR(1, 2, 3);
static_assert(CONSTANT_VECTOR.GetData()[1] == 2);

int main()
{
  storage_case<NCollection_Vec2<int>, int, 2>("vec2", NCollection_Vec2<int>(1, 2));
  storage_case<NCollection_Vec3<int>, int, 3>("vec3", NCollection_Vec3<int>(1, 2, 3));
  storage_case<NCollection_Vec4<int>, int, 4>("vec4", NCollection_Vec4<int>(1, 2, 3, 4));
  storage_case<NCollection_Vec3<AlignedComponent>, AlignedComponent, 3>(
    "aligned", NCollection_Vec3<AlignedComponent>(AlignedComponent(1), AlignedComponent(2), AlignedComponent(3)));
}
