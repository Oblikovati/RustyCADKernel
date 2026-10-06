#include <NCollection_DynamicArray.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <stdexcept>
struct Arena : NCollection_BaseAllocator {
  int allocations=0, frees=0; size_t bytes=0;
  void* AllocateOptimal(size_t n) override { ++allocations; bytes+=n; return NCollection_BaseAllocator::AllocateOptimal(n); }
  void Free(void* p) override { ++frees; NCollection_BaseAllocator::Free(p); }
};
struct Item {
  static int attempts, destroyed; int value;
  explicit Item(int v):value(v) { ++attempts; if(v==99) throw std::runtime_error("constructor"); }
  ~Item() { ++destroyed; }
};
int Item::attempts=0; int Item::destroyed=0;
struct Engine : NCollection_DynamicArray<Item> {
  using NCollection_DynamicArray<Item>::NCollection_DynamicArray;
  using NCollection_DynamicArray<Item>::allocateBlock;
  using NCollection_DynamicArray<Item>::deallocateBlock;
  using NCollection_DynamicArray<Item>::constructAt;
  using NCollection_DynamicArray<Item>::destroyAt;
  using NCollection_DynamicArray<Item>::setUsedSize;
  using NCollection_DynamicArray<Item>::ensureBlockCount;
  using NCollection_DynamicArray<Item>::blockAt;
  using NCollection_DynamicArray<Item>::blockCount;
  using NCollection_DynamicArray<Item>::copyParameters;
};
static void observe(const char* name,const Engine& e,const Arena& a) {
  std::cout<<name<<' '<<e.Size()<<' '<<e.blockCount()<<' '<<a.allocations<<' '<<a.frees<<' '
           <<a.bytes<<' '<<Item::attempts<<' '<<Item::destroyed<<'\n';
}
static void detached(Engine& e,Arena& a) {
  auto* p=static_cast<Item*>(e.allocateBlock(sizeof(Item))); observe("detached_alloc",e,a);
  e.constructAt(p,41); std::cout<<"detached_value "<<p->value<<'\n'; observe("detached_construct",e,a);
  e.destroyAt(p); observe("detached_destroy",e,a);
  e.deallocateBlock(p); observe("detached_free",e,a);
}
static void publication(Engine& e,Arena& a) {
  e.ensureBlockCount(2,sizeof(Item));
  Item* first=e.blockAt(0); Item* last=e.blockAt(1);
  e.constructAt(first,10); e.constructAt(first+1,20); e.constructAt(last,30);
  observe("constructed_unpublished",e,a); e.setUsedSize(3);
  std::cout<<"published_values "<<e[0].value<<' '<<e[1].value<<' '<<e[2].value<<'\n';
  observe("published_three",e,a); e.setUsedSize(1); observe("shrunk_one",e,a);
  e.setUsedSize(3); observe("restored_three",e,a);
  std::cout<<"restored_values "<<e[0].value<<' '<<e[1].value<<' '<<e[2].value<<'\n';
  e.setUsedSize(1); e.destroyAt(first+1); e.destroyAt(last); observe("destroyed_removed",e,a);
  e.Clear(true); observe("cleared_published",e,a);
}
static void throwing(Engine& e,Arena& a) {
  auto* p=static_cast<Item*>(e.allocateBlock(sizeof(Item)));
  try { e.constructAt(p,99); } catch(const std::runtime_error&) {}
  observe("failed_raw_constructor",e,a);
  // No Item lifetime began after the throw: skip destruction, retry construction.
  e.constructAt(p,42); e.destroyAt(p); e.deallocateBlock(p); observe("raw_retry_free",e,a);
}
static void parameters() {
  occ::handle<Arena> own=new Arena, other=new Arena;
  Engine source(size_t(4),other), keep(size_t(2),own), adopt(size_t(2),own);
  keep.copyParameters(source,false); adopt.copyParameters(source,true);
  void* p=keep.allocateBlock(sizeof(Item)); void* q=adopt.allocateBlock(sizeof(Item));
  std::cout<<"empty_copy_parameters "<<(keep.Allocator()==own)<<' '<<(adopt.Allocator()==other)<<' '
           <<keep.Size()<<' '<<adopt.Size()<<' '<<own->bytes<<' '<<other->bytes<<'\n';
  keep.deallocateBlock(p); adopt.deallocateBlock(q);
}
int main() {
  occ::handle<Arena> arena=new Arena; Engine engine(size_t(2),arena);
  detached(engine,*arena); publication(engine,*arena); throwing(engine,*arena); parameters();
}
