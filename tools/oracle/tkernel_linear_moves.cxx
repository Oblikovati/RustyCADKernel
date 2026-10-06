#include <NCollection_LinearVector.hxx>
#include <cassert>
#include <iostream>
#include <memory>
#include <string>
#include <utility>

static std::string events;
struct MoveItem;
using MoveVector = NCollection_LinearVector<MoveItem>;
static const MoveVector* observedSource = nullptr;
static const MoveItem* sourcePointer = nullptr;
static size_t sourceSize = 0, sourceCapacity = 0;

struct MoveItem
{
  std::unique_ptr<int> value;
  explicit MoveItem(int initial) : value(new int(initial)) {}
  MoveItem(const MoveItem&) = delete;
  MoveItem(MoveItem&& source) noexcept : value(std::move(source.value)) { events += "M,"; }
  ~MoveItem();
};

MoveItem::~MoveItem()
{
  if (observedSource != nullptr)
  {
    assert(observedSource->Size() == sourceSize);
    assert(observedSource->Capacity() == sourceCapacity);
    assert(observedSource->Data() == sourcePointer);
  }
  events += "D" + std::to_string(value ? *value : -1) + ",";
}

MoveVector filled(size_t capacity, size_t count, int base)
{
  MoveVector vector(capacity);
  for (size_t i = 0; i < count; ++i)
    vector.EmplaceAppend(base + static_cast<int>(i));
  return vector;
}

void row(const char* name, const MoveVector& destination, const MoveVector& source,
         const MoveItem* before, const int* payload)
{
  const bool payloadStable = destination.IsEmpty() || destination.First().value.get() == payload;
  std::cout << name << ':' << destination.Size() << ':' << destination.Capacity() << ':'
            << (destination.Data() == before) << ':' << payloadStable << ':'
            << source.Size() << ':' << source.Capacity() << ':' << (source.Data() == nullptr)
            << ':' << events << ':';
  for (const auto& item : destination)
    std::cout << *item.value << ',';
  std::cout << '\n';
}

void construct_case(const char* name, size_t capacity, size_t count, bool retain)
{
  MoveVector source = filled(capacity, count, 10);
  if (retain)
    source.Clear(false);
  const MoveItem* before = source.Data();
  const int* payload = source.IsEmpty() ? nullptr : source.First().value.get();
  events.clear();
  MoveVector destination(std::move(source));
  row(name, destination, source, before, payload);
  source.EmplaceAppend(99);
  assert(source.Size() == 1 && *source.First().value == 99);
}

void assign_case(const char* name, size_t capacity, size_t count)
{
  MoveVector source = filled(capacity, count, 10), destination = filled(3, 2, 20);
  sourcePointer = source.Data();
  sourceSize = source.Size();
  sourceCapacity = source.Capacity();
  const int* payload = source.IsEmpty() ? nullptr : source.First().value.get();
  events.clear();
  observedSource = &source;
  assert(&(destination = std::move(source)) == &destination);
  observedSource = nullptr;
  row(name, destination, source, sourcePointer, payload);
}

void self_case()
{
  MoveVector source = filled(4, 3, 10);
  auto* pointer = source.Data();
  events.clear();
  assert(&(source = std::move(source)) == &source);
  std::cout << "self:" << source.Size() << ':' << source.Capacity() << ':'
            << (source.Data() == pointer) << ':' << events << '\n';
}

int main()
{
  construct_case("empty", 0, 0, false);
  construct_case("reserved", 7, 0, false);
  construct_case("single", 1, 1, false);
  construct_case("multiple", 7, 3, false);
  construct_case("retained", 7, 3, true);
  assign_case("assign-empty", 0, 0);
  assign_case("assign-reserved", 7, 0);
  assign_case("assign-values", 7, 3);
  self_case();
}
