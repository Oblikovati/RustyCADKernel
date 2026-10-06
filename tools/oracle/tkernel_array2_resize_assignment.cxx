#include <NCollection_Array2.hxx>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <vector>

struct ResizeItem
{
  inline static std::vector<std::string> events;
  inline static int failValue = 0;
  int value = 0;
  ResizeItem() { events.push_back("D"); }
  explicit ResizeItem(int supplied) : value(supplied) {}
  ResizeItem(const ResizeItem& donor) : value(donor.value) { events.push_back("C" + std::to_string(value)); }
  ~ResizeItem() { events.push_back("X" + std::to_string(value)); }
  ResizeItem& operator=(ResizeItem&& donor)
  {
    events.push_back("A" + std::to_string(value) + ":" + std::to_string(donor.value));
    if (donor.value == failValue) throw std::runtime_error("requested assignment failure");
    value = donor.value;
    donor.value = -donor.value;
    return *this;
  }
};

static std::string trace()
{
  std::string output;
  for (const std::string& event : ResizeItem::events) output += (output.empty() ? "" : ",") + event;
  return output;
}

template<class Item>
static std::string values(const Item* pointer, size_t size)
{
  std::string output;
  for (size_t index = 0; index < size; ++index) output += (index == 0 ? "" : ",") + std::to_string(pointer[index].value);
  return output;
}

template<class Item>
static std::string state(const NCollection_Array2<Item>& array)
{
  std::ostringstream output;
  output << array.NbRows() << "," << array.NbColumns() << ";" << array.LowerRow() << "," << array.LowerCol()
         << ";" << array.Lower() << "," << array.Upper() << ";" << array.IsDeletable()
         << ";" << values(array.Data(), array.Size());
  return output.str();
}

static void resizeCopy(NCollection_Array2<ResizeItem>& array, bool trim)
{
  if (trim) array.ResizeWithTrim(7, 8, -3, -2, true);
  else array.Resize(7, 8, -3, -2, true);
}

static void ownedCase(const char* label, bool trim, int failure)
{
  NCollection_Array2<ResizeItem> array(-2, -1, 3, 5);
  for (size_t i = 0; i < 6; ++i) array.ChangeAt(i).value = int(i) + 1;
  ResizeItem::events.clear();
  ResizeItem::failValue = failure;
  bool caught = false;
  try { resizeCopy(array, trim); }
  catch (const std::runtime_error&) { caught = true; }
  std::cout << label << "=" << state(array) << ";" << caught << ";" << trace() << "\n";
}

static void borrowedCase(const char* label, bool trim, int failure)
{
  ResizeItem buffer[] = {ResizeItem(1), ResizeItem(2), ResizeItem(3), ResizeItem(4), ResizeItem(5), ResizeItem(6), ResizeItem(7)};
  NCollection_Array2<ResizeItem> array(buffer[0], -2, -1, 3, 5);
  ResizeItem::events.clear();
  ResizeItem::failValue = failure;
  bool caught = false;
  try { resizeCopy(array, trim); }
  catch (const std::runtime_error&) { caught = true; }
  std::cout << label << "=" << state(array) << ";" << caught << ";" << trace() << ";" << values(buffer, 7) << "\n";
}

static void noCopyCase()
{
  NCollection_Array2<ResizeItem> array(-2, -1, 3, 5);
  for (size_t i = 0; i < 6; ++i) array.ChangeAt(i).value = int(i) + 1;
  ResizeItem::events.clear();
  array.Resize(7, 9, -3, -2, false);
  std::cout << "same_area_no_copy=" << state(array) << ";" << trace() << "\n";
}

static void sameDimensionsCase()
{
  NCollection_Array2<ResizeItem> array(-2, -1, 3, 5);
  for (size_t i = 0; i < 6; ++i) array.ChangeAt(i).value = int(i) + 1;
  ResizeItem::events.clear();
  array.ResizeWithTrim(7, 8, -3, -1, true);
  std::cout << "same_dimensions=" << state(array) << ";" << trace() << "\n";
}

static void zeroCountCase()
{
  NCollection_Array2<ResizeItem> array(-2, -1, 3, 5);
  for (size_t i = 0; i < 6; ++i) array.ChangeAt(i).value = int(i) + 1;
  ResizeItem::events.clear();
  array.Resize(size_t(0), size_t(4), true);
  std::cout << "zero_count=" << state(array) << ";" << trace() << "\n";
}

