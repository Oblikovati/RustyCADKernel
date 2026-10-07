// Only virtual Add spans are compared; filler temporary/self-assignment phases stay separate.
#include "tkernel_ubtree_allocation_trace.hxx"
bool operator==(const Object& left,const Object& right) {
  PairTrace::event('Q','O',left.value);return left.value==right.value;
}
namespace std {template<>struct hash<Object> {
  size_t operator()(const Object& object)const {PairTrace::event('H','O',object.value);return size_t(object.value%2);}
};}
#include <NCollection_EBTree.hxx>
#include <NCollection_UBTreeFiller.hxx>
#include <vector>
using Editable=NCollection_EBTree<Object,Bounds>;
using Filler=NCollection_UBTreeFiller<Object,Bounds>;
template<class Base>struct RecordingTree:Base {
  using Base::Base;std::string calls;int next=0,rejectedCall=0;char rejectedKind='X';int ordinal=0;
  bool Add(const Object& object,const Bounds& bounds) override {
    PairTrace::reset(++next==rejectedCall?'C':'X',rejectedKind,ordinal);int result=-1;
    try {result=Base::Add(object,bounds);}catch(...) {record(object.value,result);throw;}
    record(object.value,result);return result;
  }
  void record(int object,int result) {
    PairTrace::active=false;calls+=std::to_string(object)+":"+std::to_string(result)+":";
    calls+=(PairTrace::events.empty()?"-":PairTrace::events)+"|";
  }
  int last()const {return this->IsEmpty()?-1:const_cast<RecordingTree*>(this)->ChangeLastNode().Object().value;}
};
static void queue(Filler& filler,int count,bool duplicate) {
  for(int index=1;index<=count;++index){const int value=duplicate?1:index;filler.Add(Object(value),Bounds(value*10));}
}
template<class Base>static void run(bool editable,int count,bool full,bool duplicate,bool implicit,
  int rejectedCall=0,char kind='X',int ordinal=0) {
  occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
  occ::handle<TrackingArena> pending=new TrackingArena;RecordingTree<Base> tree(arena);
  tree.rejectedCall=rejectedCall;tree.rejectedKind=kind;tree.ordinal=ordinal;int result=-1;bool failed=false;
  {Filler filler(tree,pending,full);queue(filler,count,duplicate);
   if(!implicit){try{result=filler.Fill();}catch(const std::runtime_error&){failed=true;filler.Reset();}}
  }
  std::cout<<"dispatch "<<editable<<' '<<count<<' '<<full<<' '<<duplicate<<' '<<implicit<<' '
    <<rejectedCall<<kind<<ordinal<<' '<<result<<' '<<failed<<' '<<tree.calls<<' '<<tree.last()<<' '
    <<(tree.IsEmpty()?-1:tree.Root().Bnd().value)<<' '<<arena->blocks.size()<<' '<<pending->blocks.size()<<'\n';
}
int main() {
  for(int count:{1,2,3})for(bool full:{false,true}){run<Tree>(false,count,full,false,false);run<Editable>(true,count,full,false,false);}
  for(bool full:{false,true}){run<Editable>(true,3,full,true,false);run<Editable>(true,3,full,true,true);}
  run<Tree>(false,2,true,false,true);run<Editable>(true,2,false,false,true);
  for(char kind:{'B','O'}){run<Tree>(false,2,true,false,false,1,kind,1);run<Editable>(true,2,true,false,false,1,kind,1);}
  run<Editable>(true,1,true,false,false,1,'O',2);run<Editable>(true,2,true,false,false,2,'O',1);
}
