#include <NCollection_Array2.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
using Trace = std::vector<std::string>;
struct InternalSetItem {
  int value = 0, mode = 0;
  Trace* trace = nullptr;
  InternalSetItem() = default;
  InternalSetItem(const InternalSetItem&) = delete;
  void record(const std::string& event, int failure) {
    trace->push_back(event);
    if (failure == 1) throw std::runtime_error("mode 1: expected no early failure");
  }
  void assignSelf(bool moved) {
    record(std::string(moved ? "MS" : "CS") + std::to_string(value), mode);
    value = moved ? -value * 10 : value + 100;
    if (mode == 2) throw std::runtime_error("self mode 2: expected no late failure");
  }
  InternalSetItem& operator=(const InternalSetItem& donor) {
    if (this == &donor) { assignSelf(false); return *this; }
    record("C" + std::to_string(value) + ":" + std::to_string(donor.value), donor.mode);
    value = donor.value;
    if (donor.mode == 2) throw std::runtime_error("mode 2: expected no late failure");
    return *this;
  }
  InternalSetItem& operator=(InternalSetItem&& donor) {
    if (this == &donor) { assignSelf(true); return *this; }
    record("M" + std::to_string(value) + ":" + std::to_string(donor.value), donor.mode);
    value = donor.value; donor.value = -donor.value;
    if (donor.mode == 2) throw std::runtime_error("mode 2: expected no late failure");
    return *this;
  }
  ~InternalSetItem() { if (trace) trace->push_back("X" + std::to_string(value)); }
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
static std::string bounds(const NCollection_Array1<InternalSetItem>& array) {
  return std::to_string(array.Lower()) + ":" + std::to_string(array.Upper());
}
static std::string bounds(const NCollection_Array2<InternalSetItem>& array) {
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
static void setFrom(NCollection_Array1<InternalSetItem>& array, const Request& request) {
  int target = linearIndex(request, false), source = linearIndex(request, true);
  if (request.moved) array.SetValue(target, std::move(array.ChangeValue(source)));
  else array.SetValue(target, array.Value(source));
}
static void setFrom(NCollection_Array2<InternalSetItem>& array, const Request& request) {
  auto target = coordinate(request, false), source = coordinate(request, true);
  if (request.moved) array.SetValue(target.first, target.second, std::move(array.ChangeValue(source.first, source.second)));
  else array.SetValue(target.first, target.second, array.Value(source.first, source.second));
}
template<class Array>
static std::string exercise(Array& array, const Request& request, const Trace& trace) {
  const auto* pointer = array.Data();
  int error = 0;
  try { setFrom(array, request); }
  catch(const Standard_OutOfRange& failure) {
    error = std::string(failure.what()).find("SetValue") == std::string::npos ? 2 : 3;
  }
  catch(const std::runtime_error&) { error = 1; }
  return bounds(array) + ";" + values(array) + ";" + std::to_string(error) + ";"
      + std::to_string(pointer == array.Data()) + ";" + joined(trace);
}
static std::string label(bool matrix, bool owned, const Request& request) {
  if (request.invalid != 0) return "invalid_" + std::to_string(matrix) + "_" + std::to_string(request.moved) + "_" + std::to_string(request.invalid);
  return "case_" + std::to_string(matrix) + "_" + std::to_string(owned) + "_" + std::to_string(request.moved)
      + "_" + std::to_string(request.target) + "_" + std::to_string(request.source) + "_" + std::to_string(request.mode);
}
static void ordinaryCase(bool owned, const Request& request) {
  Trace firstTrace, secondTrace;
  InternalSetItem firstBuffer[6], secondBuffer[6];
  NCollection_Array1<InternalSetItem> first(firstBuffer[0], -2, 3, !owned);
  NCollection_Array2<InternalSetItem> second = owned
      ? NCollection_Array2<InternalSetItem>(-2, -1, 3, 5)
      : NCollection_Array2<InternalSetItem>(secondBuffer[0], -2, -1, 3, 5);
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
int main() {
  for (int mode = 0; mode != 3; ++mode) { modeCases(mode, false); modeCases(mode, true); }
  for (int invalid = 1; invalid != 7; ++invalid) {
    ordinaryCase(true, {0, 2, false, 1, invalid}); ordinaryCase(true, {0, 2, true, 1, invalid});
  }
}
