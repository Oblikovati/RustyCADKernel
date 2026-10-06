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
static Node *parent=nullptr,*first=nullptr,*second=nullptr;
static void observe_parents() {
  Trace::parents &= &first->Parent()==parent && &second->Parent()==parent;
}
static void populate(Tree& tree) {
  tree.Add(Object(1),Bounds(1));Node& root=const_cast<Node&>(tree.Root());
  root.Gemmate(Bounds(10),Object(2),Bounds(2),tree.Allocator());
  root.ChangeChild(0).Gemmate(Bounds(30),Object(3),Bounds(3),tree.Allocator());
  root.ChangeChild(1).Gemmate(Bounds(40),Object(4),Bounds(4),tree.Allocator());
}
static void watch(Node& node) { Trace::object=&node.Object();Trace::bounds=&node.Bnd(); }
static void kill_branch(int killed) {
  Tree tree;populate(tree);Node& root=const_cast<Node&>(tree.Root());Node& kept=root.ChangeChild(1-killed);
  parent=&root;first=&kept.ChangeChild(0);second=&kept.ChangeChild(1);
  Trace::clear();watch(root);Trace::observe=observe_parents;
  root.Kill(killed,tree.Allocator());
  std::cout<<"kill "<<killed<<' '<<Trace::events<<' '<<Trace::stable<<' '<<Trace::parents<<' '
           <<root.Object().value<<' '<<root.Bnd().value<<' '<<root.IsRoot()<<'\n';Trace::clear();
}
static void kill_nested() {
  Tree tree;populate(tree);Node& node=const_cast<Node&>(tree.Root()).ChangeChild(0);
  Trace::clear();watch(node);node.Kill(0,tree.Allocator());
  std::cout<<"nested "<<Trace::events<<' '<<Trace::stable<<' '<<node.Object().value<<' '
           <<node.IsLeaf()<<' '<<(&node.Parent()==&tree.Root())<<'\n';Trace::clear();
}
static void kill_leaf() {
  Tree tree;populate(tree);Node& leaf=const_cast<Node&>(tree.Root()).ChangeChild(0).ChangeChild(0);
  Trace::clear();leaf.Kill(0,tree.Allocator());
  std::cout<<"leaf "<<(Trace::events.empty()?"empty":Trace::events)<<' '<<leaf.Object().value<<'\n';
}
static void assignment_failure(char kind) {
  Tree tree;populate(tree);Node& node=const_cast<Node&>(tree.Root()).ChangeChild(0);
  const Node* old0=&node.Child(0);const Node* old1=&node.Child(1);
  Trace::clear();watch(node);Trace::rejected=kind;bool failed=false;
  try {node.Kill(0,tree.Allocator());}catch(const std::runtime_error&) {failed=true;}
  std::cout<<"failure "<<kind<<' '<<Trace::events<<' '<<failed<<' '<<Trace::stable<<' '
           <<node.Object().value<<' '<<node.Bnd().value<<' '
           <<(&node.Child(0)==old0 && &node.Child(1)==old1 && &old1->Parent()==&node)<<'\n';Trace::clear();
}
int main() { kill_branch(0);kill_branch(1);kill_nested();kill_leaf();assignment_failure('B');assignment_failure('O'); }
