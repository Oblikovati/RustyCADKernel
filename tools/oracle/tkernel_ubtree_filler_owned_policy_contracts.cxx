// OCCT 3d097a0328: noexcept automatic Fill, recoverable explicit Fill.
#include "tkernel_ubtree_allocation_trace.hxx"
#include <NCollection_UBTreeFiller.hxx>
#include <exception>
using Filler=NCollection_UBTreeFiller<Object,Bounds>;
static Tree* observed=nullptr;
static void terminate_snapshot(){
 PairTrace::active=false;std::cout<<"terminate "<<PairTrace::events<<' '
 <<(observed->IsEmpty()?-1:observed->Root().Object().value)<<' '
 <<PairTrace::arena->blocks.size()<<'\n';std::cout.flush();std::_Exit(86);
}
static void retry(Filler& filler){
 try{filler.Fill();}catch(const std::runtime_error&){
  PairTrace::active=false;std::cout<<"caught "<<PairTrace::events<<'\n';
  PairTrace::reset();int result=filler.Fill();PairTrace::active=false;
  std::cout<<"retry "<<result<<' '<<PairTrace::events<<'\n';
 }
}
static void operation(const char* mode){
 occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
 Tree tree(arena);observed=&tree;
 try{Filler filler(tree);filler.Add(Object(1),Bounds(10));PairTrace::reset('A',mode[0],1);
  if(mode[1]=='e'){retry(filler);}
  else if(mode[1]=='u'){throw std::runtime_error("outer exception");}
 }catch(const std::runtime_error&){std::cout<<"outer caught\n";}
}
int main(int argc,char** argv){
 if(argc==1){std::cout<<"nothrow "<<std::is_nothrow_destructible_v<Filler><<' '
  <<std::is_nothrow_destructible_v<NCollection_DynamicArray<Filler::ObjBnd>><<'\n';return 0;}
 std::set_terminate(terminate_snapshot);operation(argv[1]);
}
