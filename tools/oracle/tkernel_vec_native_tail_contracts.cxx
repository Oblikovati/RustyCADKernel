#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <iostream>

struct CustomZero {
  int value;
  CustomZero() : value(7) {}
  explicit CustomZero(int input) : value(input) {}
};

template <class T>
static void PrintPair(const char* label, const NCollection_Vec2<T>& pair)
{
  std::cout << label << ' ' << +pair.x() << ' ' << +pair.y() << '\n';
}

static void PrintUnsignedNegation()
{
  PrintPair("neg_u8", -NCollection_Vec2<unsigned char>(0, 1));
  PrintPair("neg_u16", -NCollection_Vec2<unsigned short>(1, 32768));
  PrintPair("neg_u32", -NCollection_Vec2<unsigned int>(1, 0x80000000u));
  PrintPair("neg_u64", -NCollection_Vec2<unsigned long long>(1, 0x8000000000000000ull));
}

static void PrintBooleanOperators()
{
  const NCollection_Vec2<bool> left(true, false), right(true, true);
  PrintPair("bool_add", left + right);
  PrintPair("bool_sub", left - right);
  PrintPair("bool_mul", left * right);
  PrintPair("bool_div", left / true);
  PrintPair("bool_neg", -left);
  PrintPair("bool_abs", left.cwiseAbs());
  std::cout << "bool_dot " << left.Dot(right) << ' ' << left.Modulus() << ' ' << left.SquareModulus() << '\n';
  NCollection_Vec3<bool> spatial(true, false, true);
  spatial.Normalize();
  std::cout << "bool_normalize " << spatial.x() << ' ' << spatial.y() << ' ' << spatial.z() << '\n';
}

static void PrintCustomZero()
{
  const NCollection_Vec2<CustomZero> pair;
  const NCollection_Vec3<CustomZero> triple;
  const NCollection_Vec4<CustomZero> quadruple(NCollection_Vec2<CustomZero>(CustomZero(1), CustomZero(2)));
  std::cout << "custom_zero " << pair.x().value << ' ' << pair.y().value << ' '
            << triple.z().value << ' ' << quadruple.z().value << ' ' << quadruple.w().value << '\n';
}

static void PrintExtraConstructors()
{
  const NCollection_Vec4<CustomZero> fourth;
  const auto axis = NCollection_Vec3<CustomZero>::DZ();
  const NCollection_Vec4<CustomZero> extended(axis);
  std::cout << "extra_zero " << fourth.x().value << ' ' << fourth.w().value << ' ' << axis.x().value << ' ' << axis.z().value << ' ' << extended.w().value << '\n';
}

template <class V>
static V FromMask(unsigned mask)
{
  V vector;
  for (int i = 0; i < V::Length(); ++i) vector[i] = (mask & (1u << i)) != 0;
  return vector;
}

template <class V>
static unsigned Mask(const V& vector)
{
  unsigned result = 0;
  for (int i = 0; i < V::Length(); ++i) if (vector[i]) result |= 1u << i;
  return result;
}

template <class V>
static void PrintBooleanPair(unsigned a, unsigned b)
{
  const V left = FromMask<V>(a), right = FromMask<V>(b), ones(true);
  V add(left), sub(left), mul(left), div(left);
  const bool identity = &(add += right) == &add && &(sub -= right) == &sub && &(mul *= right) == &mul && &(div /= ones) == &div;
  std::cout << "truth" << V::Length() << ' ' << a << ' ' << b << ' ' << Mask(left + right) << ' ' << Mask(left - right) << ' ' << Mask(left * right) << ' ' << left.Dot(right)
            << ' ' << Mask(add) << ' ' << Mask(sub) << ' ' << Mask(mul) << ' ' << Mask(div) << ' ' << identity << ' ' << Mask(-left) << ' ' << Mask(left.cwiseAbs())
            << ' ' << Mask(left / true) << ' ' << Mask(left * false) << ' ' << Mask(left * true) << '\n';
}

template <class V>
static void PrintBooleanTruthTable()
{
  for (unsigned a = 0; a < (1u << V::Length()); ++a)
    for (unsigned b = 0; b < (1u << V::Length()); ++b) PrintBooleanPair<V>(a, b);
}

static void PrintSpatialBooleanTruthTable()
{
  using V = NCollection_Vec3<bool>;
  for (unsigned a = 0; a < 8; ++a) {
    const auto left = FromMask<V>(a);
    for (unsigned b = 0; b < 8; ++b) {
      const auto right = FromMask<V>(b);
      std::cout << "space " << a << ' ' << b << ' ' << Mask(V::Cross(left, right)) << ' ' << Mask(V::GetLERP(left, right, false)) << ' ' << Mask(V::GetLERP(left, right, true))
                << ' ' << Mask(left.Normalized()) << ' ' << left.Modulus() << ' ' << left.SquareModulus() << '\n';
    }
  }
}

int main()
{
  PrintUnsignedNegation();
  PrintBooleanOperators();
  PrintCustomZero();
  PrintExtraConstructors();
  PrintBooleanTruthTable<NCollection_Vec2<bool>>();
  PrintBooleanTruthTable<NCollection_Vec3<bool>>();
  PrintBooleanTruthTable<NCollection_Vec4<bool>>();
  PrintSpatialBooleanTruthTable();
}
