#include <NCollection_Array2.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
using Trace = std::vector<std::string>;
struct InternalEmplaceItem {
  int value = 0, mode = 0;
  Trace* trace = nullptr;
  InternalEmplaceItem() = default;
  InternalEmplaceItem(const InternalEmplaceItem& donor)
      : value(donor.value * 10), mode(donor.mode), trace(donor.trace) {
    record("C" + std::to_string(donor.value), 1);
    record("K" + std::to_string(value), 2);
  }
  InternalEmplaceItem(InternalEmplaceItem&& donor)
      : value(donor.value * 10), mode(donor.mode), trace(donor.trace) {
    record("M" + std::to_string(donor.value), 1);
    donor.value = -donor.value;
    record("K" + std::to_string(value), 2);
  }
  void record(const std::string& event, int failure) {
    trace->push_back(event);
    if (mode == failure) throw std::runtime_error("mode " + std::to_string(mode) + ": expected success");
  }
  InternalEmplaceItem& operator=(InternalEmplaceItem&& donor) {
    donor.record("A" + std::to_string(value) + ":" + std::to_string(donor.value), 3);
    value = donor.value; donor.value = -donor.value;
    donor.record("L" + std::to_string(value), 4);
    return *this;
  }
  ~InternalEmplaceItem() { if (trace) trace->push_back("X" + std::to_string(value)); }
};
struct Request { int target, source; bool moved; int mode, invalid; };
static std::pair<int,int> coordinate(const Request& request, bool source) {
  int offset = source ? request.source : request.target;
  int row = -2 + offset / 3, col = 3 + offset % 3;
  if (!source && (request.invalid == 1 || request.invalid == 3)) row = 40;
  if (source && (request.invalid == 2 || request.invalid == 3)) row = 50;
  if (!source && (request.invalid == 4 || request.invalid == 6)) col = 41;
  if (source && (request.invalid == 5 || request.invalid == 6)) col = 51;
  return {row, col};
}
static int linearIndex(const Request& request, bool source) {
  int row = coordinate(request, source).first;
  return row >= 40 ? row : -2 + (source ? request.source : request.target);
}
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
static std::string bounds(const NCollection_Array1<InternalEmplaceItem>& array) {
  return std::to_string(array.Lower()) + ":" + std::to_string(array.Upper());
}
static std::string bounds(const NCollection_Array2<InternalEmplaceItem>& array) {
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
static InternalEmplaceItem& emplaceFrom(NCollection_Array1<InternalEmplaceItem>& array, const Request& request) {
  int target = linearIndex(request, false), source = linearIndex(request, true);
  if (request.moved) return array.EmplaceValue(target, std::move(array.ChangeValue(source)));
  return array.EmplaceValue(target, array.Value(source));
}
static InternalEmplaceItem& emplaceFrom(NCollection_Array2<InternalEmplaceItem>& array, const Request& request) {
  auto target = coordinate(request, false), source = coordinate(request, true);
  if (request.moved) return array.EmplaceValue(target.first, target.second, std::move(array.ChangeValue(source.first, source.second)));
  return array.EmplaceValue(target.first, target.second, array.Value(source.first, source.second));
}
template<class Array>
static std::string exercise(Array& array, const Request& request, const Trace& trace) {
  const auto* pointer = array.Data();
  int error = 0;
  bool same = false;
  try { same = &emplaceFrom(array, request) == array.Data() + request.target; array.ChangeAt(0).trace->push_back("R"); }
  catch(const Standard_OutOfRange& failure) {
    error = std::string(failure.what()).find("EmplaceValue") == std::string::npos ? 2 : 3;
  }
  catch(const std::runtime_error&) { error = 1; }
  return bounds(array) + ";" + values(array) + ";" + std::to_string(error) + ";"
      + std::to_string(pointer == array.Data()) + ";" + std::to_string(same) + ";" + joined(trace);
}
static std::string label(bool matrix, bool owned, const Request& request) {
  if (request.invalid != 0) return "invalid_" + std::to_string(matrix) + "_" + std::to_string(request.moved) + "_" + std::to_string(request.invalid);
  return "case_" + std::to_string(matrix) + "_" + std::to_string(owned) + "_" + std::to_string(request.moved)
      + "_" + std::to_string(request.target) + "_" + std::to_string(request.source) + "_" + std::to_string(request.mode);
}
static void ordinaryCase(bool owned, const Request& request) {
  Trace firstTrace, secondTrace;
  InternalEmplaceItem firstBuffer[6], secondBuffer[6];
  NCollection_Array1<InternalEmplaceItem> first(firstBuffer[0], -2, 3, !owned);
  NCollection_Array2<InternalEmplaceItem> second = owned
      ? NCollection_Array2<InternalEmplaceItem>(-2, -1, 3, 5)
      : NCollection_Array2<InternalEmplaceItem>(secondBuffer[0], -2, -1, 3, 5);
  initialize(first, firstTrace, request.mode); initialize(second, secondTrace, request.mode);
  if (request.invalid <= 3) std::cout << label(false, owned, request) << "=" << exercise(first, request, firstTrace) << "\n";
  std::cout << label(true, owned, request) << "=" << exercise(second, request, secondTrace) << "\n";
}
static void modeCases(int mode, bool moved) {
  for (const auto& pair : {std::pair<int,int>{0,2}, {2,0}, {2,2}}) {
    Request request{pair.first, pair.second, moved, mode, 0};
    ordinaryCase(false, request); ordinaryCase(true, request);
  }
}
static void shortBase(bool moved) {
  Trace trace;
  NCollection_Array2<InternalEmplaceItem> array(-2, -1, 3, 5);
  NCollection_Array1<InternalEmplaceItem> shortArray(0, 0);
  array.NCollection_Array1<InternalEmplaceItem>::Move(shortArray);
  initialize(array, trace, 0);
  Request request{2, 0, moved, 0, 0};
  try { emplaceFrom(array, request); }
  catch (const Standard_OutOfRange&) { trace.push_back("E"); }
  std::cout << "short_" << moved << "=" << values(array) << ";" << joined(trace) << "\n";
}
int main() {
  shortBase(false); shortBase(true);
  for (int mode = 0; mode != 5; ++mode) { modeCases(mode, false); modeCases(mode, true); }
  for (int invalid = 1; invalid != 7; ++invalid) {
    ordinaryCase(true, {0, 2, false, 1, invalid}); ordinaryCase(true, {0, 2, true, 1, invalid});
  }
}
