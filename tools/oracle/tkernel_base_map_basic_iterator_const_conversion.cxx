// Both advertised converting overloads are ill-formed at pinned OCCT 3d097a0328e71.
#include <NCollection_BaseMap.hxx>
struct Node : NCollection_ListNode {
  int value = 0;
  Node() : NCollection_ListNode(nullptr) {}
  const int& Key() const { return value; }
  const int& Value() const { return value; }
  int& ChangeValue() { return value; }
};
struct Map : NCollection_BaseMap {
  Map() : NCollection_BaseMap(1, {}) {}
  template<bool Constant> using Cursor = BasicIterator<Map, Node, int, int, false, Constant>;
};
int main() {
  Map::Cursor<false> mutable_iterator;
#ifdef TEST_ASSIGNMENT
  Map::Cursor<true> const_iterator;
  const_iterator = mutable_iterator;
#else
  Map::Cursor<true> const_iterator(mutable_iterator);
#endif
  return const_iterator.More();
}
