#include <NCollection_Array1.hxx>
#include <NCollection_Iterator.hxx>
#include <iostream>
using Array = NCollection_Array1<int>;
using ReadCursor = NCollection_Iterator<Array, true>;
using WriteCursor = NCollection_Iterator<Array, false>;

static void defaults() {
  ReadCursor first, second(first);
  WriteCursor writable;
  Array empty;
  const ReadCursor fromEmpty(empty);
  std::cout << "default " << (first == second) << ' ' << (first.ValueIter() == nullptr)
            << ' ' << (first.EndIter() == nullptr) << ' ' << (first == writable)
            << ' ' << (first == fromEmpty) << '\n';
}
static void overlap() {
  int values[] = {10, 20, 30};
  const Array whole(values[0], 1, 3), suffix(values[1], 1, 2), shorter(values[0], 1, 2);
  const ReadCursor first(whole, whole.cbegin() + 1), second(suffix), third(shorter, shorter.cbegin() + 1);
  std::cout << "overlap " << (first == second) << ' ' << (first != third) << ' '
            << (first.ValueIter() - values) << ' ' << (first.EndIter() - values) << '\n';
}
static void assignment() {
  int values[] = {10, 20};
  const Array array(values[0], 1, 2);
  const ReadCursor source(array, array.cbegin() + 1);
  ReadCursor target;
  const bool returned = &(target = source) == &target;
  const bool self = &(target = target) == &target;
  const bool equal = target == source;
  target.Initialize(array);
  std::cout << "assign " << returned << ' ' << self << ' ' << equal << ' '
            << target.Value() << ' ' << (target.ValueIter() == values) << '\n';
}
static void mutableView() {
  int first[] = {10, 20}, second[] = {30, 40, 50};
  Array initial(first[0], 1, 2), replacement(second[0], -4, -2);
  WriteCursor cursor(initial);
  cursor.Initialize(replacement);
  cursor.Next();
  ReadCursor readonly;
  readonly = cursor;
  std::cout << "mutable " << (readonly == cursor) << ' ' << readonly.Value() << ' '
            << (cursor.ChangeValueIter() - second) << ' ' << (cursor.ChangeEndIter() - second) << '\n';
}
int main() { defaults(); overlap(); assignment(); mutableView(); }
