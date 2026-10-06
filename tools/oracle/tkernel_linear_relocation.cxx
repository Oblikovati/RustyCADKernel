#include <NCollection_LinearVector.hxx>
#include <NCollection_LocalArray.hxx>
#include <iostream>
#include <limits>
#include <string>
#include <utility>

static std::string events;

template<bool NoexceptMove> struct Relocated
{
  int value;
  explicit Relocated(int initial) : value(initial) {}
  Relocated(const Relocated& source) : value(source.value)
  {
    events += "C" + std::to_string(value) + ",";
  }
  Relocated(Relocated&& source) noexcept(NoexceptMove) : value(source.value)
  {
    events += "M" + std::to_string(value) + ",";
    source.value = -1;
  }
  ~Relocated() { events += "D" + std::to_string(value) + ","; }
};

struct MoveOnly : Relocated<false>
{
  using Relocated<false>::Relocated;
  MoveOnly(const MoveOnly&) = delete;
  MoveOnly(MoveOnly&&) = default;
};

template<class Item> void reserve_case(const char* label, size_t capacity, size_t count, size_t minimum)
{
  NCollection_LinearVector<Item> vector(capacity);
  for (size_t i = 0; i < count; ++i)
    vector.EmplaceAppend(static_cast<int>(10 + i));
  auto* before = vector.Data();
  events.clear();
  vector.Reserve(minimum);
  std::cout << label << ':' << vector.Size() << ':' << vector.Capacity() << ':'
            << (before == vector.Data()) << ':' << events << ':';
  for (const auto& item : vector)
    std::cout << item.value << ',';
  std::cout << '\n';
}

void capacity_case(size_t current, size_t minimum, size_t maximum)
{
  NCollection_LinearVector<int> vector;
  vector.myCapacity = current;
  std::cout << "capacity:" << current << ':' << minimum << ':' << maximum << ':'
            << vector.nextCapacity(minimum, maximum) << '\n';
}

void local_storage()
{
  NCollection_LocalArray<int, 4> array;
  std::cout << "local:" << array.isInline();
  array.Allocate(3);
  std::cout << ':' << array.isInline();
  array.Reallocate(6, false);
  std::cout << ':' << array.isInline();
  array.Reallocate(0);
  std::cout << ':' << array.isInline();
  array.Reallocate(2, false);
  std::cout << ':' << array.isInline();
  array.Reallocate(7, false);
  array.Deallocate();
  std::cout << ':' << array.isInline() << '\n';
}

void relocation_cases()
{
  reserve_case<Relocated<true>>("empty-noop", 0, 0, 0);
  reserve_case<Relocated<true>>("empty-grow", 0, 0, 1);
  reserve_case<Relocated<true>>("retained-noop", 4, 3, 4);
  reserve_case<Relocated<true>>("move-double", 4, 3, 5);
  reserve_case<Relocated<true>>("move-minimum", 4, 3, 20);
  reserve_case<Relocated<true>>("move-full", 6, 6, 7);
  reserve_case<Relocated<false>>("copy-double", 4, 3, 5);
  reserve_case<Relocated<false>>("copy-minimum", 4, 3, 20);
  reserve_case<MoveOnly>("move-only", 4, 3, 5);
}

void capacity_cases()
{
  const size_t largest = std::numeric_limits<size_t>::max();
  capacity_case(0, 0, largest);
  capacity_case(0, 0, 1);
  capacity_case(0, 0, 0);
  capacity_case(4, 0, 7);
  capacity_case(4, 9, 7);
  capacity_case(largest / 2, 0, largest);
  capacity_case(largest / 2 + 1, 0, largest);
  capacity_case(largest, largest, largest);
}

int main()
{
  relocation_cases();
  capacity_cases();
  local_storage();
}
