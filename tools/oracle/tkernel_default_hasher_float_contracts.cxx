#include <NCollection_DataMap.hxx>
#include <cstdint>
#include <cstring>
#include <iostream>

template <class Float, class Bits>
static void PrintFloat(const char* label, Bits bits)
{
  Float value;
  std::memcpy(&value, &bits, sizeof(value));
  NCollection_DefaultHasher<Float> policy;
  std::cout << label << ' ' << bits << ' ' << policy(value) << ' ' << policy(value, value) << '\n';
}

template <class Integer>
static void PrintInteger(const char* label, Integer value)
{
  NCollection_DefaultHasher<Integer> policy;
  std::cout << label << ' ' << sizeof(value) << ' ' << policy(value) << ' '
            << policy(value, value) << '\n';
}

static void PrintFloats()
{
  const uint32_t bits32[] = {0u, 0x80000000u, 0x3f800000u, 0xbf800000u,
    0x40490fdbu, 1u, 0x80000001u, 0x007fffffu, 0x00800000u, 0x7f7fffffu,
    0xff7fffffu, 0x7f800000u, 0xff800000u, 0x7fc00000u, 0x7fc00001u,
    0x7fa00001u, 0xffc00001u, 0xffa00001u};
  for (uint32_t bits : bits32) PrintFloat<float>("f32", bits);
  const uint64_t bits64[] = {0ull, 0x8000000000000000ull, 0x3ff0000000000000ull,
    1ull, 0x7ff0000000000000ull, 0xfff0000000000000ull, 0x7ff8000000000001ull,
    0x7ff0000000000001ull, 0xfff8000000000001ull};
  for (uint64_t bits : bits64) PrintFloat<double>("f64", bits);
}

static void PrintMapTransitions()
{
  NCollection_DataMap<float, int> map;
  std::cout << "map_zero " << map.Bind(0.0f, 1) << ' ' << map.Bind(-0.0f, 2)
            << ' ' << map.Size() << ' ' << map.Find(0.0f) << '\n';
  std::cout << "map_finite " << map.Bind(1.25f, 3) << ' ' << map.Find(1.25f) << '\n';
  float nan; const uint32_t nanBits = 0x7fc00001u;
  std::memcpy(&nan, &nanBits, sizeof(nan));
  std::cout << "map_nan " << map.Bind(nan, 4) << ' ' << map.Bind(nan, 5)
            << ' ' << map.Size() << ' ' << map.IsBound(nan) << '\n';
}

static void PrintIntegralSpecializations()
{
  PrintInteger("bool", true);
  PrintInteger("char", static_cast<char>(-1));
  PrintInteger("signed_char", static_cast<signed char>(-1));
  PrintInteger("unsigned_char", static_cast<unsigned char>(255));
  PrintInteger("wchar_t", static_cast<wchar_t>(-1));
  PrintInteger("char16_t", static_cast<char16_t>(0xffff));
  PrintInteger("char32_t", static_cast<char32_t>(0xffffffffu));
  PrintInteger("short", static_cast<short>(-1));
  PrintInteger("int", -1);
  PrintInteger("long", -1L);
  PrintInteger("long_long", -1LL);
  PrintInteger("unsigned_short", static_cast<unsigned short>(0xffff));
  PrintInteger("unsigned_int", 0xffffffffu);
  PrintInteger("unsigned_long", ~0ul);
  PrintInteger("unsigned_long_long", ~0ull);
}

static void PrintPointerSpecialization()
{
  int pointed = 1;
  NCollection_DefaultHasher<int*> pointerPolicy;
  std::cout << "pointer " << (pointerPolicy(&pointed) == reinterpret_cast<size_t>(&pointed))
            << ' ' << pointerPolicy(nullptr) << ' ' << pointerPolicy(&pointed, &pointed) << '\n';
}

int main()
{
  PrintFloats();
  PrintMapTransitions();
  PrintIntegralSpecializations();
  PrintPointerSpecialization();
}
