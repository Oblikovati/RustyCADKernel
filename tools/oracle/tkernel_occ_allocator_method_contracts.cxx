// NATIVE/glibc typed allocator forwarding, retained handle moves and realloc boundaries.
#include <NCollection_OccAllocator.hxx>
#include <NCollection_Allocator.hxx>
#include <NCollection_IncAllocator.hxx>
#include <NCollection_AccAllocator.hxx>
#include <Standard_OutOfMemory.hxx>
#include <Standard_ProgramError.hxx>
#include <iostream>
#include <limits>
static void handles() {
  occ::handle<NCollection_BaseAllocator> arena = new NCollection_IncAllocator;
  auto handle=arena;
  NCollection_OccAllocator<int> copied(handle),moved(std::move(handle));
  std::cout<<"handle_move "<<(copied==moved)<<' '<<(handle==arena)<<'\n';
  NCollection_OccAllocator<int> constructed(std::move(moved));
  std::cout<<"allocator_move "<<(constructed==moved)<<' '<<(moved.Allocator()==arena)<<'\n';
  NCollection_OccAllocator<int> source(arena);
  NCollection_OccAllocator<int> target;
  const bool assigned=&(target=source)==&target;
  const bool self=&(target=target)==&target;
  std::cout<<"assignment "<<assigned<<' '<<self<<' '<<(target.Allocator()==arena)<<'\n';
  const bool moveAssigned=&(target=std::move(constructed))==&target;
  std::cout<<"move_assignment "<<moveAssigned<<' '<<(target==constructed)<<' '<<(constructed.Allocator()==arena)<<'\n';
  std::cout<<"propagation "<<NCollection_OccAllocator<int>::propagate_on_container_move_assignment::value<<'\n';
}
static void stateless() {
  occ::handle<NCollection_BaseAllocator> null;
  NCollection_Allocator<int> target(null);
  occ::handle<NCollection_BaseAllocator> arena=new NCollection_IncAllocator;
  NCollection_Allocator<char> source(arena);
  const bool assigned=&(target=source)==&target;
  const bool self=&(target=target)==&target;
  std::cout<<"stateless "<<target.operator==(source)<<' '<<assigned<<' '<<self<<'\n';
}
static void rawOperations() {
  NCollection_OccAllocator<int> allocator;
  const char hint=17;
  int* pointer=allocator.allocate(1,&hint);allocator.construct(pointer,42);
  std::cout<<"hint_allocate "<<*pointer<<'\n';
  allocator.destroy(pointer);allocator.deallocate(reinterpret_cast<char*>(pointer),999);
  allocator.deallocate(static_cast<int*>(nullptr),0);
  std::cout<<"null_free 1\n";
  NCollection_OccAllocator<int> accumulating(new NCollection_AccAllocator(256));
  bool failed=false;
  try{accumulating.deallocate(static_cast<char*>(nullptr),0);}catch(const Standard_ProgramError&){failed=true;}
  std::cout<<"accumulating_null "<<failed<<'\n';
}
static void reallocations() {
  NCollection_Allocator<int> allocator;
  int* pointer=allocator.reallocate(nullptr,1);*pointer=42;
  pointer=allocator.reallocate(pointer,8);
  std::cout<<"null_reallocate "<<*pointer<<'\n';allocator.deallocate(pointer,8);
  pointer=allocator.reallocate(nullptr,0);
  std::cout<<"null_zero "<<(pointer!=nullptr)<<'\n';allocator.deallocate(pointer,0);
  pointer=allocator.allocate(1);
  try{int* zero=allocator.reallocate(pointer,0);std::cout<<"live_zero_failure 0\n";allocator.deallocate(zero,0);}
  catch(const Standard_OutOfMemory&){std::cout<<"live_zero_failure 1\n";}
  pointer=allocator.allocate(1);*pointer=42;bool failed=false;
  // Large, non-overflowed byte request; native failure leaves the original allocation live.
  try{int* huge=allocator.reallocate(pointer,std::numeric_limits<size_t>::max()/sizeof(int));allocator.deallocate(huge,0);}
  catch(const Standard_OutOfMemory&){failed=true;}
  if(failed){std::cout<<"nonzero_failure "<<failed<<' '<<*pointer<<'\n';allocator.deallocate(pointer,1);}
}
static int copies=0,drops=0;
struct Tracked {
  int number;
  explicit Tracked(int n):number(n){}
  Tracked(const Tracked& other):number(other.number){++copies;}
  ~Tracked(){++drops;}
};
static void construction() {
  NCollection_Allocator<Tracked> allocator;Tracked source(42);Tracked* pointer=allocator.allocate(1);
  allocator.construct(pointer,source);
  std::cout<<"copy_construct "<<copies<<' '<<pointer->number<<'\n';
  allocator.destroy(pointer);allocator.construct(pointer,source);allocator.destroy(pointer);allocator.deallocate(pointer,1);
  std::cout<<"construct_destroy "<<copies<<' '<<drops<<'\n';
}
int main(){handles();stateless();rawOperations();reallocations();construction();}
