#include <NCollection_DataMap.hxx>
#include <NCollection_FlatDataMap.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <NCollection_IndexedMap.hxx>
#include <NCollection_OrderedDataMap.hxx>
#include <iostream>
#include <utility>

static void indexedCursor() {
  NCollection_IndexedDataMap<int, int> map;
  map.Add(10, 100); map.Add(20, 200);
  const auto view = std::as_const(map).IndexedItems();
  auto cursor = view.begin();
  const auto previous = cursor++;
  std::cout << "indexed " << (*previous).Key << ' ' << (*previous).Value << ' '
            << (*previous).Index << ' ' << (*cursor).Key << ' ' << (cursor != previous)
            << ' ' << (&(cursor = previous) == &cursor) << ' ' << (&(cursor = cursor) == &cursor)
            << ' ' << (&++cursor == &cursor);
  ++cursor;
  std::cout << ' ' << (cursor == view.end()) << '\n';
}

static void indexedEnds() {
  NCollection_IndexedMap<int> first, second;
  first.Add(10); second.Add(10);
  const auto left = std::as_const(first).IndexedItems(), right = std::as_const(second).IndexedItems();
  auto cursor = left.cbegin();
  std::cout << "ends " << (cursor != right.begin()) << ' ' << (left.cend() == right.end());
  ++cursor;
  std::cout << ' ' << (cursor == right.cend()) << ' ' << (cursor == decltype(cursor)()) << '\n';
}

template<class Map> static void pairCursor(const char* label) {
  Map map;
  map.Bind(10, 100);
  const auto view = std::as_const(map).Items();
  const auto copied = view;
  auto cursor = copied.cbegin();
  const auto previous = cursor++;
  std::cout << label << ' ' << (view.begin() == copied.cbegin()) << ' '
            << (*previous).Key << ' ' << (*previous).Value << ' '
            << (cursor == copied.cend()) << '\n';
}

static void mutableConstView() {
  NCollection_IndexedDataMap<int, int> map;
  map.Add(10, 100);
  const auto view = map.IndexedItems();
  (*view.cbegin()).Value += 7;
  std::cout << "mutable " << map.FindFromIndex(1) << ' ' << (view.end() == view.cend()) << '\n';
}

int main() {
  indexedCursor(); indexedEnds();
  pairCursor<NCollection_DataMap<int, int>>("data");
  pairCursor<NCollection_FlatDataMap<int, int>>("flat");
  pairCursor<NCollection_OrderedDataMap<int, int>>("ordered");
  mutableConstView();
}
