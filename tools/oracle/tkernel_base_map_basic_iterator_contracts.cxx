#include <NCollection_BaseMap.hxx>
#include <iostream>
#include <type_traits>

struct ProbeNode : NCollection_ListNode {
  int key, value;
  ProbeNode* ordered = nullptr;
  static int reads, writes, keys, buckets, orders;
  ProbeNode(int k, int v, ProbeNode* next = nullptr) : NCollection_ListNode(next), key(k), value(v) {}
  const int& Key() const { ++keys; return key; }
  const int& Value() const { ++reads; return value; }
  int& ChangeValue() { ++writes; return value; }
  NCollection_ListNode* Next() const { ++buckets; return NCollection_ListNode::Next(); }
  ProbeNode* OrderNext() const { ++orders; return ordered; }
  static void reset() { reads = writes = keys = buckets = orders = 0; }
};
int ProbeNode::reads = 0, ProbeNode::writes = 0, ProbeNode::keys = 0, ProbeNode::buckets = 0, ProbeNode::orders = 0;
class ProbeMap : public NCollection_BaseMap {
public:
  ProbeNode* first = nullptr;
  ProbeMap() : NCollection_BaseMap(1, {}) {
    size_t count = 0; NCollection_ListNode** table = nullptr;
    beginResize(0, count, table); endResize(0, count, table);
  }
  ProbeNode* FirstNode() const { return first; }
  template<bool C, bool K = false, bool O = false>
  using Cursor = BasicIterator<ProbeMap, ProbeNode, int, int, K, C, O>;
};
int main() {
  ProbeNode a(1, 101), b(2, 202), c(3, 303);
  a.NCollection_ListNode::Next() = &b;
  c.ordered = &a; a.ordered = &b;
  ProbeMap map, other;
  map.Buckets()[2] = &a; map.Buckets()[9] = &c; map.first = &c;
  using Mutable = ProbeMap::Cursor<false>;
  using Const = ProbeMap::Cursor<true>;
  Mutable empty;
  Const empty2(other);
  empty.Next();
  std::cout << "default_end " << empty.More() << ' ' << (empty.Node() == nullptr) << ' ' << (empty == empty2) << '\n';
  Mutable cursor(map);
  ProbeNode::reset();
  const int value = *cursor;
  int* mutable_arrow = cursor.operator->();
  const int key = cursor.Key();
  std::cout << "mutable_access " << value << ' ' << key << ' ' << (mutable_arrow == &a.value) << ' '
            << ProbeNode::reads << ' ' << ProbeNode::writes << ' ' << ProbeNode::keys << '\n';
  *cursor = 111;
  Const converted(map), assigned;
  std::cout << "const_container " << (converted == cursor) << ' ' << (converted.Node() == &a) << ' ' << *converted << '\n';
  std::cout << "const_assignment " << (&(assigned = converted) == &assigned) << ' ' << (assigned == cursor) << '\n';
  ProbeNode::reset();
  const int cv = *converted;
  const int* ca = converted.operator->();
  std::cout << "const_access " << cv << ' ' << (ca == &a.value) << ' ' << ProbeNode::reads << ' ' << ProbeNode::writes << '\n';
  auto old = cursor++;
  std::cout << "postfix " << old.Key() << ' ' << cursor.Key() << ' ' << (old.Node() == &a) << '\n';
  const bool same = &(++cursor) == &cursor;
  std::cout << "prefix " << same << ' ' << cursor.Key() << ' ' << (cursor.Node() == &c) << '\n';
  cursor.Next(); cursor.Next();
  std::cout << "all_ends " << cursor.More() << ' ' << (cursor == empty) << ' ' << (cursor == empty2) << '\n';
  Const node_cursor(&a);
  std::cout << "node_chain ";
  while (node_cursor.More()) { std::cout << node_cursor.Key() << ','; node_cursor.Next(); }
  std::cout << node_cursor.More() << '\n';
  ProbeMap::Cursor<false, true> keyonly(map);
  ProbeNode::reset();
  const int kv = *keyonly, kk = keyonly.Key();
  const int* kp = keyonly.operator->();
  std::cout << "keyonly " << kv << ' ' << kk << ' ' << (kp == &a.value) << ' ' << ProbeNode::reads << ' ' << ProbeNode::writes << ' ' << ProbeNode::keys << '\n';
  ProbeMap::Cursor<true, false, true> ordered(map);
  ProbeNode::reset();
  std::cout << "ordered ";
  while (ordered.More()) { std::cout << ordered.Key() << ','; ordered.Next(); }
  std::cout << ProbeNode::buckets << ' ' << ProbeNode::orders << '\n';
  ProbeMap::Cursor<true, true, true> ordered_key(map);
  ProbeNode::reset();
  const int ok = ordered_key.Key(), ov = *ordered_key;
  std::cout << "ordered_keyonly " << ok << ' ' << ov << ' ' << ProbeNode::reads << ' ' << ProbeNode::keys << '\n';
  Mutable copied(old), target;
  std::cout << "copy_assignment " << (&(target = copied) == &target) << ' ' << (target == old) << ' ' << (&(target = target) == &target) << '\n';
  other.Buckets()[40] = &a;
  Const same_node(other);
  std::cout << "shared_node_equality " << (same_node == old) << ' ' << (same_node != empty) << '\n';
  return 0;
}
