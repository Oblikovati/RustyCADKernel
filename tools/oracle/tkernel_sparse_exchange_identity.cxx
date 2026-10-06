// OCCT 3d097a swaps protected fields, retaining observer addresses in each container.
#include <NCollection_SparseArrayBase.hxx>
#include <iostream>
#include <sstream>
#include <string>

static size_t* observedSize = nullptr;
static std::ostringstream destroyed;
static void discardItem(void*) {}
static void copyInteger(void* destination, void* source)
{
  *static_cast<int*>(destination) = *static_cast<int*>(source);
}
static void observeDestruction(void* item)
{
  destroyed << *static_cast<int*>(item) << ':' << *observedSize << ',';
}

struct ExposedSparse : NCollection_SparseArrayBase
{
  ExposedSparse(size_t stride, size_t block, DestroyItemFunc destroy = discardItem)
      : NCollection_SparseArrayBase(stride, block, destroy) {}
  void Insert(size_t index, int value) { setValue(index, &value, copyInteger, copyInteger); }
  size_t* SizeSlot() { return &mySize; }
  void*** TableSlot() { return &myData; }
  size_t Stride() const { return myItemSize; }
  size_t BlockSize() const { return myBlockSize; }
  using NCollection_SparseArrayBase::exchange;
  using NCollection_SparseArrayBase::clearItems;
  using NCollection_SparseArrayBase::getValue;
};

static void savedObservers()
{
  ExposedSparse first(4, 4), second(12, 9);
  first.Insert(0, 10); second.Insert(8, 80); second.Insert(9, 90);
  auto* firstSize = first.SizeSlot(); auto* secondSize = second.SizeSlot();
  auto* firstTable = first.TableSlot(); auto* secondTable = second.TableSlot();
  void* item = second.getValue(8); void** table = *secondTable;
  first.exchange(second);
  std::cout << "identity " << (firstSize == first.SizeSlot()) << ' ' << (secondSize == second.SizeSlot())
            << ' ' << (firstTable == first.TableSlot()) << ' ' << (secondTable == second.TableSlot()) << '\n';
  std::cout << "contents " << *firstSize << ' ' << *secondSize << ' ' << first.Stride() << ' '
            << first.BlockSize() << ' ' << (first.getValue(8) == item) << ' ' << (*firstTable == table) << '\n';
  observedSize = firstSize; first.clearItems(observeDestruction);
  std::cout << "clear_phase " << destroyed.str() << ' ' << *firstSize << ' ' << *secondSize << '\n';
  observedSize = nullptr; destroyed.str("");
}

static void storedDestructorObserver()
{
  auto* first = new ExposedSparse(4, 4);
  auto* second = new ExposedSparse(4, 9, observeDestruction);
  first->Insert(0, 10); second->Insert(8, 80); second->Insert(9, 90);
  observedSize = first->SizeSlot(); first->exchange(*second);
  delete first;
  std::cout << "drop_phase " << destroyed.str() << '\n';
  observedSize = nullptr; destroyed.str(""); delete second;
}

static void repeatedAndSelfExchange()
{
  ExposedSparse first(4, 4), second(4, 9);
  first.Insert(0, 10); second.Insert(8, 80); second.Insert(9, 90);
  auto* firstSize = first.SizeSlot(); auto* secondSize = second.SizeSlot();
  void** firstTable = *first.TableSlot(); void** secondTable = *second.TableSlot();
  first.exchange(second); first.exchange(second);
  std::cout << "round_trip " << *firstSize << ' ' << *secondSize << ' '
            << (*first.TableSlot() == firstTable) << ' ' << (*second.TableSlot() == secondTable) << '\n';
  first.exchange(first);
  std::cout << "self " << (firstSize == first.SizeSlot()) << ' ' << *firstSize << ' '
            << (*first.TableSlot() == firstTable) << ' ' << *static_cast<int*>(first.getValue(0)) << '\n';
}

int main()
{
  savedObservers(); storedDestructorObserver(); repeatedAndSelfExchange();
}
