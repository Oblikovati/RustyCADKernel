#include <NCollection_LinearVector.hxx>
#include <cstdlib>
#include <functional>
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
static bool failingAssignment = false;
static size_t expectedSize = 0;
static std::function<size_t()> observedSize;

struct SetPayload
{
  int value;
  explicit SetPayload(int initial) : value(initial) { ++resources; }
  ~SetPayload() { --resources; }
};

void constructed(char phase, int value)
{
  if (!active) return;
  events += std::string(1, phase) + std::to_string(value) + ",";
  if (phase != 'A') return;
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

void assigned(char phase, int before, int after)
{
  if (observedSize() != expectedSize) std::abort();
  events += std::string(1, phase) + std::to_string(before) + ">" + std::to_string(after) + ",";
  if (failingAssignment)
    throw std::runtime_error("assignment value " + std::to_string(after) + ": expected success");
}

template<bool NoexceptMove> struct SetItem
{
  int value;
  std::unique_ptr<SetPayload> payload;
  explicit SetItem(int initial) : value(initial), payload(new SetPayload(initial)) {}
  SetItem() : value(0), payload(new SetPayload(0)) { constructed('A', value); }
  SetItem(const SetItem& source)
    : value(source.value), payload(new SetPayload(source.payload->value))
  { constructed('G', value); }
  SetItem(SetItem&& source) noexcept(NoexceptMove)
    : value(source.value), payload(std::move(source.payload))
  { source.value = -1; constructed('M', value); }
  SetItem& operator=(const SetItem& source)
  {
    const int before = value;
    value = source.value;
    payload.reset(new SetPayload(source.payload->value));
    assigned('C', before, value);
    return *this;
  }
  SetItem& operator=(SetItem&& source)
  {
    const int before = value;
    value = source.value;
    source.value = -1;
    payload = std::move(source.payload);
    assigned('M', before, value);
    return *this;
  }
  ~SetItem() { destroyed(this, value); }
};

struct SetCase {
  const char* name; size_t capacity, count, index;
  bool move, relocating; size_t fail; bool assignFail;
};

template<class Item> Item* set_value(NCollection_LinearVector<Item>& vector, Item& donor,
                                    const SetCase& test)
{
  try
  {
    if (test.move) return &vector.SetValue(test.index, std::move(donor));
    return &vector.SetValue(test.index, donor);
  }
  catch (const std::runtime_error&) { return nullptr; }
}

template<class Item> void emit(const char* phase, const SetCase& test,
                               const NCollection_LinearVector<Item>& vector,
                               const Item& donor, const Item* before,
                               const SetPayload* payload, Item* result)
{
  const bool samePayload = test.index < vector.Size() && vector(test.index).payload.get() == payload;
  std::cout << phase << ':' << test.name << ':' << (result ? "ok" : "throw") << ':'
            << vector.Size() << ':' << vector.Capacity() << ':' << (before == vector.Data())
            << ':' << events << ':';
  for (const Item& item : vector) std::cout << item.value << '/' << bool(item.payload) << ',';
  std::cout << ':' << donor.value << '/' << bool(donor.payload) << ':' << resources << ':'
            << (result && result == vector.Data() + test.index) << ':' << samePayload << '\n';
}

template<class Item> void cleanup_tail(NCollection_LinearVector<Item>& vector)
{
  // Native Resize leaves successfully constructed tail values unpublished on constructor failure.
  // This explicit fixture repair is separate from native behavior, before normal vector cleanup.
  for (size_t offset = 0; offset < completedTail; ++offset)
    vector.Data()[vector.Size() + offset].~Item();
}

template<class Item> void prepare(const NCollection_LinearVector<Item>& vector, const SetCase& test)
{
  originalSlots.clear();
  for (size_t index = 0; index < vector.Capacity(); ++index) originalSlots.push_back(vector.Data() + index);
  events.clear(); tailAttempts = completedTail = 0; failingAttempt = test.fail;
  failingAssignment = test.assignFail;
  expectedSize = test.index >= test.count ? test.index + 1 : test.count;
  observedSize = [&vector] { return vector.Size(); };
}

template<bool Moving> void run_case(const SetCase& test)
{
  using Item = SetItem<Moving>;
  NCollection_LinearVector<Item> vector(test.capacity);
  for (size_t index = 0; index < test.count; ++index) vector.EmplaceAppend(10 + index);
  Item donor(99);
  const Item* before = vector.Data();
  const SetPayload* payload = donor.payload.get();
  prepare(vector, test);
  active = true;
  Item* result = set_value(vector, donor, test);
  emit("native", test, vector, donor, before, payload, result);
  if (!result && vector.Size() == test.count && completedTail > 0)
  {
    cleanup_tail(vector);
    emit("recovered", test, vector, donor, before, payload, result);
  }
  active = false;
}

static const SetCase cases[] = {
  {"copy-existing",4,2,0,false,false,0,false}, {"move-existing",4,2,1,true,false,0,false},
  {"copy-grow",2,2,4,false,true,0,false}, {"move-grow",2,2,4,true,true,0,false},
  {"copy-fallback",2,2,2,false,false,0,false}, {"move-fallback",2,2,2,true,false,0,false},
  {"copy-kept",6,2,4,false,false,0,false}, {"move-kept",6,2,4,true,false,0,false},
  {"empty-copy",0,0,0,false,false,0,false}, {"empty-move",0,0,2,true,false,0,false},
  {"copy-fail-existing",4,2,1,false,false,0,true}, {"move-fail-existing",4,2,0,true,false,0,true},
  {"copy-fail-grow",2,2,3,false,true,0,true}, {"move-fail-grow",2,2,3,true,false,0,true},
  {"copy-fail-kept",6,2,4,false,false,0,true}, {"move-fail-kept",6,2,4,true,false,0,true},
  {"first-copy",2,2,4,false,true,1,false}, {"first-move",6,2,4,true,false,1,false},
  {"late-copy",2,2,4,false,true,3,false}, {"late-move",6,2,4,true,false,3,false},
};

int main()
{
  for (const auto& test : cases)
  {
    test.relocating ? run_case<true>(test) : run_case<false>(test);
    if (resources != 0) std::abort();
  }
}
