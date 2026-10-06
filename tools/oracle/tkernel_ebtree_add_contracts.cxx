// Native EBTree Add publication across tree and actual object map.
#include "tkernel_ubtree_allocation_trace.hxx"
#include <NCollection_EBTree.hxx>
#include <exception>
bool operator==(const Object& left,const Object& right) {
  PairTrace::event('Q','O',left.value);return left.value==right.value;
}
namespace std {
template<>struct hash<Object> {
  size_t operator()(const Object& object)const {
    PairTrace::event('H','O',object.value);return size_t(object.value%2);
  }
};
}
using Editable=NCollection_EBTree<Object,Bounds>;
using AddSignature=bool(Editable::*)(const Object&,const Bounds&);
static_assert(std::is_same_v<decltype(&Editable::Add),AddSignature>);
struct ExposedEditable: Editable {
  using Editable::Editable;
  int last_object() {return IsEmpty()?-1:this->ChangeLastNode().Object().value;}
};
static void populate(ExposedEditable& tree,int count) {
  for(int value=1;value<=count;++value)tree.Add(Object(value),Bounds(value*10));
}
static std::string binding(const ExposedEditable& tree,int value) {
  const Object key(value);if(!tree.Contains(key))return "-";
  const auto& node=tree.FindNode(key);
  return std::to_string(node.Object().value)+(node.IsLeaf()?"L":"B");
}
static void insertion(int count,int target,char phase,char kind,int ordinal) {
  occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
  ExposedEditable tree(arena);populate(tree,count);const Object object(target);const Bounds bounds(target*10);
  PairTrace::reset(phase,kind,ordinal);bool failed=false;int result=-1;
  try {result=tree.Add(object,bounds);}catch(const std::runtime_error&) {failed=true;}
  PairTrace::active=false;
  std::cout<<"editable "<<count<<' '<<target<<' '<<phase<<kind<<ordinal<<' '<<PairTrace::events<<' '
    <<failed<<' '<<result<<' '<<tree.last_object()<<' '<<(tree.IsEmpty()?-1:tree.Root().Bnd().value)
    <<' '<<binding(tree,1)<<' '<<binding(tree,target)<<' '<<arena->blocks.size()<<'\n';
  PairTrace::reset();tree.Clear();PairTrace::active=false;
  std::cout<<"clear "<<count<<' '<<target<<' '<<phase<<kind<<ordinal<<' '<<PairTrace::events<<' '
    <<arena->blocks.size()<<'\n';
}
static ExposedEditable* observed=nullptr;
static void termination_snapshot() {
  std::cout<<"terminate "<<PairTrace::events;PairTrace::active=false;
  if(observed)std::cout<<' '<<observed->last_object()<<' '<<observed->Root().Bnd().value
    <<' '<<binding(*observed,1)<<' '<<binding(*observed,9)<<' '<<PairTrace::arena->blocks.size();
  std::cout<<'\n';std::cout.flush();std::_Exit(86);
}
static void default_hasher_termination(char phase,int ordinal) {
  std::set_terminate(termination_snapshot);
  occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
  ExposedEditable tree(arena);populate(tree,1);observed=&tree;
  PairTrace::reset(phase,'O',ordinal);tree.Add(Object(9),Bounds(90));
}
int main(int argc,char** argv) {
  if(argc>1) {default_hasher_termination(argv[1][0],argv[1][1]-'0');return 1;}
  for(int count:{0,1,2,3})insertion(count,9,'X','X',0);
  for(int count:{1,2,3})insertion(count,1,'C','O',1);
  insertion(0,9,'C','B',1);insertion(0,9,'C','O',1);insertion(0,9,'C','O',2);
  for(int count:{1,2,3})insertion(count,9,'C','O',1);
  insertion(1,9,'C','B',1);insertion(1,9,'J','B',1);insertion(1,9,'D','B',1);
  insertion(1,9,'N','B',1);insertion(1,9,'N','O',2);insertion(1,9,'A','B',1);
  insertion(1,9,'A','B',3);insertion(1,9,'A','O',3);insertion(1,9,'N','O',3);
}
