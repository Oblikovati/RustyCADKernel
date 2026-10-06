#include "tkernel_ubtree_trace.hxx"
static Node *parent=nullptr,*first=nullptr,*second=nullptr;
static void observe_parents() {
  Trace::parents &= &first->Parent()==parent && &second->Parent()==parent;
}
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
