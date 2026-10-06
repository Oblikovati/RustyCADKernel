#include <NCollection_Array2.hxx>
#include <iostream>
#include <string>
#include <type_traits>
struct UninitCell { int value; };
static_assert(std::is_trivially_default_constructible_v<UninitCell>);
static_assert(std::is_trivially_copyable_v<UninitCell>);
template<class Array>
static void fillDefined(Array& array) {
  for (size_t index = 0; index < array.Size(); ++index) {
    array.ChangeAt(index).value = int(index) * 7 + 11;
  }
}
template<class Array>
static std::string filledValues(const Array& array) {
  std::string result;
  for (size_t index = 0; index < array.Size(); ++index) {
    result += (index == 0 ? "" : ",") + std::to_string(array.At(index).value);
  }
  return result;
}
static void linearSize(size_t size) {
  NCollection_Array1<UninitCell> array(size);
  const auto* pointer = array.Data();
  fillDefined(array);
  std::cout << "linear_size_" << size << "=" << array.Lower() << ":" << array.Upper()
      << ";" << array.IsDeletable() << ";" << (pointer == array.Data()) << ";" << filledValues(array) << "\n";
}
static void linearBounds(int lower, int upper) {
  NCollection_Array1<UninitCell> array(lower, upper);
  const auto* pointer = array.Data();
  fillDefined(array);
  std::cout << "linear_bounds_" << lower << "_" << upper << "=" << array.Lower() << ":" << array.Upper()
      << ";" << array.IsDeletable() << ";" << (pointer == array.Data()) << ";" << filledValues(array) << "\n";
}
static std::string matrixBounds(const NCollection_Array2<UninitCell>& array) {
  return std::to_string(array.LowerRow()) + ":" + std::to_string(array.UpperRow()) + ":"
      + std::to_string(array.LowerCol()) + ":" + std::to_string(array.UpperCol()) + ":"
      + std::to_string(array.Lower()) + ":" + std::to_string(array.Upper());
}
static void matrixSize(size_t rows, size_t cols) {
  NCollection_Array2<UninitCell> array(rows, cols);
  const auto* pointer = array.Data();
  fillDefined(array);
  std::cout << "matrix_size_" << rows << "_" << cols << "=" << matrixBounds(array)
      << ";" << array.IsDeletable() << ";" << (pointer == array.Data()) << ";" << filledValues(array) << "\n";
}
static void matrixBoundsCase(int rl, int ru, int cl, int cu) {
  NCollection_Array2<UninitCell> array(rl, ru, cl, cu);
  const auto* pointer = array.Data();
  fillDefined(array);
  std::cout << "matrix_bounds_" << rl << "_" << ru << "_" << cl << "_" << cu << "=" << matrixBounds(array)
      << ";" << array.IsDeletable() << ";" << (pointer == array.Data()) << ";" << filledValues(array) << "\n";
}
static void borrowedLinear() {
  UninitCell buffer[5];
  NCollection_Array1<UninitCell> array(buffer[0], -2, 1);
  fillDefined(array);
  std::cout << "borrowed_linear=" << array.Lower() << ":" << array.Upper() << ";"
      << array.IsDeletable() << ";" << (array.Data() == buffer) << ";" << filledValues(array) << "\n";
}
static void borrowedMatrix() {
  UninitCell buffer[5];
  NCollection_Array2<UninitCell> array(buffer[0], -2, -1, 3, 4);
  fillDefined(array);
  std::cout << "borrowed_matrix=" << matrixBounds(array) << ";"
      << array.IsDeletable() << ";" << (array.Data() == buffer) << ";" << filledValues(array) << "\n";
}
static void changedBase(size_t size) {
  NCollection_Array2<UninitCell> array(size_t(2), size_t(2));
  array.ChangeArray1().Resize(size, false);
  const auto* pointer = array.Data();
  fillDefined(array);
  std::cout << "changed_base_" << size << "=" << matrixBounds(array) << ";"
      << array.IsDeletable() << ";" << (pointer == array.Data()) << ";" << filledValues(array) << "\n";
}
int main() {
  for (size_t size : {0, 1, 4}) linearSize(size);
  linearBounds(-2, 1); linearBounds(3, 2);
  matrixSize(0, 3); matrixSize(2, 0); matrixSize(2, 3);
  matrixBoundsCase(-2, -1, 3, 4); matrixBoundsCase(-2, -3, 3, 4);
  borrowedLinear(); borrowedMatrix();
  for (size_t size : {0, 2, 6}) changedBase(size);
}
