#include <NCollection_Array2.hxx>
#include <iostream>
#include <string>
#include <vector>
using Trace = std::vector<int>;
struct SurfaceCell {
  int value = 0;
  Trace* trace = nullptr;
  SurfaceCell() = default;
  SurfaceCell(const SurfaceCell&) = delete;
  ~SurfaceCell() { if (trace) trace->push_back(value); }
};
static std::string traceRow(const Trace& trace) {
  std::string result;
  for (int value : trace) result += (result.empty() ? "" : ",") + std::to_string(value);
  return result;
}
template<class Array>
static void attachTrace(Array& array, Trace& trace) {
  for (size_t i = 0; i < array.Size(); ++i) { array.ChangeAt(i).value = int(i) + 1; array.ChangeAt(i).trace = &trace; }
}
template<class T>
static std::string linearQueries(const NCollection_Array1<T>& array) {
  return std::to_string(array.Size()) + ":" + std::to_string(array.Length()) + ":"
      + std::to_string(array.IsEmpty()) + ":" + std::to_string(array.Lower()) + ":"
      + std::to_string(array.Upper()) + ":" + std::to_string(array.IsDeletable());
}
template<class T>
static std::string matrixQueries(const NCollection_Array2<T>& array) {
  return std::to_string(array.Size()) + ":" + std::to_string(array.Length()) + ":"
      + std::to_string(array.IsEmpty()) + ":" + std::to_string(array.LowerRow()) + ":"
      + std::to_string(array.UpperRow()) + ":" + std::to_string(array.LowerCol()) + ":"
      + std::to_string(array.UpperCol()) + ":" + std::to_string(array.IsDeletable()) + ":"
      + std::to_string(array.NbRows()) + ":" + std::to_string(array.NbColumns()) + ":"
      + std::to_string(array.RowSize()) + ":" + std::to_string(array.ColSize()) + ":"
      + std::to_string(array.RowLength()) + ":" + std::to_string(array.ColLength());
}
static void moveLinear(bool owned) {
  Trace trace; SurfaceCell buffer[4];
  NCollection_Array1<SurfaceCell> source(buffer[0], -2, 1, !owned);
  attachTrace(source, trace);
  const auto* pointer = source.Data();
  {
    NCollection_Array1<SurfaceCell> target(std::move(source));
    std::cout << "move_linear_" << owned << "=" << linearQueries(source) << ";" << linearQueries(target)
        << ";" << (source.Data() == nullptr) << ";" << (target.Data() == pointer) << ";" << traceRow(trace) << "\n";
  }
  std::cout << "drop_linear_" << owned << "=" << traceRow(trace) << "\n";
}
static void moveMatrix(bool owned) {
  Trace trace; SurfaceCell buffer[4];
  NCollection_Array2<SurfaceCell> source = owned ? NCollection_Array2<SurfaceCell>(-2, -1, 3, 4)
      : NCollection_Array2<SurfaceCell>(buffer[0], -2, -1, 3, 4);
  attachTrace(source, trace);
  const auto* pointer = source.Data();
  {
    NCollection_Array2<SurfaceCell> target(std::move(source));
    std::cout << "move_matrix_" << owned << "=" << matrixQueries(source) << ";" << matrixQueries(target)
        << ";" << (source.Data() == nullptr) << ";" << (target.Data() == pointer) << ";" << traceRow(trace) << "\n";
  }
  std::cout << "drop_matrix_" << owned << "=" << traceRow(trace) << "\n";
}
static void staticHelpers() {
  using First = NCollection_Array1<int>; using Second = NCollection_Array2<int>;
  std::cout << "static=" << First::RangeSize(-2, 3) << ":" << First::RangeSize(5, 3) << ":"
      << Second::RangeSize(-2, 3) << ":" << Second::ArraySize(size_t(-1), 2) << ":"
      << Second::UpperForSize(4, 0) << ":" << Second::UpperForSize(3, size_t(-1)) << ":"
      << Second::BeginPosition(-2, -1, 3, 5) << ":" << Second::LastPosition(-2, -1, 3, 5) << "\n";
}
static void defaultQueries() {
  NCollection_Array1<int> first; NCollection_Array2<int> second;
  std::cout << "default_linear=" << linearQueries(first) << "\n";
  std::cout << "default_matrix=" << matrixQueries(second) << "\n";
}
static void boundUpdates() {
  NCollection_Array1<int> first(-2, 1);
  NCollection_Array2<int> second(-2, -1, 3, 5);
  first.UpdateLowerBound(7); first.UpdateUpperBound(14);
  second.UpdateLowerRow(7); second.UpdateUpperRow(14);
  second.UpdateLowerCol(-7); second.UpdateUpperCol(-2);
  std::cout << "updated_linear=" << linearQueries(first) << "\n";
  std::cout << "updated_matrix=" << matrixQueries(second) << ";" << linearQueries(second.Array1()) << "\n";
}
int main() {
  defaultQueries(); staticHelpers(); boundUpdates();
  moveLinear(false); moveLinear(true); moveMatrix(false); moveMatrix(true);
}
