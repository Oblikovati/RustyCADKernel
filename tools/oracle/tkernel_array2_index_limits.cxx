#include <NCollection_Array2.hxx>
#include <Standard_OutOfRange.hxx>
#include <climits>
#include <iostream>
#include <string>
struct BoundsCase { const char* label; int rowLower, colLower, row, col; };
static int operation(NCollection_Array2<int>& array, int row, int col, int op) {
  const NCollection_Array2<int>& constant = array;
  if (op == 0) return array.Value(row, col);
  if (op == 1) return (array.ChangeValue(row, col) += 100);
  if (op == 2) return constant(row, col);
  if (op == 3) return (array(row, col) += 100);
  if (op == 4) { const int donor = 77; array.SetValue(row, col, donor); return donor; }
  if (op == 5) { int donor = 78; array.SetValue(row, col, std::move(donor)); return donor; }
  return array.EmplaceValue(row, col, 79);
}
static std::string definedValues(const NCollection_Array2<int>& array) {
  std::string result;
  for (size_t index = 0; index < array.Size(); ++index) {
    result += (index == 0 ? "" : ",") + std::to_string(array.At(index));
  }
  return result;
}
static void exercise(const BoundsCase& sample, int op, bool owned) {
  int buffer[9] = {1,2,3,4,5,6,7,8,9};
  NCollection_Array2<int> array = owned ? NCollection_Array2<int>(size_t(3), size_t(3)) : NCollection_Array2<int>(buffer, size_t(3), size_t(3));
  for (size_t index = 0; index < 9; ++index) array.ChangeAt(index) = buffer[index];
  const auto* pointer = array.Data();
  array.UpdateLowerRow(sample.rowLower); array.UpdateLowerCol(sample.colLower);
  int result = 0, error = 0;
  try { result = operation(array, sample.row, sample.col, op); }
  catch (const Standard_OutOfRange&) { error = 1; }
  std::cout << sample.label << "_" << op << "_" << owned << "=" << result << ";" << error << ";"
      << (array.Data() == pointer) << ";" << array.IsDeletable() << ";" << definedValues(array) << "\n";
}
static void hugeEmptyShape() {
  NCollection_Array2<int> array(size_t(-1), size_t(0));
  array.UpdateLowerRow(-2);
  int error = 0;
  try { (void)array.Value(0, 0); }
  catch (const Standard_OutOfRange&) { error = 1; }
  std::cout << "empty_huge_shape=" << error << ";" << array.Size() << "\n";
}
int main() {
  hugeEmptyShape();
  const BoundsCase cases[] = {
    {"row_first", INT_MAX-1, 0, INT_MAX-1, 0}, {"row_last_reachable", INT_MAX-1, 0, INT_MAX, 1},
    {"col_first", 0, INT_MAX-1, 0, INT_MAX-1}, {"col_last_reachable", 0, INT_MAX-1, 1, INT_MAX},
    {"both_first", INT_MAX-1, INT_MAX-1, INT_MAX-1, INT_MAX-1},
    {"both_last_reachable", INT_MAX-1, INT_MAX-1, INT_MAX, INT_MAX},
    {"negative_valid", -2, -3, -1, -2}, {"minimum_valid", INT_MIN, INT_MIN, INT_MIN+1, INT_MIN+1},
    {"row_negative_invalid", INT_MAX-1, 0, INT_MIN, 0},
    {"col_negative_invalid", 0, INT_MAX-1, 0, INT_MIN},
    {"minimum_row_invalid", INT_MIN, INT_MIN, INT_MAX, INT_MIN},
    {"minimum_col_invalid", INT_MIN, INT_MIN, INT_MIN, INT_MAX}
  };
  for (const auto& sample : cases) for (int op = 0; op != 7; ++op) { exercise(sample, op, false); exercise(sample, op, true); }
}
