#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

using Trace = std::vector<std::string>;
struct EmplaceItem
{
  int value = 0;
  Trace* trace = nullptr;
  int mode = 0;
  EmplaceItem() = default;
  EmplaceItem(int number, Trace& recorder, int failure, bool record = true)
      : value(number), trace(&recorder), mode(failure)
  {
    if (record) trace->push_back("C" + std::to_string(value));
    if (mode == 1) throw std::runtime_error("constructor mode 1: expected success");
  }
  EmplaceItem(const EmplaceItem&) = delete;
  EmplaceItem& operator=(EmplaceItem&& donor)
  {
    trace->push_back("A" + std::to_string(value) + ":" + std::to_string(donor.value));
    if (donor.mode == 2) throw std::runtime_error("assignment mode 2: expected success");
    value = donor.value;
    donor.value = -donor.value;
    if (donor.mode == 3) throw std::runtime_error("assignment mode 3: expected no late failure");
    return *this;
  }
  ~EmplaceItem()
  {
    if (trace != nullptr) trace->push_back("X" + std::to_string(value));
  }
};

struct CopyEmplaceItem : EmplaceItem
{
  using EmplaceItem::EmplaceItem;
  CopyEmplaceItem& operator=(const CopyEmplaceItem& donor)
  {
    trace->push_back("Q" + std::to_string(value) + ":" + std::to_string(donor.value));
    value = donor.value;
    return *this;
  }
};

static std::string joined(const Trace& events)
{
  std::string output;
  for (const auto& event : events) output += (output.empty() ? "" : ",") + event;
  return output;
}

template<class Item>
static std::string values(const Item* items, size_t size)
{
  Trace output;
  for (size_t i = 0; i < size; ++i) output.push_back(std::to_string(items[i].value));
  return joined(output);
}

template<class Array>
static void initialize(Array& array, Trace& trace)
{
  for (size_t i = 0; i < array.Size(); ++i)
  {
    array.ChangeAt(i).value = int(i) + 1;
    array.ChangeAt(i).trace = &trace;
  }
}

template<class Item>
static Item& emplace(NCollection_Array1<Item>& array, Trace& trace, int mode, int invalid)
{
  return array.EmplaceValue(invalid == 0 ? -1 : 2, 9, trace, mode);
}

template<class Item>
static Item& emplace(NCollection_Array2<Item>& array, Trace& trace, int mode, int invalid)
{
  return array.EmplaceValue(invalid == 1 ? 0 : -2, invalid == 2 ? 5 : 4, 9, trace, mode);
}

template<class Array>
static std::string exercise(Array& array, Trace& trace, int mode, int invalid)
{
  const auto* address = array.Size() > 1 ? array.Data() + 1 : nullptr;
  int error = 0;
  bool same = false;
  try { same = &emplace(array, trace, mode, invalid) == address; trace.push_back("R"); }
  catch (const Standard_OutOfRange&) { error = 2; }
  catch (const std::runtime_error&) { error = 1; }
  return values(array.Data(), array.Size()) + ";" + std::to_string(error) + ";"
       + std::to_string(same) + ";" + joined(trace);
}

static void ownedCases(int mode)
{
  Trace firstTrace, secondTrace;
  NCollection_Array1<EmplaceItem> first(-2, 1);
  NCollection_Array2<EmplaceItem> second(-2, -1, 3, 4);
  initialize(first, firstTrace);
  initialize(second, secondTrace);
  std::cout << "owned_0_" << mode << "=" << exercise(first, firstTrace, mode, 0) << "\n";
  std::cout << "owned_1_" << mode << "=" << exercise(second, secondTrace, mode, 0) << "\n";
}

static void borrowedCases(int mode)
{
  Trace firstTrace, secondTrace;
  EmplaceItem firstBuffer[5], secondBuffer[5];
  NCollection_Array1<EmplaceItem> first(firstBuffer[0], -2, 1);
  NCollection_Array2<EmplaceItem> second(secondBuffer[0], -2, -1, 3, 4);
  initialize(first, firstTrace);
  initialize(second, secondTrace);
  firstBuffer[4].value = secondBuffer[4].value = 5;
  std::cout << "borrowed_0_" << mode << "=" << exercise(first, firstTrace, mode, 0)
            << ";" << values(firstBuffer, 5) << "\n";
  std::cout << "borrowed_1_" << mode << "=" << exercise(second, secondTrace, mode, 0)
            << ";" << values(secondBuffer, 5) << "\n";
}

static void invalidCases()
{
  Trace firstTrace, secondTrace;
  NCollection_Array1<EmplaceItem> first(-2, 1);
  NCollection_Array2<EmplaceItem> second(-2, -1, 3, 4);
  initialize(first, firstTrace);
  initialize(second, secondTrace);
  std::cout << "invalid_0_1=" << exercise(first, firstTrace, 1, 1) << "\n";
  std::cout << "invalid_1_1=" << exercise(second, secondTrace, 1, 1) << "\n";
  std::cout << "invalid_1_2=" << exercise(second, secondTrace, 1, 2) << "\n";
}

static void shortBase()
{
  Trace trace;
  NCollection_Array2<EmplaceItem> array(-2, -1, 3, 4);
  NCollection_Array1<EmplaceItem> empty;
  initialize(array, trace);
  array.NCollection_Array1<EmplaceItem>::Move(empty);
  trace.clear();
  std::cout << "short_base=" << exercise(array, trace, 0, 0) << "\n";
}

static void copyFallback()
{
  Trace firstTrace, secondTrace;
  NCollection_Array1<CopyEmplaceItem> first(-2, 1);
  NCollection_Array2<CopyEmplaceItem> second(-2, -1, 3, 4);
  initialize(first, firstTrace);
  initialize(second, secondTrace);
  std::cout << "owned_0_4=" << exercise(first, firstTrace, 4, 0) << "\n";
  std::cout << "owned_1_4=" << exercise(second, secondTrace, 4, 0) << "\n";
}

int main()
{
  for (int mode = 0; mode != 4; ++mode) { ownedCases(mode); borrowedCases(mode); }
  invalidCases();
  shortBase();
  copyFallback();
}
