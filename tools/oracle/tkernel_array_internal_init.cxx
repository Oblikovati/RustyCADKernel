#include <NCollection_Array2.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
using Trace = std::vector<std::string>;
struct InternalDonorItem {
  int value = 0, mode = 0;
  Trace* trace = nullptr;
  InternalDonorItem() = default;
  InternalDonorItem(const InternalDonorItem&) = delete;
  void assignSelf() {
    trace->push_back("S" + std::to_string(value));
    if (mode == 2) throw std::runtime_error("self mode 2: expected no early failure");
    value += 100;
    if (mode == 3) throw std::runtime_error("self mode 3: expected no late failure");
  }
  InternalDonorItem& operator=(const InternalDonorItem& donor) {
    if (this == &donor) { assignSelf(); return *this; }
    int old = value;
    trace->push_back("Q" + std::to_string(old) + ":" + std::to_string(donor.value));
    if ((donor.mode == 1 && old == 2) || (donor.mode == 4 && old == 4))
      throw std::runtime_error("expected no early assignment failure");
    value = donor.value;
    if (donor.mode == 5 && old == 4) throw std::runtime_error("expected no late assignment failure");
    return *this;
  }
  ~InternalDonorItem() { if (trace) trace->push_back("X" + std::to_string(value)); }
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
static std::string bounds(const NCollection_Array1<InternalDonorItem>& array) {
  return std::to_string(array.Lower()) + ":" + std::to_string(array.Upper());
}
static std::string bounds(const NCollection_Array2<InternalDonorItem>& array) {
  return std::to_string(array.LowerRow()) + ":" + std::to_string(array.UpperRow()) + ":"
      + std::to_string(array.LowerCol()) + ":" + std::to_string(array.UpperCol()) + ":"
      + std::to_string(array.Lower()) + ":" + std::to_string(array.Upper());
}
template<class Array>
static void initialize(Array& array, Trace& trace, int mode) {
  for (size_t i = 0; i < array.Size(); ++i) {
    array.ChangeAt(i).value = int(i) + 1;
    array.ChangeAt(i).trace = &trace; array.ChangeAt(i).mode = mode;
  }
}
static void initFrom(NCollection_Array1<InternalDonorItem>& array, int offset, int invalid) {
  array.Init(array.Value(invalid != 0 ? 4 : -2 + offset));
}
static void initFrom(NCollection_Array2<InternalDonorItem>& array, int offset, int invalid) {
  int row = invalid == 1 ? 0 : -2 + offset / 3;
  int col = invalid == 2 ? 6 : 3 + offset % 3;
  array.Init(array.Value(row, col));
}
template<class Array>
static std::string exercise(Array& array, int offset, int invalid, const Trace& trace) {
  const auto* pointer = array.Data();
  int error = 0;
  try { initFrom(array, offset, invalid); }
  catch(const Standard_OutOfRange&) { error = 2; }
  catch(const std::runtime_error&) { error = 1; }
  return bounds(array) + ";" + values(array) + ";" + std::to_string(error) + ";"
      + std::to_string(pointer == array.Data()) + ";" + joined(trace);
}
static void ordinaryCase(bool owned, int offset, int mode) {
  Trace firstTrace, secondTrace;
  InternalDonorItem firstBuffer[6], secondBuffer[6];
  NCollection_Array1<InternalDonorItem> first(firstBuffer[0], -2, 3, !owned);
  NCollection_Array2<InternalDonorItem> second = owned
      ? NCollection_Array2<InternalDonorItem>(-2, -1, 3, 5)
      : NCollection_Array2<InternalDonorItem>(secondBuffer[0], -2, -1, 3, 5);
  initialize(first, firstTrace, mode); initialize(second, secondTrace, mode);
  std::cout << "case_0_" << owned << "_" << offset << "_" << mode << "=" << exercise(first, offset, 0, firstTrace) << "\n";
  std::cout << "case_1_" << owned << "_" << offset << "_" << mode << "=" << exercise(second, offset, 0, secondTrace) << "\n";
}
static void invalidCases() {
  Trace trace;
  NCollection_Array1<InternalDonorItem> first(-2, 3);
  NCollection_Array2<InternalDonorItem> second(-2, -1, 3, 5);
  initialize(first, trace, 1); initialize(second, trace, 1);
  std::cout << "invalid_0_1=" << exercise(first, 2, 1, trace) << "\n";
  std::cout << "invalid_1_1=" << exercise(second, 2, 1, trace) << "\n";
  std::cout << "invalid_1_2=" << exercise(second, 2, 2, trace) << "\n";
}
static void emptyCases() {
  Trace trace;
  NCollection_Array1<InternalDonorItem> first;
  NCollection_Array2<InternalDonorItem> second;
  std::cout << "empty_0=" << exercise(first, 0, 0, trace) << "\n";
  std::cout << "empty_1=" << exercise(second, 0, 0, trace) << "\n";
}
static void flatCase(size_t length, int offset) {
  Trace trace;
  InternalDonorItem buffer[8];
  NCollection_Array1<InternalDonorItem> view(buffer, length);
  NCollection_Array2<InternalDonorItem> array(-2, -1, 3, 5);
  array.ChangeArray1().Move(view);
  initialize(array, trace, 0);
  std::cout << "flat_" << length << "=" << exercise(array, offset, 0, trace) << "\n";
}
int main() {
  for (int mode = 0; mode != 6; ++mode) { ordinaryCase(false, 2, mode); ordinaryCase(true, 2, mode); }
  ordinaryCase(false, 0, 0); ordinaryCase(true, 0, 0);
  ordinaryCase(false, 5, 0); ordinaryCase(true, 5, 0);
  invalidCases(); emptyCases(); flatCase(8, 2); flatCase(2, 0);
}
