#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <iostream>

static void map3() {
  int values[] = {2, 0, 0, 1, 11, 0, 0, 0, 1};
  const auto& readonly = NCollection_Mat3<int>::Map(static_cast<const int*>(values));
  auto& matrix = NCollection_Mat3<int>::Map(values);
  std::cout << "mat3 " << readonly.Determinant() << ' '
            << (readonly.GetData() == values) << ' ' << (matrix.ChangeData() == values);
  matrix.Transpose();
  matrix.SetDiagonal(NCollection_Vec3<int>(3, 4, 5));
  for (int value : values) std::cout << ' ' << value;
  std::cout << '\n';
}

static void map4() {
  double values[] = {1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1};
  auto& matrix = NCollection_Mat4<double>::Map(values);
  matrix.Translate(NCollection_Vec3<double>(2, 3, 4));
  const auto& readonly = NCollection_Mat4<double>::Map(static_cast<const double*>(values));
  const auto inverse = readonly.Inverted();
  std::cout << "mat4 " << (matrix.ChangeData() == values) << ' '
            << (readonly.GetData() == values) << ' ' << values[12] << ' ' << values[13]
            << ' ' << values[14] << ' ' << readonly.GetMat3().Determinant() << ' '
            << inverse.GetValue(0, 3) << ' ' << inverse.GetValue(1, 3) << ' '
            << inverse.GetValue(2, 3) << ' ' << (matrix * inverse).IsIdentity() << '\n';
}

int main() { map3(); map4(); }
