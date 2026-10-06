// Actual UBTree Add including observable last-node publication before local-bound destruction.
#include "tkernel_ubtree_allocation_trace.hxx"
struct ExposedTree: Tree {
  using Tree::Tree;
  int last_object() {return IsEmpty()?-1:ChangeLastNode().Object().value;}
};
using AddSignature=bool(Tree::*)(const Object&,const Bounds&);
static_assert(std::is_same_v<decltype(&Tree::Add),AddSignature>);
static void populate_add(ExposedTree& tree,int count) {
  for(int item=1;item<=count;++item)tree.Add(Object(item),Bounds(item*10));
}
static void insertion(int count,char phase,char kind,int ordinal) {
  occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
  ExposedTree tree(arena);populate_add(tree,count);const Object object(9);const Bounds bounds(90);
  PairTrace::reset(phase,kind,ordinal);bool failed=false;
  try {tree.Add(object,bounds);}catch(const std::runtime_error&) {failed=true;}
  PairTrace::active=false;
  std::cout<<"add "<<count<<' '<<phase<<kind<<ordinal<<' '<<PairTrace::events<<' '<<failed<<' '
    <<tree.last_object()<<' '<<(tree.IsEmpty()?-1:tree.Root().Bnd().value)<<' '<<arena->blocks.size()<<'\n';
}
int main() {
  insertion(0,'X','X',0);insertion(0,'C','B',1);insertion(0,'C','O',1);
  for(int count:{1,2,3}) {
    insertion(count,'X','X',0);insertion(count,'R','B',1);insertion(count,'C','B',1);
    insertion(count,'J','B',1);insertion(count,'D','B',count==1?1:3);
  }
  insertion(1,'N','B',1);insertion(1,'N','O',2);insertion(1,'A','B',1);
  insertion(2,'E','B',1);insertion(2,'E','B',2);insertion(2,'C','B',2);
}
