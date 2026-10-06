// CellFilter protected reset invariants and target placement-new ordering.
#include <NCollection_CellFilter.hxx>
#include <iostream>
#include <vector>
class ProbeAllocator : public NCollection_IncAllocator {
public:
  int allocations=0;
  void* Allocate(const size_t bytes) override {
    ++allocations;
    return NCollection_IncAllocator::Allocate(bytes);
  }
};
struct Target {
  int number;
  ProbeAllocator* arena;
  std::vector<int>* observed;
  Target(int n,ProbeAllocator* a,std::vector<int>* o):number(n),arena(a),observed(o){}
  Target(const Target& other):number(other.number),arena(other.arena),observed(other.observed){observed->push_back(arena->allocations);}
};
struct Inspector {
  using Target=::Target;
  using Point=double;
  static constexpr int Dimension=1;
  std::vector<int> visited;
  static double Coord(int,const double& point){return point;}
  static bool IsEqual(const Target& a,const Target& b){return a.number==b.number;}
  NCollection_CellFilter_Action Inspect(const Target& value){visited.push_back(value.number);return CellFilter_Keep;}
};
struct Geometry {
  using Target=int;
  using Point=double;
  static constexpr int Dimension=1;
  static double Coord(int,const double& point){return point;}
  static bool IsEqual(int a,int b){return a==b;}
  NCollection_CellFilter_Action Inspect(int){return CellFilter_Keep;}
};
struct ProbeFilter : NCollection_CellFilter<Geometry> {
  using NCollection_CellFilter<Geometry>::NCollection_CellFilter;
  size_t Buckets()const{return myCells.NbBuckets();}
  int Count()const{return myCells.Extent();}
  int Dimension()const{return myDim;}
  auto Allocator()const{return myAllocator;}
  int Lower()const{return myCellSize.Lower();}
  int Upper()const{return myCellSize.Upper();}
  double SizeAt(int index)const{return myCellSize(index);}
};
static void allocationOrder() {
  occ::handle<ProbeAllocator> arena=new ProbeAllocator;
  std::vector<int> observed;
  Target target(7,arena.get(),&observed);
  NCollection_CellFilter<Inspector> filter(1.0,arena);
  filter.Add(target,0.0);target.number=8;filter.Add(target,0.0);
  std::cout << "copy_allocations";for(int count:observed)std::cout << ' ' << count;std::cout << '\n';
  Inspector inspector;filter.Inspect(0.0,inspector);
  std::cout << "newest_first";for(int n:inspector.visited)std::cout << ' ' << n;std::cout << '\n';
}
static void resetBuckets() {
  occ::handle<NCollection_IncAllocator> arena=new NCollection_IncAllocator;
  ProbeFilter filter(1.0,arena);
  for(int i=0;i<103;++i)filter.Add(i,double(i));
  std::cout << "populated " << filter.Buckets() << '\n';
  filter.Reset(2.0,arena);
  std::cout << "reset_same " << filter.Buckets() << ' ' << filter.Count() << ' ' << (filter.Allocator()==arena) << '\n';
  filter.Reset(3.0);
  std::cout << "reset_replacement " << filter.Buckets() << ' ' << (filter.Allocator()!=arena) << '\n';
  NCollection_Array1<double> sizes(4,5);sizes(4)=2;sizes(5)=3;
  filter.Reset(sizes,arena);
  std::cout << "reset_sizes " << filter.Dimension() << ' ' << filter.Buckets() << ' ' << filter.Lower() << ' ' << filter.Upper() << ' ' << filter.SizeAt(4) << ' ' << filter.SizeAt(5) << '\n';
}
static void defaultsAndEnum() {
  ProbeFilter filter;
  std::cout << "default " << filter.Dimension() << ' ' << filter.Buckets() << ' ' << filter.SizeAt(0) << '\n';
  filter.Reset(1.0);
  std::cout << "initialized " << filter.SizeAt(0) << '\n';
  std::cout << "action_abi " << int(CellFilter_Keep) << ' ' << int(CellFilter_Purge) << ' ' << sizeof(NCollection_CellFilter_Action) << ' ' << alignof(NCollection_CellFilter_Action) << '\n';
}
int main(){allocationOrder();resetBuckets();defaultsAndEnum();}
