#include <NCollection_Array2.hxx>
#include <Standard_DimensionMismatch.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <vector>
using Trace = std::vector<std::string>;
struct AssignOnlyItem {
  int value = 0, mode = 0;
  Trace* trace = nullptr;
  AssignOnlyItem() = default;
  AssignOnlyItem(const AssignOnlyItem&) = delete;
  AssignOnlyItem& operator=(const AssignOnlyItem& donor) {
    int old = value;
    trace->push_back("Q" + std::to_string(old) + ":" + std::to_string(donor.value));
    if (donor.mode == 1 && old == 2) throw std::runtime_error("early assignment failure");
    value = donor.value;
    if (donor.mode == 2 && old == 2) throw std::runtime_error("late assignment failure");
    return *this;
  }
  ~AssignOnlyItem() { if (trace) trace->push_back("X" + std::to_string(value)); }
};
static std::string joined(const Trace& trace) {
  std::string output;
  for (const auto& event : trace) output += (output.empty() ? "" : ",") + event;
  return output;
}
template<class Array>
static std::string values(const Array& array) {
  Trace output;
  for (size_t i = 0; i < array.Size(); ++i) output.push_back(std::to_string(array.At(i).value));
  return joined(output);
}
static std::string bounds(const NCollection_Array1<AssignOnlyItem>& array) {
  return std::to_string(array.Lower()) + ":" + std::to_string(array.Upper());
}
static std::string bounds(const NCollection_Array2<AssignOnlyItem>& array) {
  return std::to_string(array.LowerRow()) + ":" + std::to_string(array.UpperRow()) + ":"
      + std::to_string(array.LowerCol()) + ":" + std::to_string(array.UpperCol()) + ":"
      + std::to_string(array.Lower()) + ":" + std::to_string(array.Upper());
}
template<class Array>
static void initialize(Array& array, Trace& trace, bool source, int mode = 0) {
  for (size_t i = 0; i < array.Size(); ++i) {
    array.ChangeAt(i).value = int(i) + (source ? 11 : 1);
    array.ChangeAt(i).trace = &trace; array.ChangeAt(i).mode = mode;
  }
}
static void rebase(NCollection_Array1<AssignOnlyItem>& array) { array.UpdateLowerBound(7); }
static void rebase(NCollection_Array2<AssignOnlyItem>& array) { array.UpdateLowerRow(7); array.UpdateLowerCol(-4); }
template<bool Matrix> using ArrayType = std::conditional_t<Matrix, NCollection_Array2<AssignOnlyItem>, NCollection_Array1<AssignOnlyItem>>;
template<bool Matrix>
static ArrayType<Matrix> makeArray(bool owned, AssignOnlyItem* buffer) {
  if constexpr (Matrix) {
    if (owned) return ArrayType<Matrix>(-2, -1, 3, 4);
    return ArrayType<Matrix>(buffer[0], -2, -1, 3, 4);
  } else {
    if (owned) return ArrayType<Matrix>(-2, 1);
    return ArrayType<Matrix>(buffer[0], -2, 1);
  }
}
template<class Array>
static std::string exercise(Array& target, const Array& source, const Trace& trace) {
  const auto* pointer = target.Data();
  bool returned = false;
  int error = 0;
  try { returned = &target.CopyValues(source) == &target; }
  catch(const Standard_DimensionMismatch& failure) {
    error = std::string(failure.what()).find("Array2") == std::string::npos ? 2 : 3;
  }
  catch(const std::runtime_error&) { error = 1; }
  return bounds(target) + ";" + values(target) + ";" + values(source) + ";" + std::to_string(error)
      + ";" + std::to_string(returned) + ";" + std::to_string(pointer == target.Data()) + ";" + joined(trace);
}
template<bool Matrix>
static void ordinaryCase(bool targetOwned, bool sourceOwned, int mode) {
  Trace trace;
  AssignOnlyItem targets[4], donors[4];
  auto target = makeArray<Matrix>(targetOwned, targets);
  auto source = makeArray<Matrix>(sourceOwned, donors);
  initialize(target, trace, false); initialize(source, trace, true, mode); rebase(source);
  std::cout << "case_" << Matrix << "_" << targetOwned << "_" << sourceOwned << "_" << mode
      << "=" << exercise(target, source, trace) << "\n";
}
template<bool Matrix>
static void disjointCase() {
  Trace trace;
  AssignOnlyItem buffer[8];
  auto target = makeArray<Matrix>(false, buffer);
  auto source = makeArray<Matrix>(false, buffer + 4);
  initialize(target, trace, false); initialize(source, trace, true); rebase(source);
  std::cout << "disjoint_" << Matrix << "=" << exercise(target, source, trace) << "\n";
}
static void replaceBase(NCollection_Array1<AssignOnlyItem>& array, AssignOnlyItem* buffer, size_t length) {
  NCollection_Array1<AssignOnlyItem> view(buffer, length);
  array.Move(view);
}
static void linearMismatch() {
  Trace trace;
  AssignOnlyItem donors[3];
  auto target = makeArray<false>(true, nullptr);
  auto source = makeArray<false>(true, nullptr);
  replaceBase(source, donors, 3);
  initialize(target, trace, false); initialize(source, trace, true, 1);
  trace.clear();
  std::cout << "mismatch_0_0=" << exercise(target, source, trace) << "\n";
}
static void matrixMismatch(bool shape) {
  Trace trace;
  AssignOnlyItem donors[3];
  auto target = makeArray<true>(true, nullptr);
  NCollection_Array2<AssignOnlyItem> source(7, shape ? 7 : 8, -4, shape ? -1 : -3);
  replaceBase(source.ChangeArray1(), donors, 3);
  initialize(target, trace, false); initialize(source, trace, true, 1);
  trace.clear();
  std::cout << "mismatch_1_" << shape << "=" << exercise(target, source, trace) << "\n";
}
static void flatCase(size_t length) {
  Trace trace;
  AssignOnlyItem targets[5], donors[5];
  auto target = makeArray<true>(true, nullptr);
  auto source = makeArray<true>(true, nullptr);
  replaceBase(target.ChangeArray1(), targets, length); replaceBase(source.ChangeArray1(), donors, length);
  initialize(target, trace, false); initialize(source, trace, true); rebase(source);
  trace.clear();
  std::cout << "flat_" << length << "=" << exercise(target, source, trace) << "\n";
}
int main() {
  for (int mode = 0; mode != 3; ++mode) {
    ordinaryCase<false>(false, false, mode); ordinaryCase<false>(false, true, mode);
    ordinaryCase<false>(true, false, mode); ordinaryCase<false>(true, true, mode);
    ordinaryCase<true>(false, false, mode); ordinaryCase<true>(false, true, mode);
    ordinaryCase<true>(true, false, mode); ordinaryCase<true>(true, true, mode);
  }
  disjointCase<false>(); disjointCase<true>(); linearMismatch(); matrixMismatch(false); matrixMismatch(true);
  flatCase(5); flatCase(2); flatCase(0);
}
