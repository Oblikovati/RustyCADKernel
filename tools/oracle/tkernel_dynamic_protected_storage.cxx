#include <NCollection_DynamicArray.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <limits>
struct Arena : NCollection_BaseAllocator {
  int allocations=0, frees=0;
  void* AllocateOptimal(size_t n) override { ++allocations; return NCollection_BaseAllocator::AllocateOptimal(n); }
  void Free(void* p) override { ++frees; NCollection_BaseAllocator::Free(p); }
};
struct Item {
  static int constructed, destroyed;
  int value;
  explicit Item(int v=0):value(v) { ++constructed; }
  ~Item() { ++destroyed; }
};
int Item::constructed=0; int Item::destroyed=0;
struct Engine : NCollection_DynamicArray<Item> {
  using NCollection_DynamicArray<Item>::NCollection_DynamicArray;
  using NCollection_DynamicArray<Item>::roundUpPow2;
  using NCollection_DynamicArray<Item>::log2Pow2;
  using NCollection_DynamicArray<Item>::ensureStorageForIndex;
  using NCollection_DynamicArray<Item>::ensureBlockCount;
  using NCollection_DynamicArray<Item>::expandBlock;
  using NCollection_DynamicArray<Item>::blockCount;
  using NCollection_DynamicArray<Item>::availableSize;
  using NCollection_DynamicArray<Item>::blockAddress;
  using NCollection_DynamicArray<Item>::blockAt;
  using NCollection_DynamicArray<Item>::clearBlocks;
  using NCollection_DynamicArray<Item>::at;
};
static void snapshot(const char* name,const Engine& e,const Arena& a) {
  std::cout<<name<<' '<<e.blockCount()<<' '<<e.availableSize()<<' '<<e.Size()<<' '
           <<Item::constructed<<' '<<Item::destroyed<<' '<<a.allocations<<' '<<a.frees<<'\n';
}
static void numeric() {
  const size_t high=size_t(1)<<(std::numeric_limits<size_t>::digits-1);
  std::cout<<"round";
  for(size_t n : {size_t(0),size_t(1),size_t(3),size_t(8),size_t(9),high,high+1,size_t(-1)})
    std::cout<<' '<<Engine::roundUpPow2(n);
  std::cout<<"\nlog";
  for(size_t n : {size_t(0),size_t(1),size_t(3),size_t(8),size_t(9),high,size_t(-1)})
    std::cout<<' '<<Engine::log2Pow2(n);
  std::cout<<'\n';
}
static void growing(Engine& e,Arena& a) {
  e.ensureBlockCount(0,sizeof(Item)); snapshot("reserve_zero",e,a);
  e.ensureStorageForIndex(8,sizeof(Item)); snapshot("reserve_index8",e,a);
  void* first=e.blockAddress(0); const Engine& c=e;
  std::cout<<"addresses "<<(first==c.blockAddress(0))<<' '<<(first==e.blockAt(0))<<' '
           <<(first==c.blockAt(0))<<' '<<(first!=e.blockAddress(1))<<'\n';
  e.ensureBlockCount(2,sizeof(Item)); snapshot("reserve_smaller",e,a);
  auto* last=e.expandBlock(sizeof(Item)); snapshot("expand",e,a);
  std::cout<<"expand_identity "<<(last==e.blockAddress(3))<<' '<<(first==e.blockAddress(0))<<'\n';
  e.EmplaceAppend(11); e.at(0).value=12;
  std::cout<<"initialized_at "<<c.at(0).value<<'\n'; snapshot("one_initialized",e,a);
}
static void retained(Engine& e,Arena& a) {
  void* first=e.blockAddress(0);
  e.Clear(false); snapshot("clear_retain",e,a);
  e.SetIncrement(size_t(2)); snapshot("decrease_stride",e,a);
  std::cout<<"retained_identity "<<(first==e.blockAddress(0))<<'\n';
  e.clearBlocks(); snapshot("clear_empty_blocks",e,a);
  e.EmplaceAppend(22); snapshot("before_clear_live_blocks",e,a);
  e.clearBlocks(); snapshot("clear_live_blocks",e,a);
}
int main() {
  numeric(); occ::handle<Arena> arena=new Arena; Engine engine(size_t(3),arena);
  growing(engine,*arena); retained(engine,*arena);
}
