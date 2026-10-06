#include <NCollection_IndexedMap.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <iostream>
using KeyMap = NCollection_IndexedMap<int>;
using KeyCursor = NCollection_BaseMap::IndexedIterator<KeyMap, int, true>;
using ValueMap = NCollection_IndexedDataMap<int, int>;
using ValueCursor = NCollection_BaseMap::IndexedIterator<ValueMap, int, false>;

static KeyMap keys() {
  KeyMap map;
  for (int key : {10, 20, 30}) map.Add(key);
  return map;
}
static void states() {
  auto map = keys();
  KeyCursor empty, zero(map, 0), past(map, 4), cursor(map, 2);
  std::cout << "states " << empty.Index() << ' ' << empty.More() << ' '
            << zero.More() << ' ' << (empty == past) << '\n';
  std::cout << "read " << *cursor << ' ' << cursor.Key() << ' '
            << (cursor.operator->() == &map.FindKey(2)) << ' '
            << cursor[-1] << ' ' << cursor[1] << ' ' << cursor.Index() << '\n';
}
static void increments() {
  auto map = keys();
  KeyCursor cursor(map, 1);
  const bool prefix = &++cursor == &cursor;
  const auto oldPlus = cursor++;
  const bool retreat = &--cursor == &cursor;
  const auto oldMinus = cursor--;
  std::cout << "steps " << prefix << ' ' << oldPlus.Index() << ' ' << retreat
            << ' ' << oldMinus.Index() << ' ' << cursor.Index() << ' '
            << (&cursor.Iterator() == &cursor) << ' ' << (&cursor.ChangeIterator() == &cursor) << '\n';
  KeyCursor zero(map, 0);
  const auto oldZero = zero--;
  const bool wrapped = zero.Index() == size_t(-1);
  ++zero;
  std::cout << "wrap " << oldZero.Index() << ' ' << wrapped << ' ' << zero.Index() << '\n';
}
static void arithmetic() {
  auto map = keys();
  KeyCursor cursor(map, 1), first(cursor);
  const auto plus = (cursor += 2).Index();
  const auto minus = (cursor -= 1).Index();
  std::cout << "offset " << plus << ' ' << minus << ' ' << (cursor + 1).Index()
            << ' ' << (1 + cursor).Index() << ' ' << (cursor - 1).Index() << ' '
            << (cursor - first) << ' ' << (first < cursor) << ' ' << (cursor > first)
            << ' ' << (cursor <= cursor) << ' ' << (cursor >= cursor) << '\n';
}
static void equalityAndSize() {
  auto first = keys(), second = keys();
  KeyCursor left(first, 4), right(second, 5), live(first, 1), other(second, 1);
  std::cout << "equality " << (left == right) << ' ' << (left < right) << ' '
            << (live == other) << ' ' << (live - other) << '\n';
  KeyMap changing;
  changing.Add(10);
  KeyCursor cursor(changing, 2);
  const bool before = cursor.More();
  changing.Add(20);
  const bool after = cursor.More();
  const int value = *cursor;
  changing.Clear(false);
  std::cout << "live " << before << ' ' << after << ' ' << value << ' ' << cursor.More() << '\n';
}
static void mutableConversion() {
  ValueMap map;
  map.Add(10, 100); map.Add(20, 200);
  ValueCursor cursor(map, 1);
  *cursor = 101; cursor[1] = 202;
  NCollection_BaseMap::IndexedIterator<ValueMap, int, true> readonly;
  readonly = cursor;
  std::cout << "mutable " << (readonly == cursor) << ' ' << readonly.Key() << ' '
            << *readonly << ' ' << readonly[1] << ' '
            << (cursor.operator->() == &map.ChangeFromIndex(1)) << '\n';
  ValueCursor copied;
  const bool assigned = &(copied = cursor) == &copied;
  const bool self = &(copied = copied) == &copied;
  std::cout << "assign " << assigned << ' ' << self << ' ' << copied.Index() << '\n';
}
int main() {
  states(); increments(); arithmetic(); equalityAndSize(); mutableConversion();
}
