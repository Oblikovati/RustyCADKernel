#include <NCollection_LinearVector.hxx>
#include <cstdlib>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>
#include <vector>

static bool active = false;
static std::string events;
static std::vector<const void*> originalSlots;
static size_t tailAttempts = 0, completedTail = 0, failingAttempt = 0;
static int resources = 0;

struct ResizePayload
{
  int value;
  explicit ResizePayload(int initial) : value(initial) { ++resources; }
  ~ResizePayload() { --resources; }
};

void constructed(char phase, int value)
{
  if (!active) return;
  events += std::string(1, phase) + std::to_string(value) + ",";
  if (phase != 'A' && phase != 'C') return;
  if (++tailAttempts == failingAttempt)
    throw std::runtime_error("tail constructor attempt " + std::to_string(tailAttempts)
                             + ": expected successful construction");
  ++completedTail;
}

void destroyed(const void* pointer, int value)
{
  if (!active) return;
  size_t slot = 0;
  while (slot < originalSlots.size() && originalSlots[slot] != pointer) ++slot;
  const std::string position = slot == originalSlots.size() ? "X" : std::to_string(slot);
  events += "D" + std::to_string(value) + "@" + position + ",";
}

template<bool NoexceptMove> struct ResizeItem
{
  int value;
  std::unique_ptr<ResizePayload> payload;
  explicit ResizeItem(int initial) : value(initial), payload(new ResizePayload(initial)) {}
  ResizeItem() : value(0), payload(new ResizePayload(0)) { constructed('A', value); }
  ResizeItem(const ResizeItem& source)
    : value(source.value), payload(new ResizePayload(source.payload->value))
  { constructed(value == 99 ? 'C' : 'G', value); }
  ResizeItem(ResizeItem&& source) noexcept(NoexceptMove)
    : value(source.value), payload(std::move(source.payload))
  { source.value = -1; constructed('M', value); }
  ~ResizeItem() { destroyed(this, value); }
};

struct ResizeCase { const char* name; size_t capacity, count, size; bool fill, moving; size_t fail; };

template<class Item> bool resize(NCollection_LinearVector<Item>& vector, const Item& donor,
                                 const ResizeCase& test)
{
  try
  {
    if (test.fill) vector.Resize(test.size, donor); else vector.Resize(test.size);
    return true;
  }
  catch (const std::runtime_error&) { return false; }
}

template<class Item> void emit(const char* phase, const ResizeCase& test,
                               const NCollection_LinearVector<Item>& vector,
                               const Item& donor, const Item* before, bool success)
{
  std::cout << phase << ':' << test.name << ':' << (success ? "ok" : "throw") << ':'
            << vector.Size() << ':' << vector.Capacity() << ':' << (before == vector.Data())
            << ':' << events << ':';
  for (const Item& item : vector) std::cout << item.value << '/' << bool(item.payload) << ',';
  std::cout << ':' << donor.value << '/' << bool(donor.payload) << ':' << resources << '\n';
}

template<class Item> void cleanup_tail(NCollection_LinearVector<Item>& vector)
{
  // Native Resize leaves successfully constructed tail values unpublished on constructor failure.
  // This explicit fixture repair is separate from native behavior, before normal vector cleanup.
  for (size_t offset = 0; offset < completedTail; ++offset)
    vector.Data()[vector.Size() + offset].~Item();
}

template<bool Moving> void run_case(const ResizeCase& test)
{
  using Item = ResizeItem<Moving>;
  NCollection_LinearVector<Item> vector(test.capacity);
  for (size_t index = 0; index < test.count; ++index) vector.EmplaceAppend(10 + index);
  Item donor(99);
  const Item* before = vector.Data();
  originalSlots.clear();
  for (size_t index = 0; index < vector.Capacity(); ++index) originalSlots.push_back(before + index);
  events.clear(); tailAttempts = completedTail = 0; failingAttempt = test.fail;
  active = true;
  const bool success = resize(vector, donor, test);
  emit("native", test, vector, donor, before, success);
  if (!success && completedTail > 0)
  {
    cleanup_tail(vector);
    emit("recovered", test, vector, donor, before, success);
  }
  active = false;
}

int main()
{
  const ResizeCase cases[] = {
    {"empty",0,0,0,false,false,0}, {"reserved-empty",4,0,0,true,false,0},
    {"same-default",4,2,2,false,false,0}, {"same-fill",4,2,2,true,false,0},
    {"kept-default",6,2,4,false,false,0}, {"kept-fill",6,2,4,true,false,0},
    {"move-default",2,2,5,false,true,0}, {"move-fill",2,2,5,true,true,0},
    {"copy-default",2,2,3,false,false,0}, {"copy-fill",2,2,3,true,false,0},
    {"empty-default",0,0,3,false,false,0}, {"empty-fill",0,0,1,true,false,0},
    {"shrink-default",6,4,1,false,false,0}, {"shrink-fill",6,4,1,true,false,0},
    {"zero-default",4,2,0,false,false,0}, {"zero-fill",4,2,0,true,false,0},
    {"first-default",2,2,4,false,true,1}, {"first-fill",6,2,4,true,false,1},
    {"late-default",2,2,5,false,true,3}, {"late-fill",6,2,5,true,false,3},
  };
  for (const auto& test : cases)
  {
    test.moving ? run_case<true>(test) : run_case<false>(test);
    if (resources != 0) std::abort();
  }
}
