#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <iostream>

struct ArithmeticOnly
{
  int value;

  constexpr ArithmeticOnly(int theValue = 0) : value(theValue) {}
  friend constexpr ArithmeticOnly operator*(ArithmeticOnly theLeft, ArithmeticOnly theRight)
  {
    return theLeft.value * theRight.value;
  }
  friend constexpr ArithmeticOnly operator+(ArithmeticOnly theLeft, ArithmeticOnly theRight)
  {
    return theLeft.value + theRight.value;
  }
  friend constexpr ArithmeticOnly operator-(ArithmeticOnly theLeft, ArithmeticOnly theRight)
  {
    return theLeft.value - theRight.value;
  }
};

template <class Element>
void Fill3(NCollection_Mat3<Element>& theMatrix)
{
  const int aValues[3][3] = {{2, 1, 0}, {0, 11, 0}, {0, 0, 1}};
  for (int aRow = 0; aRow < 3; ++aRow)
  {
    for (int aCol = 0; aCol < 3; ++aCol)
    {
      theMatrix.SetValue(aRow, aCol, Element(aValues[aRow][aCol]));
    }
  }
}

template <class Element>
void Fill4(NCollection_Mat4<Element>& theMatrix)
{
  NCollection_Mat3<Element> aBlock;
  Fill3(aBlock);
  theMatrix.SetValue(3, 3, Element(1));
  for (int aRow = 0; aRow < 3; ++aRow)
  {
    for (int aCol = 0; aCol < 3; ++aCol)
    {
      theMatrix.SetValue(aRow, aCol, aBlock.GetValue(aRow, aCol));
    }
  }
}

int main()
{
  NCollection_Mat3<int> aMat3Int;
  NCollection_Mat3<ArithmeticOnly> aMat3Only;
  NCollection_Mat4<int> aMat4Int;
  NCollection_Mat4<ArithmeticOnly> aMat4Only;
  Fill3(aMat3Int);
  Fill3(aMat3Only);
  Fill4(aMat4Int);
  Fill4(aMat4Only);
  std::cout << "Mat3-int " << aMat3Int.Determinant() << '\n';
  std::cout << "Mat3-arithmetic-only " << aMat3Only.Determinant().value << '\n';
  std::cout << "Mat4-int " << aMat4Int.DeterminantMat3() << '\n';
  std::cout << "Mat4-arithmetic-only " << aMat4Only.DeterminantMat3().value << '\n';
}
