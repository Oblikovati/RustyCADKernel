#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <Standard_Failure.hxx>

#include <iostream>

template <typename T>
NCollection_Mat3<T> diagonal3(T theX, T theY, T theZ)
{
  NCollection_Mat3<T> aMatrix;
  aMatrix.SetValue(0, 0, theX);
  aMatrix.SetValue(1, 1, theY);
  aMatrix.SetValue(2, 2, theZ);
  return aMatrix;
}

template <typename T>
NCollection_Mat4<T> diagonal4(T theX, T theY, T theZ, T theW)
{
  NCollection_Mat4<T> aMatrix;
  aMatrix.SetValue(0, 0, theX);
  aMatrix.SetValue(1, 1, theY);
  aMatrix.SetValue(2, 2, theZ);
  aMatrix.SetValue(3, 3, theW);
  return aMatrix;
}

template <typename Matrix, int N>
void print_data(const Matrix& theMatrix)
{
  const auto* aData = theMatrix.GetData();
  for (int anIndex = 0; anIndex < N * N; ++anIndex)
  {
    std::cout << (anIndex == 0 ? "" : " ") << aData[anIndex];
  }
}

template <typename T>
void probe_mat3_regular()
{
  auto aUnit = diagonal3<T>(T(1), T(1), T(1));
  auto aNegative = diagonal3<T>(T(-1), T(1), T(1));
  auto aTwo = diagonal3<T>(T(2), T(1), T(1));
  NCollection_Mat3<T> anOutput;
  T aDeterminant = T(0);
  std::cout << "mat3 unit " << aUnit.Determinant() << ' ';
  print_data<NCollection_Mat3<T>, 3>(aUnit.Adjoint());
  std::cout << ' ' << aUnit.Inverted(anOutput, aDeterminant) << ' ' << aDeterminant << ' ';
  print_data<NCollection_Mat3<T>, 3>(anOutput);
  std::cout << '\n' << "mat3 negative " << aNegative.Determinant() << " adj ";
  print_data<NCollection_Mat3<T>, 3>(aNegative.Adjoint());
  std::cout << ' ' << aNegative.Inverted(anOutput, aDeterminant) << ' ' << aDeterminant << ' ';
  print_data<NCollection_Mat3<T>, 3>(anOutput);
  std::cout << '\n' << "mat3 two " << aTwo.Inverted(anOutput, aDeterminant) << ' ' << aDeterminant << ' ';
  print_data<NCollection_Mat3<T>, 3>(anOutput);
  std::cout << '\n';
}

template <typename T>
void probe_mat3_singular()
{
  auto aSingular = diagonal3<T>(T(0), T(1), T(1));
  NCollection_Mat3<T> anOutput;
  for (int anIndex = 0; anIndex < 9; ++anIndex)
  {
    anOutput.SetValue(anIndex % 3, anIndex / 3, T(7));
  }
  T aDeterminant = T(7);
  std::cout << "mat3 zero " << aSingular.Inverted(anOutput, aDeterminant) << ' ' << aDeterminant << ' ';
  print_data<NCollection_Mat3<T>, 3>(anOutput);
  std::cout << '\n' << "mat3 wrappers " << diagonal3<T>(T(1), T(1), T(1)).Inverted().GetValue(0, 0) << ' ';
  try { aSingular.Inverted(); std::cout << "no-error\n"; }
  catch (const Standard_Failure&) { std::cout << "throws\n"; }
}

template <typename T>
void probe_mat3()
{
  probe_mat3_regular<T>();
  probe_mat3_singular<T>();
}

template <typename T>
void probe_mat4_regular()
{
  auto aUnit = diagonal4<T>(T(1), T(1), T(1), T(1));
  auto aNegative = diagonal4<T>(T(-1), T(1), T(1), T(1));
  auto aTwo = diagonal4<T>(T(2), T(1), T(1), T(1));
  NCollection_Mat4<T> anOutput;
  T aDeterminant = T(0);
  std::cout << "mat4 unit " << aUnit.Inverted(anOutput, aDeterminant) << ' ' << aDeterminant << ' ';
  print_data<NCollection_Mat4<T>, 4>(aUnit.Adjoint());
  std::cout << ' ';
  print_data<NCollection_Mat4<T>, 4>(anOutput);
  std::cout << '\n' << "mat4 negative ";
  print_data<NCollection_Mat4<T>, 4>(aNegative.Adjoint());
  std::cout << ' ' << aNegative.Inverted(anOutput, aDeterminant) << ' ' << aDeterminant << ' ';
  print_data<NCollection_Mat4<T>, 4>(anOutput);
  std::cout << '\n' << "mat4 two " << aTwo.Inverted(anOutput, aDeterminant) << ' ' << aDeterminant << ' ';
  print_data<NCollection_Mat4<T>, 4>(anOutput);
  std::cout << '\n';
}

template <typename T>
void probe_mat4_singular()
{
  auto aSingular = diagonal4<T>(T(0), T(1), T(1), T(1));
  NCollection_Mat4<T> anOutput;
  T aDeterminant = T(7);
  std::cout << "mat4 zero " << aSingular.Inverted(anOutput, aDeterminant) << ' ' << aDeterminant << ' ';
  print_data<NCollection_Mat4<T>, 4>(anOutput);
  std::cout << '\n' << "mat4 wrappers " << diagonal4<T>(T(1), T(1), T(1), T(1)).Inverted().GetValue(0, 0) << ' ';
  try { aSingular.Inverted(); std::cout << "no-error\n"; }
  catch (const Standard_Failure&) { std::cout << "throws\n"; }
}

template <typename T>
void probe_mat4_translation()
{
  auto aTranslated = diagonal4<T>(T(2), T(3), T(4), T(1));
  aTranslated.Translate(NCollection_Vec3<T>(T(1), T(-2), T(3)));
  std::cout << "mat4 translate ";
  print_data<NCollection_Mat4<T>, 4>(aTranslated);
  std::cout << '\n';
}

template <typename T>
void probe_mat4()
{
  probe_mat4_regular<T>();
  probe_mat4_singular<T>();
  probe_mat4_translation<T>();
}

int main()
{
  std::cout << "i32\n";
  probe_mat3<int>();
  probe_mat4<int>();
  std::cout << "i64\n";
  probe_mat3<long long>();
  probe_mat4<long long>();
}
