#include <NCollection_LinearVector.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>
#include <vector>

struct EraseItem;
static EraseItem* base = nullptr;
static bool active = false;
static int failValue = -99;
static std::string events;

struct EraseItem
{
  int value;
  std::unique_ptr<int> resource;
  explicit EraseItem(int initial) : value(initial), resource(new int(initial)) {}
  EraseItem(const EraseItem&) = delete;
  EraseItem(EraseItem&& source) noexcept
    : value(source.value), resource(std::move(source.resource)) { source.value = -1; }
  EraseItem& operator=(EraseItem&& source);
  ~EraseItem();
};

EraseItem& EraseItem::operator=(EraseItem&& source)
{
  events += "A" + std::to_string(this - base) + "<-" + std::to_string(&source - base)
          + ":" + std::to_string(value) + "<-" + std::to_string(source.value) + ",";
  value = source.value + 100;
  if (source.value == failValue)
    throw std::runtime_error("injected move assignment");
  value = source.value;
  source.value = -1;
  resource = std::move(source.resource);
  return *this;
}

EraseItem::~EraseItem()
{
  if (active)
    events += "D" + std::to_string(this - base) + ":" + std::to_string(value) + ",";
}

using EraseVector = NCollection_LinearVector<EraseItem>;

std::string apply_erase(EraseVector& vector, size_t from, size_t to, bool single)
{
  try
  {
    if (single) vector.Erase(from); else vector.Erase(from, to);
    return "ok";
  }
  catch (const Standard_OutOfRange&) { return "range"; }
  catch (const std::runtime_error&) { return "throw"; }
}

void print_values(const EraseVector& vector, const std::vector<const int*>& resources)
{
  for (const auto& item : vector) std::cout << item.value << ',';
  std::cout << ':';
  for (const auto& item : vector)
  {
    int index = -1;
    for (size_t i = 0; i < resources.size(); ++i)
      if (item.resource.get() == resources[i]) index = static_cast<int>(i);
    std::cout << index << ',';
  }
  std::cout << '\n';
}

void erase_case(const char* name, size_t count, size_t from, size_t to, bool single, int fail)
{
  EraseVector vector(8);
  std::vector<const int*> resources;
  for (size_t i = 0; i < count; ++i)
    resources.push_back(vector.EmplaceAppend(static_cast<int>(10 + i)).resource.get());
  base = vector.Data();
  failValue = fail;
  events.clear();
  active = true;
  const std::string outcome = apply_erase(vector, from, to, single);
  active = false;
  std::cout << name << ':' << outcome << ':' << vector.Size() << ':' << vector.Capacity()
            << ':' << (base == vector.Data()) << ':' << events << ':';
  print_values(vector, resources);
}

void successful_cases()
{
  erase_case("single-head", 5, 0, 0, true, -99);
  erase_case("single-middle", 5, 2, 0, true, -99);
  erase_case("single-tail", 5, 4, 0, true, -99);
  erase_case("range-overlap", 5, 0, 2, false, -99);
  erase_case("range-disjoint", 5, 1, 4, false, -99);
  erase_case("range-tail", 5, 2, 5, false, -99);
  erase_case("range-all", 5, 0, 5, false, -99);
}

void failure_cases()
{
  erase_case("empty-single", 0, 0, 0, true, -99);
  erase_case("empty-range", 0, 0, 1, false, -99);
  erase_case("past-end", 5, 5, 0, true, -99);
  erase_case("empty-interval", 5, 2, 2, false, -99);
  erase_case("reversed", 5, 4, 1, false, -99);
  erase_case("range-past-end", 5, 1, 6, false, -99);
  erase_case("throw-first", 5, 1, 0, true, 12);
  erase_case("throw-after-one", 5, 1, 0, true, 13);
  erase_case("throw-range", 5, 0, 2, false, 13);
}

int main()
{
  successful_cases();
  failure_cases();
}