static void emptySourceCase()
{
  NCollection_Array2<ResizeItem> array(size_t(0), size_t(3));
  ResizeItem::events.clear();
  array.ResizeWithTrim(7, 8, -3, -2, true);
  std::cout << "empty_source=" << state(array) << ";" << trace() << "\n";
}

static void retainedFlatLength()
{
  NCollection_Array2<ResizeItem> array(-2, -1, 3, 5);
  array.NCollection_Array1<ResizeItem>::Resize(size_t(7), false);
  for (size_t i = 0; i < 7; ++i) array.ChangeAt(i).value = int(i) + 1;
  ResizeItem::events.clear();
  array.Resize(7, 8, -3, -1, true);
  std::cout << "retained_flat_length=" << state(array) << ";" << trace() << "\n";
}

struct CopyFallback : ResizeItem
{
  CopyFallback() = default;
  CopyFallback(const CopyFallback& donor) : ResizeItem(donor) {}
  CopyFallback& operator=(const CopyFallback& donor)
  {
    events.push_back("Q" + std::to_string(value) + ":" + std::to_string(donor.value));
    if (donor.value == failValue) throw std::runtime_error("requested copy assignment failure");
    value = donor.value;
    return *this;
  }
};

static void copyFallbackFailure()
{
  NCollection_Array2<CopyFallback> array(-2, -1, 3, 5);
  for (size_t i = 0; i < 6; ++i) array.ChangeAt(i).value = int(i) + 1;
  ResizeItem::events.clear();
  ResizeItem::failValue = 3;
  bool caught = false;
  try { array.Resize(7, 8, -3, -2, true); }
  catch (const std::runtime_error&) { caught = true; }
  std::cout << "copy_fallback_failure=" << state(array) << ";" << caught << ";" << trace() << "\n";
}

static void countCase(bool trim)
{
  NCollection_Array2<ResizeItem> array(-2, -1, 3, 5);
  for (size_t i = 0; i < 6; ++i) array.ChangeAt(i).value = int(i) + 1;
  ResizeItem::events.clear();
  ResizeItem::failValue = 0;
  if (trim) array.ResizeWithTrim(size_t(2), size_t(2), true);
  else array.Resize(size_t(2), size_t(2), true);
  std::cout << "count_resize_" << trim << "=" << state(array) << ";" << trace() << "\n";
}

struct DefaultFailure
{
  int value;
  DefaultFailure() { throw std::runtime_error("requested default failure"); }
  explicit DefaultFailure(int supplied) : value(supplied) {}
};

static void defaultFailure()
{
  DefaultFailure buffer[] = {DefaultFailure(1), DefaultFailure(2)};
  NCollection_Array2<DefaultFailure> array(buffer, size_t(1), size_t(2));
  bool caught = false;
  try { array.Resize(3, 4, 5, 6, true); }
  catch (const std::runtime_error&) { caught = true; }
  std::cout << "default_failure=" << state(array) << ";" << caught << ";" << values(buffer, 2) << "\n";
}

static void copyFallbackBorrowed()
{
  CopyFallback buffer[6];
  for (size_t i = 0; i < 6; ++i) buffer[i].value = int(i) + 1;
  NCollection_Array2<CopyFallback> array(buffer[0], -2, -1, 3, 5);
  ResizeItem::events.clear();
  ResizeItem::failValue = 0;
  array.ResizeWithTrim(7, 8, -3, -2, true);
  std::cout << "shared_copy_fallback=" << state(array) << ";" << trace() << ";" << values(buffer, 6) << "\n";
}

int main()
{
  ownedCase("owned_packed", false, 0);
  ownedCase("owned_trim", true, 0);
  ownedCase("owned_assignment_failure", false, 3);
  borrowedCase("borrowed_packed", false, 0);
  borrowedCase("borrowed_trim", true, 0);
  borrowedCase("borrowed_assignment_failure", true, 4);
  noCopyCase();
  sameDimensionsCase();
  zeroCountCase();
  emptySourceCase();
  retainedFlatLength();
  copyFallbackFailure();
  countCase(false);
  countCase(true);
  defaultFailure();
  copyFallbackBorrowed();
}
