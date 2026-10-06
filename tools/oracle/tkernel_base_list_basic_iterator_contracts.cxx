#include <NCollection_BaseList.hxx>
#include <iostream>
struct Prefix { long long padding[4] = {}; };
struct Node : Prefix, NCollection_ListNode {
  int item;
  mutable int reads = 0;
  int writes = 0;
  explicit Node(int value) : NCollection_ListNode(nullptr), item(value) {}
  const int& Value() const { ++reads; return item; }
  int& ChangeValue() { ++writes; return item; }
};
struct ReadOnly : NCollection_ListNode {
  int item = 42;
  ReadOnly() : NCollection_ListNode(nullptr) {}
  const int& Value() const { return item; }
};
struct List : NCollection_BaseList {
  template<class N, bool C> using Cursor = BasicIterator<N, int, C>;
  void Push(NCollection_ListNode* node) { pAppend(node); }
};
int main() {
  Node a(7), b(9); List list; list.Push(&a); list.Push(&b);
  List::Cursor<Node, false> mut(list);
  std::cout << "initial " << mut.More() << ' ' << (mut.myPrevious == nullptr) << '\n';
  mut.Value() = 8; (void)*mut; int* pointer = mut.operator->(); mut.ChangeValue() += 2;
  List::Cursor<Node, true> c(mut); (void)c.Value(); (void)*c; const int* cp = c.operator->();
  std::cout << "dispatch " << a.item << ' ' << a.reads << ' ' << a.writes << ' ' << (pointer == cp) << '\n';
  auto old = mut++; bool second = mut.myCurrent == &b && mut.myPrevious == &a;
  mut.Next(); auto previous = mut.myPrevious; ++mut;
  std::cout << "movement " << (old.myCurrent == &a) << ' ' << second << ' ' << !mut.More() << ' ' << (previous == mut.myPrevious) << '\n';
  mut.Init(list); ++mut; List::Cursor<Node,true> converted(mut), assigned; assigned = mut;
  std::cout << "conversion " << (converted.myCurrent == &b && converted.myPrevious == &a) << ' ' << (assigned == mut) << '\n';
  auto differing = mut; differing.myPrevious = nullptr;
  std::cout << "equality " << (differing == mut) << ' ' << (mut != old) << ' ' << assigned.IsEqual(mut) << '\n';
  mut.Initialize(list); c.Init(list); assigned.Initialize(list);
  std::cout << "reset " << (mut.myCurrent == &a) << ' ' << (c == assigned) << ' ' << (&mut.Iterator() == &mut) << ' ' << (&mut.ChangeIterator() == &mut) << '\n';
  std::cout << "offset " << (static_cast<void*>(&a) != static_cast<void*>(static_cast<NCollection_ListNode*>(&a))) << ' ' << c.Value() << '\n';
  ReadOnly readonly; List readList; readList.Push(&readonly); List::Cursor<ReadOnly,true> read(readList);
  std::cout << "readonly_only " << read.Value() << ' ' << *read << ' ' << *read.operator->() << '\n';
  List::Cursor<Node,false> empty; auto before = empty++; empty.Next();
  std::cout << "empty " << !empty.More() << ' ' << (before == empty) << ' ' << (empty.myPrevious == nullptr) << '\n';
}
