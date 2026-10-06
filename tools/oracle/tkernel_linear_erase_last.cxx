#include <NCollection_LinearVector.hxx>
#include <cstdlib>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>
#include <vector>

struct LastItem;
using LastVector = NCollection_LinearVector<LastItem>;
static LastVector* observed = nullptr;
static std::vector<const LastItem*> slots;
static std::string events;
static bool active = false;
static int failingValue = -99, resources = 0;

struct LastPayload
{
  int value;
  explicit LastPayload(int initial) : value(initial) { ++resources; }
  ~LastPayload()
  {
    if (active) events += "R" + std::to_string(value) + ",";
    --resources;
  }
};

struct LastItem
{
  int value;
  std::unique_ptr<LastPayload> payload;
  explicit LastItem(int initial) : value(initial), payload(new LastPayload(initial)) {}
  LastItem(const LastItem&) = delete;
  LastItem(LastItem&& source) noexcept : value(source.value), payload(std::move(source.payload)) {}
  ~LastItem() noexcept(false);
};

LastItem::~LastItem() noexcept(false)
{
  if (!active) return;
  size_t slot = 0;
  while (slot < slots.size() && slots[slot] != this) ++slot;
  // Native permits this read-only alias; Rust verifies the corresponding catch-time state.
  if (slot == slots.size() || observed->Size() != slot) std::abort();
  events += "D" + std::to_string(value) + "@" + std::to_string(slot) + ",";
  if (value == failingValue) throw std::runtime_error("erase-last destructor failure");
}

struct LastCase { const char* name; size_t capacity, count, calls; int fail; bool append; };

std::string erase_calls(LastVector& vector, size_t calls)
{
  std::string outcomes;
  for (size_t call = 0; call < calls; ++call)
  {
    try { vector.EraseLast(); outcomes += "ok,"; }
    catch (const std::runtime_error&) { outcomes += "throw,"; }
  }
  return outcomes;
}

void emit(const LastCase& test, const LastVector& vector, const LastItem* before,
          const std::string& outcomes)
{
  std::cout << test.name << ':' << outcomes << ':' << vector.Size() << ':' << vector.Capacity()
            << ':' << (before == vector.Data()) << ':' << events << ':';
  for (size_t index = 0; index < vector.Size(); ++index) std::cout << vector(index).value << ',';
  std::cout << ':' << resources << '\n';
}

void run_case(const LastCase& test)
{
  LastVector vector(test.capacity);
  for (size_t index = 0; index < test.count; ++index) vector.EmplaceAppend(10 + index);
  observed = &vector;
  slots.clear();
  for (size_t index = 0; index < vector.Size(); ++index) slots.push_back(vector.Data() + index);
  const LastItem* before = vector.Data();
  failingValue = test.fail;
  events.clear();
  active = true;
  const std::string outcomes = erase_calls(vector, test.calls);
  if (test.append) vector.EmplaceAppend(99);
  emit(test, vector, before, outcomes);
  active = false;
}

int main()
{
  const LastCase cases[] = {
    {"empty",0,0,1,-99,false}, {"reserved-empty",4,0,1,-99,false},
    {"one",4,1,1,-99,false}, {"many",4,3,1,-99,false},
    {"all",4,3,4,-99,false}, {"append",4,3,1,-99,true},
    {"throw-tail",4,3,1,12,false}, {"throw-single",4,1,1,10,false},
    {"throw-then-next",4,3,2,12,false}, {"throw-then-append",4,3,1,12,true},
  };
  for (const auto& test : cases) run_case(test);
  return resources == 0 ? 0 : 1;
}
