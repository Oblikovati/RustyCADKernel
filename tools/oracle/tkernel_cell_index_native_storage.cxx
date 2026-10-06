// OCCT3d097a03: concrete CellIndex field offsets, empty heap moves and actual node requests.
#include <NCollection_CellFilter.hxx>
#include <iostream>
#include <vector>
struct Arena : NCollection_IncAllocator {
  std::vector<size_t> sizes;
  Arena():NCollection_IncAllocator(1024){}
  void* Allocate(size_t size) override {
    sizes.push_back(size);return NCollection_IncAllocator::Allocate(size);
  }
};
struct Inspector {
  using Target=int;using Point=double;static constexpr int Dimension=1;
  static double Coord(int,double point){return point;}
  static bool IsEqual(int first,int second){return first==second;}
};
using Filter=NCollection_CellFilter<Inspector>;using Index=Filter::CellIndex;
void layout_row() {
  std::cout<<"index "<<sizeof(Index)<<' '<<alignof(Index)<<' '<<offsetof(Index,myStorage)<<' '
           <<offsetof(Index,myPtr)<<' '<<offsetof(Index,mySize)<<'\n';
}
void allocation_row() {
  occ::handle<Arena> arena=new Arena;Filter filter(1.,arena);
  filter.Add(7,0.);filter.Add(8,0.);filter.Add(9,1.);
  std::cout<<"requests";for(size_t bytes:arena->sizes)std::cout<<' '<<bytes;std::cout<<'\n';
}
void empty_heap_rows() {
  Index first(11),second(24);const int* retained=second;second=std::move(first);
  std::cout<<"empty-donor "<<first.Size()<<' '<<first.isInline()<<' '<<(static_cast<const int*>(first)==retained)<<'\n';
  Index moved(std::move(first));
  std::cout<<"move-empty "<<moved.Size()<<' '<<moved.isInline()<<' '<<first.isInline()<<' '<<(static_cast<const int*>(moved)==retained)<<'\n';
  Filter::Cell lookup(moved);
  std::cout<<"copy-empty "<<lookup.index.Size()<<' '<<lookup.index.isInline()<<' '<<moved.isInline()<<'\n';
  Index receiver(11);const int* old=receiver;receiver=std::move(moved);
  std::cout<<"assign-empty "<<receiver.Size()<<' '<<receiver.isInline()<<' '<<moved.isInline()<<' '
           <<(static_cast<const int*>(receiver)==retained)<<' '<<(static_cast<const int*>(moved)==old)<<'\n';
  Index inlineSource(3);receiver=std::move(inlineSource);
  std::cout<<"return-inline "<<receiver.Size()<<' '<<receiver.isInline()<<' '<<inlineSource.Size()<<' '<<inlineSource.isInline()<<'\n';
}
void default_row() {
  Index empty;
  std::cout<<"default "<<empty.Size()<<' '<<empty.isInline()<<'\n';
}
int main(){layout_row();allocation_row();empty_heap_rows();default_row();}
