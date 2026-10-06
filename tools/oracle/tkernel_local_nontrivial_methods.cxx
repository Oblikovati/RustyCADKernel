#include <NCollection_LocalArray.hxx>
#include <iostream>
#include <string>
#include <utility>

static std::string events;

struct MoveObserved
{
  int value = 0;
  MoveObserved() { events += "C,"; }
  MoveObserved(MoveObserved&& source) noexcept : value(source.value)
  {
    events += "M" + std::to_string(value) + ",";
    source.value = -1;
  }
  ~MoveObserved() { events += "D" + std::to_string(value) + ","; }
};

using ObservedArray = NCollection_LocalArray<MoveObserved, 4>;

void fill(ObservedArray& array, int base)
{
  for (size_t index = 0; index < array.Size(); ++index)
    array.begin()[index].value = base + static_cast<int>(index);
}

void emit(const std::string& label, const ObservedArray& array, bool retained)
{
  std::cout << label << ':' << array.Size() << ':' << retained << ':' << events << ':';
  for (const auto& item : array)
    std::cout << item.value << ',';
  std::cout << '\n';
  events.clear();
}

void reallocate_case(const char* label, size_t initial, size_t shrunk, size_t count, bool copy)
{
  ObservedArray array(initial);
  fill(array, 10);
  array.Reallocate(shrunk);
  const auto* before = array.begin();
  events.clear();
  array.Reallocate(count, copy);
  emit(label, array, array.begin() == before);
  array.Deallocate();
  emit(std::string(label) + "-drop", array, false);
}

void construct_case(const char* label, size_t count)
{
  ObservedArray source(count);
  fill(source, 10);
  const auto* before = source.begin();
  events.clear();
  ObservedArray destination(std::move(source));
  emit(label, destination, destination.begin() == before);
  emit(std::string(label) + "-source", source, source.begin() == before);
}

void assign_case(const char* label, size_t initial, size_t count)
{
  ObservedArray destination(initial), source(count);
  fill(destination, 20);
  fill(source, 10);
  const auto* before = source.begin();
  events.clear();
  destination = std::move(source);
  emit(label, destination, destination.begin() == before);
  emit(std::string(label) + "-source", source, source.begin() == before);
}

int main()
{
  reallocate_case("inline-grow", 2, 2, 4, true);
  reallocate_case("inline-grow-no-copy", 2, 2, 4, false);
  reallocate_case("inline-heap", 2, 2, 6, true);
  reallocate_case("heap-heap", 6, 6, 8, true);
  reallocate_case("heap-inline", 6, 1, 3, true);
  reallocate_case("inline-heap-no-copy", 2, 2, 6, false);
  reallocate_case("heap-heap-no-copy", 6, 6, 8, false);
  reallocate_case("heap-inline-no-copy", 6, 1, 3, false);
  reallocate_case("shrink", 6, 6, 2, true);
  reallocate_case("same", 3, 3, 3, false);
  construct_case("construct-inline", 3);
  construct_case("construct-heap", 6);
  assign_case("assign-inline-inline", 2, 3);
  assign_case("assign-heap-inline", 6, 3);
  assign_case("assign-inline-heap", 2, 6);
  assign_case("assign-heap-heap", 6, 7);
  events.clear();
  { ObservedArray array(3); fill(array, 30); events.clear(); }
  std::cout << "destructor:" << events << '\n';
}
