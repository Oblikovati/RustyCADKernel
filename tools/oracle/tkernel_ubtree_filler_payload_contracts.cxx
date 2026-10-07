// Native queue temporary construction and explicit self-assignment payload phases.
#include "tkernel_ubtree_allocation_trace.hxx"
#include <NCollection_UBTreeFiller.hxx>
template<char Kind>struct MovablePayload:Payload<Kind> {
  using Payload<Kind>::Payload;
  MovablePayload()=default;
  MovablePayload(const MovablePayload& source):Payload<Kind>(source){}
  MovablePayload(MovablePayload&& source):Payload<Kind>(source.value) {
    source.value=-source.value;PairTrace::event('M',Kind,this->value);
  }
  MovablePayload& operator=(const MovablePayload& source) {
    Payload<Kind>::operator=(source);return *this;
  }
};
static std::string events() {PairTrace::active=false;return PairTrace::events.empty()?"-":PairTrace::events;}
template<class ObjectType,class BoundsType>static void queued(bool moving,int before,char phase,char kind,int ordinal) {
  using Pending=NCollection_UBTreeFiller<ObjectType,BoundsType>;
  occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
  NCollection_UBTree<ObjectType,BoundsType> tree;const ObjectType object(9);const BoundsType bounds(90);
  std::string added,cleared;bool failed=false;size_t after=0,remaining=0;
  {Pending filler(tree,arena,true);if(before)filler.Add(ObjectType(1),BoundsType(10));
   PairTrace::reset(phase,kind,ordinal);
   try{filler.Add(object,bounds);}catch(const std::runtime_error&){failed=true;}
   added=events();after=arena->blocks.size();PairTrace::reset();filler.Reset();
   cleared=events();remaining=arena->blocks.size();if(filler.Fill()!=0)std::abort();
  }
  std::cout<<"queued "<<moving<<' '<<before<<' '<<phase<<kind<<ordinal<<' '<<failed<<' '
    <<added<<' '<<after<<' '<<cleared<<' '<<remaining<<' '<<arena->blocks.size()<<' '<<tree.IsEmpty()<<'\n';
}
static void self_assignment(char kind) {
  using Pair=NCollection_UBTreeFiller<Object,Bounds>::ObjBnd;
  occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
  const Object object(9);const Bounds bounds(90);Pair pair(object,bounds);bool failed=false;
  PairTrace::reset('A',kind,1);try{pair=pair;}catch(const std::runtime_error&){failed=true;}
  std::cout<<"self "<<kind<<' '<<failed<<' '<<events()<<' '<<pair.myObj.value<<' '<<pair.myBnd.value<<'\n';
}
static void fill_phases(int count,bool full,char kind) {
  occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
  Tree tree(arena);NCollection_UBTreeFiller<Object,Bounds> filler(tree,nullptr,full);
  for(int index=1;index<=count;++index)filler.Add(Object(index),Bounds(index*10));
  PairTrace::reset('A',kind,1);bool failed=false;int result=-1;
  try{result=filler.Fill();}catch(const std::runtime_error&){failed=true;}
  const auto filled=events();PairTrace::reset();filler.Reset();const auto reset=events();
  std::cout<<"filled "<<count<<' '<<full<<' '<<kind<<' '<<failed<<' '<<result<<' '<<filled<<' '
    <<reset<<' '<<tree.Root().Bnd().value<<' '<<arena->blocks.size()<<'\n';
}
int main() {
  for(int before:{0,1}) {
    queued<Object,Bounds>(false,before,'X','X',0);
    for(char kind:{'O','B'})for(int ordinal:{1,2})queued<Object,Bounds>(false,before,'C',kind,ordinal);
    for(char kind:{'B','O'})queued<Object,Bounds>(false,before,'D',kind,1);
    queued<MovablePayload<'O'>,MovablePayload<'B'>>(true,before,'X','X',0);
    for(char phase:{'C','M','D'})for(char kind:{'O','B'})queued<MovablePayload<'O'>,MovablePayload<'B'>>(true,before,phase,kind,1);
  }
  for(char kind:{'X','O','B'})self_assignment(kind);
  for(int count:{1,2,3})for(bool full:{false,true})fill_phases(count,full,'X');
  for(int count:{1,2})for(bool full:{false,true})for(char kind:{'O','B'})fill_phases(count,full,kind);
}
