#include <NCollection_Array2.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
using Trace = std::vector<std::string>;
struct AssignmentItem {
  int value = 0, mode = 0;
  Trace* trace = nullptr;
  AssignmentItem() = default;
  AssignmentItem(const AssignmentItem&) = delete;
  AssignmentItem& operator=(AssignmentItem&& donor) {
    trace->push_back("A" + std::to_string(value) + ":" + std::to_string(donor.value));
    if (donor.mode == 1) throw std::runtime_error("mode 1: expected no early failure");
    value = donor.value; donor.value = -donor.value;
    if (donor.mode == 2) throw std::runtime_error("mode 2: expected no late failure");
    return *this;
  }
  ~AssignmentItem() { if (trace) trace->push_back("X" + std::to_string(value)); }
};
struct CopyAssignmentItem : AssignmentItem {
  CopyAssignmentItem& operator=(const CopyAssignmentItem& donor) {
    trace->push_back("Q" + std::to_string(value) + ":" + std::to_string(donor.value));
    value = donor.value;
    return *this;
  }
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
template<class Array>
static void initialize(Array& array, Trace& trace) {
  for (size_t i = 0; i < array.Size(); ++i) {
    array.ChangeAt(i).value = int(i) + 1;
    array.ChangeAt(i).trace = &trace;
  }
}
template<class Item>
static void assign(NCollection_Array1<Item>& array, Item& donor, int invalid) {
  array.SetValue(invalid == 1 ? -3 : (invalid == 2 ? 2 : -1), std::move(donor));
}
template<class Item>
static void assign(NCollection_Array2<Item>& array, Item& donor, int invalid) {
  int row = invalid == 1 ? -3 : (invalid == 2 ? 0 : -2);
  int col = invalid == 3 ? 2 : (invalid == 4 ? 5 : 4);
  array.SetValue(row, col, std::move(donor));
}
template<class Array, class Item>
static std::string exercise(Array& array, Item& donor, int invalid) {
  const auto* address = &array.At(1);
  int error = 0;
  try { assign(array, donor, invalid); }
  catch(const Standard_OutOfRange&) { error = 2; }
  catch(const std::runtime_error&) { error = 1; }
  return values(array) + ";" + std::to_string(donor.value) + ";" + std::to_string(error)
      + ";" + std::to_string(address == &array.At(1)) + ";" + joined(*donor.trace);
}
template<class Array>
static void external(const std::string& label, Array& array, Trace& trace, int mode, int invalid = 0) {
  initialize(array, trace);
  {
    typename Array::value_type donor;
    donor.value = 9; donor.mode = mode; donor.trace = &trace;
    std::cout << label << "=" << exercise(array, donor, invalid) << "\n";
  }
  std::cout << label << "_dropped=" << joined(trace) << "\n";
}
template<class Item>
static void externalCases(int mode) {
  Trace firstTrace, secondTrace, thirdTrace, fourthTrace;
  NCollection_Array1<Item> first(-2, 1);
  NCollection_Array2<Item> second(-2, -1, 3, 4);
  Item firstBuffer[4], secondBuffer[4];
  NCollection_Array1<Item> third(firstBuffer[0], -2, 1);
  NCollection_Array2<Item> fourth(secondBuffer[0], -2, -1, 3, 4);
  external("external_0_1_" + std::to_string(mode), first, firstTrace, mode);
  external("external_1_1_" + std::to_string(mode), second, secondTrace, mode);
  external("external_0_0_" + std::to_string(mode), third, thirdTrace, mode);
  external("external_1_0_" + std::to_string(mode), fourth, fourthTrace, mode);
}
template<class Item>
static void disjointCases(int mode) {
  Trace firstTrace, secondTrace;
  Item firstBuffer[5], secondBuffer[5];
  NCollection_Array1<Item> first(firstBuffer[0], -2, 1);
  NCollection_Array2<Item> second(secondBuffer[0], -2, -1, 3, 4);
  initialize(first, firstTrace); initialize(second, secondTrace);
  firstBuffer[4].value = secondBuffer[4].value = 9;
  firstBuffer[4].mode = secondBuffer[4].mode = mode;
  firstBuffer[4].trace = &firstTrace; secondBuffer[4].trace = &secondTrace;
  std::cout << "disjoint_0_" << mode << "=" << exercise(first, firstBuffer[4], 0) << "\n";
  std::cout << "disjoint_1_" << mode << "=" << exercise(second, secondBuffer[4], 0) << "\n";
}
static void invalidCase(bool matrix, int invalid) {
  Trace trace;
  const std::string label = "invalid_" + std::to_string(matrix) + "_" + std::to_string(invalid);
  if (matrix) {
    NCollection_Array2<AssignmentItem> array(-2, -1, 3, 4);
    external(label, array, trace, 1, invalid);
    return;
  }
  NCollection_Array1<AssignmentItem> array(-2, 1);
  external(label, array, trace, 1, invalid);
}
int main() {
  for (int mode = 0; mode != 3; ++mode) {
    externalCases<AssignmentItem>(mode); disjointCases<AssignmentItem>(mode);
  }
  externalCases<CopyAssignmentItem>(3); disjointCases<CopyAssignmentItem>(3);
  for (int invalid = 1; invalid != 3; ++invalid) invalidCase(false, invalid);
  for (int invalid = 1; invalid != 5; ++invalid) invalidCase(true, invalid);
}
