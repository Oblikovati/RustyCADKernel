#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>

#include <iostream>

struct BinaryOnly
{
  int value;
  constexpr BinaryOnly(int theValue = 0) : value(theValue) {}
};

constexpr BinaryOnly operator+(BinaryOnly theLeft, BinaryOnly theRight)
{
  return BinaryOnly(theLeft.value + theRight.value);
}

constexpr BinaryOnly operator*(BinaryOnly theLeft, BinaryOnly theRight)
{
  return BinaryOnly(theLeft.value * theRight.value);
}

void printBinaryOnlyProducts()
{
  NCollection_Mat3<BinaryOnly> aMat3;
  NCollection_Mat4<BinaryOnly> aMat4;
  for (int anIndex = 0; anIndex < 9; ++anIndex)
  {
    aMat3.SetValue(anIndex % 3, anIndex / 3, BinaryOnly(anIndex + 1));
  }
  for (int anIndex = 0; anIndex < 16; ++anIndex)
  {
    aMat4.SetValue(anIndex % 4, anIndex / 4, BinaryOnly(anIndex + 1));
  }
  const auto aProduct3 = aMat3 * NCollection_Vec3<BinaryOnly>(BinaryOnly(2), BinaryOnly(3), BinaryOnly(4));
  const auto aProduct4 = aMat4 * NCollection_Vec4<BinaryOnly>(BinaryOnly(1), BinaryOnly(2), BinaryOnly(3), BinaryOnly(4));
  std::cout << aProduct3.x().value << ' ' << aProduct3.y().value << ' ' << aProduct3.z().value << '\n';
  std::cout << aProduct4.x().value << ' ' << aProduct4.y().value << ' ' << aProduct4.z().value << ' '
            << aProduct4.w().value << '\n';
}

void printSignedProducts()
{
  NCollection_Mat3<int> aMat3;
  NCollection_Mat4<int> aMat4;
  for (int anIndex = 0; anIndex < 9; ++anIndex)
  {
    aMat3.SetValue(anIndex % 3, anIndex / 3, anIndex + 1);
  }
  for (int anIndex = 0; anIndex < 16; ++anIndex)
  {
    aMat4.SetValue(anIndex % 4, anIndex / 4, anIndex + 1);
  }
  const auto aProduct3 = aMat3 * NCollection_Vec3<int>(2, 3, 4);
  const auto aProduct4 = aMat4 * NCollection_Vec4<int>(1, 2, 3, 4);
  std::cout << aProduct3.x() << ' ' << aProduct3.y() << ' ' << aProduct3.z() << '\n';
  std::cout << aProduct4.x() << ' ' << aProduct4.y() << ' ' << aProduct4.z() << ' ' << aProduct4.w() << '\n';
}

int main()
{
  printBinaryOnlyProducts();
  printSignedProducts();
}
