#include <NCollection_Array2.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
using Trace = std::vector<std::string>;
struct CopyAssignmentOnly {
  int value = 0, mode = 0;
  Trace* trace = nullptr;
  CopyAssignmentOnly() = default;
  CopyAssignmentOnly(const CopyAssignmentOnly&) = delete;
  CopyAssignmentOnly& operator=(const CopyAssignmentOnly& donor) {
    int previous = value;
    trace->push_back("Q" + std::to_string(previous) + ":" + std::to_string(donor.value));
    if (donor.mode == 1 && previous == 2) throw std::runtime_error("early assignment failure");
    value = donor.value;
    if (donor.mode == 2 && previous == 2) throw std::runtime_error("late assignment failure");
    return *this;
  }
  ~CopyAssignmentOnly() { if (trace) trace->push_back("X" + std::to_string(value)); }
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
static void assign(NCollection_Array1<CopyAssignmentOnly>& array, const CopyAssignmentOnly& donor, int invalid) {
  array.SetValue(invalid != 0 ? 2 : -1, donor);
}
static void assign(NCollection_Array2<CopyAssignmentOnly>& array, const CopyAssignmentOnly& donor, int invalid) {
  array.SetValue(invalid == 1 ? 0 : -2, invalid == 2 ? 5 : 4, donor);
}
template<class Array>
static std::string exercise(Array& array, const CopyAssignmentOnly& donor, bool init, int invalid = 0) {
  const auto* address = array.Data();
  int error = 0;
  try { if (init) array.Init(donor); else assign(array, donor, invalid); }
  catch(const Standard_OutOfRange&) { error = 2; }
  catch(const std::runtime_error&) { error = 1; }
  return values(array) + ";" + std::to_string(donor.value) + ";" + std::to_string(error)
      + ";" + std::to_string(address == array.Data()) + ";" + joined(*donor.trace);
}
template<class Array>
static void external(Array& array, Trace& trace, int matrix, int owned, int mode, bool init) {
  initialize(array, trace);
  const std::string label = "case_" + std::to_string(matrix) + "_" + std::to_string(owned)
      + "_" + std::to_string(init) + "_" + std::to_string(mode);
  {
    CopyAssignmentOnly donor;
    donor.value = 9; donor.mode = mode; donor.trace = &trace;
    std::cout << label << "=" << exercise(array, donor, init) << "\n";
  }
  std::cout << label << "_dropped=" << joined(trace) << "\n";
}
static void externalCases(int mode, bool init) {
  Trace firstTrace, secondTrace, thirdTrace, fourthTrace;
  NCollection_Array1<CopyAssignmentOnly> first(-2, 1);
  NCollection_Array2<CopyAssignmentOnly> second(-2, -1, 3, 4);
  CopyAssignmentOnly firstBuffer[4], secondBuffer[4];
  NCollection_Array1<CopyAssignmentOnly> third(firstBuffer[0], -2, 1);
  NCollection_Array2<CopyAssignmentOnly> fourth(secondBuffer[0], -2, -1, 3, 4);
  external(first, firstTrace, 0, 1, mode, init);
  external(second, secondTrace, 1, 1, mode, init);
  external(third, thirdTrace, 0, 0, mode, init);
  external(fourth, fourthTrace, 1, 0, mode, init);
}
static void disjointCases(bool init) {
  Trace firstTrace, secondTrace;
  CopyAssignmentOnly firstBuffer[5], secondBuffer[5];
  NCollection_Array1<CopyAssignmentOnly> first(firstBuffer[0], -2, 1);
  NCollection_Array2<CopyAssignmentOnly> second(secondBuffer[0], -2, -1, 3, 4);
  initialize(first, firstTrace); initialize(second, secondTrace);
  firstBuffer[4].value = secondBuffer[4].value = 9;
  firstBuffer[4].trace = &firstTrace; secondBuffer[4].trace = &secondTrace;
  std::cout << "disjoint_0_" << init << "=" << exercise(first, firstBuffer[4], init) << "\n";
  std::cout << "disjoint_1_" << init << "=" << exercise(second, secondBuffer[4], init) << "\n";
}
static void invalidCases() {
  Trace trace;
  NCollection_Array1<CopyAssignmentOnly> first(-2, 1);
  NCollection_Array2<CopyAssignmentOnly> second(-2, -1, 3, 4);
  CopyAssignmentOnly donor;
  initialize(first, trace); initialize(second, trace);
  donor.value = 9; donor.mode = 1; donor.trace = &trace;
  std::cout << "invalid_0_1=" << exercise(first, donor, false, 1) << "\n";
  std::cout << "invalid_1_1=" << exercise(second, donor, false, 1) << "\n";
  std::cout << "invalid_1_2=" << exercise(second, donor, false, 2) << "\n";
}
static void flatCase(size_t length) {
  Trace trace;
  NCollection_Array2<CopyAssignmentOnly> array(-2, -1, 3, 4);
  CopyAssignmentOnly buffer[5], donor;
  NCollection_Array1<CopyAssignmentOnly> view(buffer, length);
  initialize(view, trace);
  array.ChangeArray1().Move(view);
  donor.value = 9; donor.trace = &trace;
  trace.clear();
  std::cout << "flat_" << length << "=" << exercise(array, donor, true) << "\n";
}
int main() {
  for (int mode = 0; mode != 3; ++mode) { externalCases(mode, false); externalCases(mode, true); }
  disjointCases(false); disjointCases(true); invalidCases();
  flatCase(5); flatCase(2); flatCase(0);
}
