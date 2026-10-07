// Full filler phases: observation evidence, with known queue/self-assignment gaps.
#include "tkernel_ubtree_allocation_trace.hxx"
bool operator==(const Object& left,const Object& right){return left.value==right.value;}
namespace std {template<>struct hash<Object>{size_t operator()(const Object& object)const{return size_t(object.value);}};}
#include <NCollection_EBTree.hxx>
#include <NCollection_UBTreeFiller.hxx>
using Editable=NCollection_EBTree<Object,Bounds>;
using Filler=NCollection_UBTreeFiller<Object,Bounds>;
static void queue(Filler& filler,int count,bool duplicate){
 for(int index=1;index<=count;++index){const int value=duplicate?1:index;filler.Add(Object(value),Bounds(value*10));}
}
static void operation(int count,bool full,bool editable,bool duplicate,bool implicit){
 occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
 occ::handle<TrackingArena> pending=new TrackingArena;
 Tree base(arena);Editable derived(arena);Tree& tree=editable?static_cast<Tree&>(derived):base;
 std::string appended,filled;int result=-1;
 {Filler filler(tree,pending,full);PairTrace::reset();queue(filler,count,duplicate);
  PairTrace::active=false;appended=PairTrace::events;PairTrace::reset();
  if(!implicit)result=filler.Fill();
 }
 PairTrace::active=false;filled=PairTrace::events;
 std::cout<<count<<' '<<full<<' '<<editable<<' '<<duplicate<<' '<<implicit<<' '<<appended<<' '
   <<filled<<' '<<result<<' '<<arena->blocks.size()<<' '<<pending->blocks.size()<<'\n';
}
int main(){
 for(int count:{1,2,3})for(bool full:{false,true})operation(count,full,false,false,false);
 for(bool full:{false,true})operation(3,full,true,true,false);
 operation(2,true,false,false,true);operation(3,true,true,true,true);
}
