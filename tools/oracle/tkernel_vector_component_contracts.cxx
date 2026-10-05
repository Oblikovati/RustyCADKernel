#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <sstream>

template <typename T> static void scalar_contract(const char* label)
{
  const NCollection_Vec2<T> pair(T(3), T(4));
  const NCollection_Vec3<T> triple(T(3), T(4), T(0));
  const auto normalized = triple.Normalized();
  const auto zero = NCollection_Vec3<T>(T(0)).Normalized();
  const auto abs2 = NCollection_Vec2<T>(T(-3), T(4)).cwiseAbs();
  const auto abs3 = NCollection_Vec3<T>(T(-3), T(-4), T(-5)).cwiseAbs();
  const auto abs4 = NCollection_Vec4<T>(T(-3)).cwiseAbs();
  std::cout << label << ' ' << +pair.SquareModulus() << ' ' << +triple.SquareModulus() << ' ' << +pair.Modulus() << ' ' << +triple.Modulus() << ' '
            << +normalized.x() << ' ' << +normalized.y() << ' ' << +normalized.z() << ' '
            << +zero.x() << ' ' << +zero.y() << ' ' << +zero.z() << ' '
            << +abs2.x() << ' ' << +abs2.y() << ' '
            << +abs3.x() << ' ' << +abs3.y() << ' ' << +abs3.z() << ' '
            << +abs4.x() << ' ' << +abs4.y() << ' ' << +abs4.z() << ' ' << +abs4.w() << '\n';
}
template <typename T> static void promoted_contract(const char* label, T x, T y)
{
  const auto normalized = NCollection_Vec3<T>(x, y, T(0)).Normalized();
  std::cout << label << ' ' << +NCollection_Vec2<T>(x, y).SquareModulus() << ' ' << +NCollection_Vec2<T>(x, y).Modulus() << ' '
            << +normalized.x() << ' ' << +normalized.y() << ' ' << +normalized.z() << '\n';
}
template <typename T> static void square_contract(const char* label, T x, T y)
{
  std::cout << label << ' ' << static_cast<unsigned long long>(NCollection_Vec2<T>(x, y).SquareModulus()) << ' '
            << static_cast<unsigned long long>(NCollection_Vec3<T>(x, y, T(0)).SquareModulus()) << '\n';
}
static void remaining_square_contracts()
{
  square_contract<unsigned char>("square-u8", 100, 0);
  square_contract<unsigned short>("square-u16", 200, 200);
  square_contract<unsigned int>("square-u32", 65536, 0);
  square_contract<unsigned long long>("square-u64", 4294967296, 0);
  square_contract<unsigned long>("square-usize", 3, 4);
  square_contract<__int128>("square-i128", 3, 4);
  square_contract<unsigned __int128>("square-u128", 3, 4);
}
template <typename T, typename Bits> static void absolute_bits(const char* label, Bits bits)
{
  T nan;
  std::memcpy(&nan, &bits, sizeof(T));
  const auto vector = NCollection_Vec2<T>(T(-0.0), nan).cwiseAbs();
  Bits zeroBits, nanBits;
  const T zero = vector.x(), absolute = vector.y();
  std::memcpy(&zeroBits, &zero, sizeof(T));
  std::memcpy(&nanBits, &absolute, sizeof(T));
  std::cout << label << ' ' << std::hex << zeroBits << ' ' << nanBits << std::dec << '\n';
}
template <typename Value> static void dump_contract(const char* label, const Value& value)
{
  std::ostringstream output;
  value.DumpJson(output, -1);
  std::cout << label << ' ' << output.str() << '\n';
}
template <typename T> static void floating_dumps(const char* label)
{
  dump_contract(label, NCollection_Vec2<T>(T(0.1), T(-2)));
  dump_contract(label, NCollection_Vec3<T>(T(1), T(0.5), T(-2)));
  dump_contract(label, NCollection_Vec4<T>(T(0.1), T(0), T(0), T(1)));
  dump_contract(label, NCollection_Mat3<T>());
  dump_contract(label, NCollection_Mat4<T>());
}
struct DivisionElement
{
  double value;
  int divisions;
  DivisionElement(double number, int count = 0) : value(number), divisions(count) {}
  operator double() const { return value; }
  DivisionElement& operator/=(const DivisionElement&) = delete;
};
static DivisionElement operator*(DivisionElement left, DivisionElement right)
{
  return DivisionElement(left.value * right.value);
}
static DivisionElement operator+(DivisionElement left, DivisionElement right)
{
  return DivisionElement(left.value + right.value);
}
static DivisionElement operator/(DivisionElement left, DivisionElement right)
{
  return DivisionElement(left.value / right.value, left.divisions + 1);
}
static bool operator!=(DivisionElement left, DivisionElement right)
{
  return left.value != right.value;
}
static void custom_division_contract()
{
  const auto normalized = NCollection_Vec3<DivisionElement>(3, 4, 0).Normalized();
  std::cout << "custom " << normalized.x().value << ' ' << normalized.y().value << ' '
            << normalized.z().value << ' ' << normalized.x().divisions << ' '
            << normalized.y().divisions << ' ' << normalized.z().divisions << '\n';
}
static void primitive_contracts()
{
  scalar_contract<float>("f32");
  scalar_contract<double>("f64");
  scalar_contract<signed char>("i8");
  scalar_contract<short>("i16");
  scalar_contract<int>("i32");
  scalar_contract<long long>("i64");
  scalar_contract<long>("isize");
  promoted_contract<signed char>("promoted-i8", 100, 0);
  promoted_contract<short>("promoted-i16", 200, 200);
}
int main()
{
  std::cout << std::setprecision(17);
  primitive_contracts();
  absolute_bits<float>("bits-f32", uint32_t(0xffc12345));
  absolute_bits<double>("bits-f64", uint64_t(0xfff8000000001234));
  floating_dumps<float>("dump-f32");
  floating_dumps<double>("dump-f64");
  custom_division_contract();
  remaining_square_contracts();
}
