// Pinned OCCT3d097a: actual Cell.Objects and Object-first ListNode callback phases.
// -fno-access-control exposes protected state without replacing production method bodies.
#include <NCollection_CellFilter.hxx>
#include <iostream>
#include <map>
#include <sstream>
struct Arena:NCollection_IncAllocator {
 int allocations=0,frees=0;
 Arena():NCollection_IncAllocator(1024){}
 void* Allocate(size_t size)override{++allocations;return NCollection_IncAllocator::Allocate(size);}
 void Free(void*pointer)override{++frees;NCollection_IncAllocator::Free(pointer);}
};
struct Target {int number;bool stored=false;Target(int n):number(n){};Target(const Target&);~Target();};
struct Inspector {
 using Point=double;using Target=::Target;static constexpr int Dimension=1;
 bool all=false;
 static double Coord(int,const double&p){return p;}
 static bool IsEqual(const Target&a,const Target&b){return a.number==b.number;}
 NCollection_CellFilter_Action Inspect(const Target&);
};
using Filter=NCollection_CellFilter<Inspector>;
const Filter::Cell* observed=nullptr;Arena* arena=nullptr;
std::map<const void*,int> labels;std::string events;
Target::Target(const Target&source):number(source.number),stored(true){events+="C"+std::to_string(number)+":"+std::to_string(arena->allocations)+";";}
std::string edges(){std::string result;for(auto*n=observed->Objects;n;n=n->Next)result+=std::to_string(labels.at(n))+">"+std::to_string(n->Next?labels.at(n->Next):0)+",";return result;}
Target::~Target(){if(stored&&observed)events+="D"+std::to_string(number)+"["+edges()+"];";}
NCollection_CellFilter_Action Inspector::Inspect(const Target&t){events+="I"+std::to_string(t.number)+";";return all||t.number%2==0?CellFilter_Purge:CellFilter_Keep;}
void bind_observer(Filter&filter){
 Filter::Cell lookup(0.,filter.myCellSize);observed=&filter.myCells.Contained(lookup)->get();
 labels.clear();for(auto*n=observed->Objects;n;n=n->Next)labels[n]=n->Object.number;
}
void remaining(const Filter&filter){
 std::cout<<" R";Filter::Cell lookup(0.,filter.myCellSize);auto found=filter.myCells.Contained(lookup);
 if(found)for(auto*n=found->get().Objects;n;n=n->Next)std::cout<<n->Object.number<<',';
 std::cout<<" cells"<<filter.myCells.Size()<<'\n';
}
void scenario(int mode){
 occ::handle<Arena> resource=new Arena;arena=resource.get();Filter filter(1.,resource);events.clear();
 for(int n=1;n<=3;++n){Target target(n);filter.Add(target,0.);}
 std::cout<<"copy"<<mode<<' '<<events<<'\n';bind_observer(filter);events.clear();
 if(mode<3){Target target(3-mode);filter.Remove(target,0.);}
 else if(mode<5){Inspector inspector;inspector.all=mode==3;filter.Inspect(0.,inspector);}
 else if(mode==5)filter.Reset(2.,resource);else filter.Reset(2.);
 std::cout<<"phase"<<mode<<' '<<events;observed=nullptr;remaining(filter);
 std::cerr<<"free"<<mode<<' '<<resource->frees<<'\n';
}
int main(){for(int mode=0;mode<7;++mode)scenario(mode);}
