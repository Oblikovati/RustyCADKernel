#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>

#include <iostream>

struct AssignmentOnly
{
  int value = 0;
  constexpr AssignmentOnly() = default;
  constexpr AssignmentOnly(int theValue) : value(theValue) {}
  constexpr AssignmentOnly& operator+=(const AssignmentOnly& theOther)
  {
    value += theOther.value;
    return *this;
  }
  constexpr AssignmentOnly& operator-=(const AssignmentOnly& theOther)
  {
    value -= theOther.value;
    return *this;
  }
};

template <typename Matrix>
static void dump(const char* theName, const Matrix& theMatrix, int theSize)
{
  std::cout << theName;
  for (int column = 0; column < theSize; ++column)
  {
    for (int row = 0; row < theSize; ++row)
      std::cout << ' ' << theMatrix.GetValue(row, column).value;
  }
  std::cout << '\n';
}

static void run_mat3()
{
  NCollection_Mat3<AssignmentOnly> left, right;
  for (int column = 0; column < 3; ++column)
    for (int row = 0; row < 3; ++row)
    {
      left.SetValue(row, column, row + 3 * column);
      right.SetValue(row, column, 10 + row + 3 * column);
    }
  auto sum = left.Added(right), difference = left.Subtracted(right);
  left += right;
  dump("mat3_add", left, 3);
  left -= right;
  dump("mat3_subtract", left, 3);
  dump("mat3_added", sum, 3);
  dump("mat3_subtracted", difference, 3);
}

static void run_mat4()
{
  NCollection_Mat4<AssignmentOnly> left, right;
  for (int column = 0; column < 4; ++column)
    for (int row = 0; row < 4; ++row)
    {
      left.SetValue(row, column, row + 4 * column);
      right.SetValue(row, column, 10 + row + 4 * column);
    }
  auto sum = left.Added(right), difference = left.Subtracted(right);
  left += right;
  dump("mat4_add", left, 4);
  left -= right;
  dump("mat4_subtract", left, 4);
  dump("mat4_added", sum, 4);
  dump("mat4_subtracted", difference, 4);
}

int main()
{
  run_mat3();
  run_mat4();
}
