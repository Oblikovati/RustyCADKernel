// Shared native UBTree payload fixture; OCCT 3d097a0328 behavior oracle.
#pragma once
#include <NCollection_UBTree.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct Trace {
  static std::string events;
  static char rejected;
  static const void *object, *bounds;
  static bool stable, parents;
  static void (*observe)();
  static void event(char phase,char kind,int value,const void* address) {
    events+=std::string{phase,kind}+std::to_string(value)+",";
    if(phase=='A') stable &= address==(kind=='O'?object:bounds);
    if(phase=='D' && observe) observe();
    if(phase=='A' && rejected==kind) { rejected=0;throw std::runtime_error("rejected assignment"); }
  }
  static void clear() { events.clear();rejected=0;stable=parents=true;observe=nullptr; }
};
std::string Trace::events;char Trace::rejected=0;
const void *Trace::object=nullptr,*Trace::bounds=nullptr;
bool Trace::stable=true,Trace::parents=true;void (*Trace::observe)()=nullptr;
template<char Kind> struct Item {
  int value=0;
  Item()=default;explicit Item(int n):value(n) {}
  Item(const Item& source):value(source.value) { Trace::event('C',Kind,value,this); }
  Item& operator=(const Item& source) { value=source.value;Trace::event('A',Kind,value,this);return *this; }
  ~Item() { Trace::event('D',Kind,value,this); }
  void Add(const Item&) {} bool IsOut(const Item&) const { return false; }
  double SquareExtent() const { return 0; }
};
using Object=Item<'O'>;using Bounds=Item<'B'>;
using Tree=NCollection_UBTree<Object,Bounds>;using Node=Tree::TreeNode;
static void populate(Tree& tree) {
  tree.Add(Object(1),Bounds(1));Node& root=const_cast<Node&>(tree.Root());
  root.Gemmate(Bounds(10),Object(2),Bounds(2),tree.Allocator());
  root.ChangeChild(0).Gemmate(Bounds(30),Object(3),Bounds(3),tree.Allocator());
  root.ChangeChild(1).Gemmate(Bounds(40),Object(4),Bounds(4),tree.Allocator());
}
static void watch(Node& node) { Trace::object=&node.Object();Trace::bounds=&node.Bnd(); }
