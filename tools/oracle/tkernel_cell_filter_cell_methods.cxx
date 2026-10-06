// OCCT 3d097a03: protected Cell callable ownership, index storage and hasher observations.
#include <NCollection_CellFilter.hxx>
#include <array>
#include <iostream>
#include <sstream>
#include <vector>
struct Arena : NCollection_IncAllocator {
  int allocations=0, frees=0;
  Arena():NCollection_IncAllocator(1024){}
  void* Allocate(size_t size) override { ++allocations; return NCollection_IncAllocator::Allocate(size); }
  void Free(void* value) override { ++frees; NCollection_IncAllocator::Free(value); }
};
struct Target {
  int number; std::string* events; bool copied=false;
  Target(int value,std::string& log):number(value),events(&log){}
  Target(const Target& other):number(other.number),events(other.events),copied(true){*events+="C"+std::to_string(number)+",";}
  ~Target(){if(copied)*events+="D"+std::to_string(number)+",";}
};
struct Policy {
  using Target=::Target; using Point=std::array<double,3>; static constexpr int Dimension=3;
  static double Coord(int axis,const Point& point){return point[axis];}
};
template<class I> struct Access : NCollection_CellFilter<I> {
  using typename NCollection_CellFilter<I>::Cell;
  using typename NCollection_CellFilter<I>::CellIndex;
  using typename NCollection_CellFilter<I>::ListNode;
  using typename NCollection_CellFilter<I>::CellHasher;
};
using Exposed=Access<Policy>;
Exposed::CellIndex index_of(size_t size,int offset) {
  Exposed::CellIndex index(size);
  for(size_t i=0;i<size;++i)index[i]=offset+int(i);
  return index;
}
void prepend(Exposed::Cell& cell,int value,std::string& events,Arena& arena) {
  Target source(value,events);
  auto* node=static_cast<Exposed::ListNode*>(arena.Allocate(sizeof(Exposed::ListNode)));
  new(&node->Object)Target(source);node->Next=cell.Objects;cell.Objects=node;
}
void copy_row(size_t size) {
  auto index=index_of(size,10);Exposed::Cell cell(index);
  if(size)index[0]=99;
  std::cout<<"copy "<<size<<" len "<<cell.index.Size()<<" first "<<(size?cell.index[0]:-1)
           <<" distinct "<<(static_cast<const int*>(cell.index)!=static_cast<const int*>(index))<<" empty "<<(cell.Objects==nullptr)<<'\n';
}
void construct_row(size_t size) {
  occ::handle<Arena> arena=new Arena;std::string events;
  {
    Exposed::Cell source(index_of(size,10));prepend(source,1,events,*arena);prepend(source,2,events,*arena);
    const int* buffer=source.index;auto* head=source.Objects;Exposed::Cell moved(std::move(source));
    std::cout<<"move "<<size<<" lens "<<moved.index.Size()<<' '<<source.index.Size()<<" head "<<(moved.Objects==head)<<' '<<(source.Objects==nullptr)
             <<" pointer "<<(static_cast<const int*>(moved.index)==buffer)<<" allocations "<<arena->allocations<<" events "<<events<<'\n';
  }
  std::cout<<"move-drop "<<size<<" events "<<events<<" frees "<<arena->frees<<'\n';
}
void assignment_row(size_t from,size_t to) {
  occ::handle<Arena> arena=new Arena;std::string events;
  {
    Exposed::Cell source(index_of(from,10)), destination(index_of(to,20));
    prepend(source,1,events,*arena);prepend(destination,2,events,*arena);
    const int* sourceBuffer=source.index;const int* destinationBuffer=destination.index;auto* head=source.Objects;
    destination=std::move(source);
    std::cout<<"assign "<<from<<' '<<to<<" lens "<<destination.index.Size()<<' '<<source.index.Size()<<" head "<<(destination.Objects==head)<<' '<<(source.Objects==nullptr)
             <<" pointer "<<(static_cast<const int*>(destination.index)==sourceBuffer)<<' '<<(static_cast<const int*>(source.index)==destinationBuffer)<<" events "<<events<<'\n';
  }
  std::cout<<"assign-drop "<<from<<' '<<to<<" events "<<events<<" frees "<<arena->frees<<'\n';
}
void self_row(size_t size) {
  occ::handle<Arena> arena=new Arena;std::string events;
  {
    Exposed::Cell cell(index_of(size,10));prepend(cell,1,events,*arena);prepend(cell,2,events,*arena);
    const int* buffer=cell.index;cell=std::move(cell);
    std::cout<<"self "<<size<<" len "<<cell.index.Size()<<" empty "<<(cell.Objects==nullptr)<<" pointer "<<(static_cast<const int*>(cell.index)==buffer)<<" events "<<events<<'\n';
  }
  std::cout<<"self-drop "<<size<<" events "<<events<<" frees "<<arena->frees<<'\n';
}
void equality_rows() {
  NCollection_Array1<double> sizes(-2,0);sizes(-2)=2;sizes(-1)=0.5;sizes(0)=4;
  Exposed::Cell point(Policy::Point{5,-1.9,12},sizes);auto index=index_of(3,0);index[0]=2;index[1]=-3;index[2]=3;
  Exposed::Cell equal(index),shorter(index_of(2,2)),unequal(index_of(3,2));Exposed::CellHasher hasher;
  std::cout<<"point";for(size_t i=0;i<point.index.Size();++i)std::cout<<' '<<point.index[i];
  std::cout<<" hash "<<hasher(point)<<" equal "<<point.IsEqual(equal)<<' '<<(point==equal)<<' '<<hasher(point,equal)<<' '<<point.IsEqual(shorter)<<' '<<point.IsEqual(unequal)<<'\n';
  occ::handle<Arena> arena=new Arena;std::string events;prepend(equal,7,events,*arena);
  std::cout<<"objects-ignored "<<point.IsEqual(equal)<<' '<<(hasher(point)==hasher(equal))<<'\n';
  // Payload observer must outlive equal's native destructor.
  equal.Objects->Object.~Target();equal.Objects=nullptr;
}
struct IntegerPolicy {using Target=int;using Point=double;static constexpr int Dimension=1;static double Coord(int,double value){return value;}};
void retained_bytes_row() {
  using Native=Access<IntegerPolicy>;occ::handle<Arena> arena=new Arena;
  auto* before=static_cast<char*>(arena->Allocate(8));Native::Cell cell(index_of(1,0));
  auto* node=static_cast<Native::ListNode*>(arena->Allocate(sizeof(Native::ListNode)));
  new(&node->Object)int(7);node->Next=nullptr;cell.Objects=node;cell=std::move(cell);
  auto* after=static_cast<char*>(arena->Allocate(8));
  std::cout<<"retained delta "<<(after-before)<<" node "<<sizeof(Native::ListNode)<<" frees "<<arena->frees<<'\n';
}
int main() {
  for(size_t size:{0u,1u,10u,11u,24u}){copy_row(size);construct_row(size);}
  for(size_t from:{3u,11u})for(size_t to:{3u,11u})assignment_row(from,to);
  self_row(3);self_row(11);equality_rows();retained_bytes_row();
}
