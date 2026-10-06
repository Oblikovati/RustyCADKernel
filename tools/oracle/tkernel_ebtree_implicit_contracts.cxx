// Native derived map member destruction precedes UBTree base destruction.
#include "tkernel_ubtree_allocation_trace.hxx"
#include <NCollection_EBTree.hxx>
bool operator==(const Object& left,const Object& right) {return left.value==right.value;}
namespace std {
template<>struct hash<Object> {size_t operator()(const Object& object)const {return size_t(object.value);}};
}
static void implicit_editable(int count) {
  occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
  const Object objects[]={Object(1),Object(2),Object(3)};const Bounds bounds[]={Bounds(10),Bounds(20),Bounds(30)};
  {
    NCollection_EBTree<Object,Bounds> tree(arena);PairTrace::reset();
    for(int index=0;index<count;++index)tree.Add(objects[index],bounds[index]);
    std::cout<<"construct "<<count<<' '<<PairTrace::events<<' '<<arena->blocks.size()<<'\n';
    PairTrace::reset();
  }
  PairTrace::active=false;
  std::cout<<"drop "<<count<<' '<<PairTrace::events<<' '<<arena->blocks.size()<<'\n';
}
int main() {for(int count:{0,1,2,3})implicit_editable(count);}
