// Shared payload lifecycle fixture for pinned OCCT TreeNode and ObjBnd methods.
#pragma once
#include <NCollection_UBTree.hxx>
#include <NCollection_UBTreeFiller.hxx>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>
struct Trace {
  static std::string events;
  static char rejectPhase, rejectKind;
  static const void *object, *bounds;
  static bool stable;
  static void event(char phase, char kind, const void* address) {
    events += std::string{phase,kind}+",";
    const void* expected = kind == 'O' ? object : bounds;
    if (expected && (phase == 'A' || phase == 'D')) stable &= expected == address;
    if (rejectPhase == phase && rejectKind == kind) {
      rejectPhase = 0; throw std::runtime_error("rejected payload callback");
    }
  }
  static void clear() { events.clear(); rejectPhase=rejectKind=0; object=bounds=nullptr; stable=true; }
};
std::string Trace::events;
char Trace::rejectPhase=0,Trace::rejectKind=0;
const void* Trace::object=nullptr;const void* Trace::bounds=nullptr;
bool Trace::stable=true;
template<char Kind> struct Item {
  int value;
  Item():value(0) { Trace::event('N',Kind,this); }
  explicit Item(int n):value(n) {}
  Item(const Item& source):value(source.value) { Trace::event('C',Kind,this); }
  Item& operator=(const Item& source) { value=source.value;Trace::event('A',Kind,this);return *this; }
  ~Item() noexcept(false) { Trace::event('D',Kind,this); }
  void Add(const Item&) {}
  bool IsOut(const Item&) const { return false; }
  double SquareExtent() const { return 0; }
};
using Object=Item<'O'>;using Bounds=Item<'B'>;
using Tree=NCollection_UBTree<Object,Bounds>;
using Node=Tree::TreeNode;
using Pair=NCollection_UBTreeFiller<Object,Bounds>::ObjBnd;
static void watch(const Node& node) { Trace::object=&node.Object();Trace::bounds=&node.Bnd(); }
static void watch(const Pair& pair) { Trace::object=&pair.myObj;Trace::bounds=&pair.myBnd; }
static void row(const char* label) {
  std::cout << label << ' ' << Trace::events << ' ' << Trace::stable << '\n';Trace::clear();
}
