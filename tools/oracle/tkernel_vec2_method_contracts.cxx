#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <cstdint>
#include <iostream>
#include <limits>

template <class T>
static void Print(const char* label, const NCollection_Vec2<T>& vector)
{
  std::cout << label << ' ' << +vector.x() << ' ' << +vector.y() << '\n';
}

static void PrintConversions()
{
  const NCollection_Vec2<int64_t> integral(-257, 65537);
  Print("i8", NCollection_Vec2<int8_t>(integral));
  Print("u8", NCollection_Vec2<uint8_t>(integral));
  Print("i16", NCollection_Vec2<int16_t>(integral));
  Print("u16", NCollection_Vec2<uint16_t>(integral));
  Print("float_i8", NCollection_Vec2<int8_t>(NCollection_Vec2<double>(-12.75, 127.75)));
  Print("float_u8", NCollection_Vec2<uint8_t>(NCollection_Vec2<float>(0.75f, 254.75f)));
  Print("float_i64", NCollection_Vec2<int64_t>(NCollection_Vec2<double>(-123.5, 456.75)));
  Print("bool", NCollection_Vec2<bool>(NCollection_Vec2<double>(std::numeric_limits<double>::quiet_NaN(), -0.0)));
  Print("bool_float", NCollection_Vec2<float>(NCollection_Vec2<bool>(true, false)));
}

static void PrintPointersAndReturns()
{
  NCollection_Vec2<int> vector(2, 4);
  const NCollection_Vec2<int>& borrowed = vector;
  std::cout << "pointer " << (borrowed.GetData() == &vector.x()) << ' '
            << (vector.ChangeData() == &vector.x()) << ' '
            << (static_cast<const int*>(borrowed) == vector.ChangeData()) << '\n';
  vector.ChangeData()[1] = 9;
  std::cout << "mutable " << vector.y() << ' ' << (&vector.y() == vector.ChangeData() + 1) << '\n';
  std::cout << "returns " << (&(vector += vector) == &vector) << ' '
            << (&(vector -= vector) == &vector) << ' ' << vector.x() << ' ' << vector.y() << '\n';
}

template <class T>
static void PrintUnsignedMagnitude(const char* label)
{
  NCollection_Vec2<T> planar(T(3), T(4));
  NCollection_Vec3<T> spatial(T(3), T(4), T(0));
  const T length = spatial.Modulus();
  spatial.Normalize();
  NCollection_Vec3<T> zero;
  zero.Normalize();
  std::cout << label << ' ' << +planar.Modulus() << ' ' << +length << ' '
            << +spatial.x() << ' ' << +spatial.y() << ' ' << +zero.x() << '\n';
}

static void PrintDefaultZ()
{
  const NCollection_Vec3<unsigned short> spatial(NCollection_Vec2<unsigned short>(1, 2));
  std::cout << "default_z " << spatial.x() << ' ' << spatial.y() << ' ' << spatial.z() << '\n';
}

static void PrintNarrowArithmetic()
{
  NCollection_Vec2<signed char> signedPair(100, 2), signedOther(100, 3);
  NCollection_Vec2<unsigned char> unsignedPair(200, 2), unsignedOther(2, 3);
  Print("signed_product", signedPair * signedOther);
  Print("unsigned_product", unsignedPair * unsignedOther);
  std::cout << "narrow_dot " << +signedPair.Dot(signedOther) << ' ' << +unsignedPair.Dot(unsignedOther) << '\n';
  Print("signed_sum", signedPair + signedOther);
  Print("signed_negation", -NCollection_Vec2<signed char>(-128, 9));
  Print("signed_division", NCollection_Vec2<signed char>(-128, 9) / static_cast<signed char>(-1));
  signedPair *= signedOther;
  Print("signed_assignment", signedPair);
  const auto triple = NCollection_Vec3<signed char>(-128, 9, 0) / static_cast<signed char>(-1);
  std::cout << "vec3_division " << +triple.x() << ' ' << +triple.y() << ' ' << +triple.z() << '\n';
  std::cout << "unsigned_wrap_modulus " << NCollection_Vec3<unsigned int>(65536, 1, 0).Modulus() << '\n';
}

int main()
{
  PrintConversions();
  PrintPointersAndReturns();
  PrintUnsignedMagnitude<unsigned char>("modulus_u8");
  PrintUnsignedMagnitude<unsigned short>("modulus_u16");
  PrintUnsignedMagnitude<unsigned int>("modulus_u32");
  PrintUnsignedMagnitude<unsigned long long>("modulus_u64");
  PrintUnsignedMagnitude<size_t>("modulus_usize");
  PrintDefaultZ();
  PrintNarrowArithmetic();
}
