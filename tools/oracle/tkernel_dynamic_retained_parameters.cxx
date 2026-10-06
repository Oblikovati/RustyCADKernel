#include <NCollection_DynamicArray.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <NCollection_HeapAllocator.hxx>
#include <iostream>
#include <string>
#include <vector>
struct CommonOwner : NCollection_BaseAllocator {};
struct HeapOwner : NCollection_HeapAllocator {};
struct ParameterArena : NCollection_BaseAllocator {
  int allocated=0, freed=0;
  void* AllocateOptimal(size_t bytes) override { ++allocated;return NCollection_BaseAllocator::AllocateOptimal(bytes); }
  void Free(void* pointer) override { ++freed;NCollection_BaseAllocator::Free(pointer); }
};
struct ParameterItem {
  static int constructed,destroyed;int value;
  explicit ParameterItem(int input):value(input) { ++constructed; }
  ~ParameterItem() { ++destroyed; }
};
int ParameterItem::constructed=0;int ParameterItem::destroyed=0;
template<class T> struct ParameterEngine : NCollection_DynamicArray<T> {
  using NCollection_DynamicArray<T>::NCollection_DynamicArray;
  using NCollection_DynamicArray<T>::copyParameters;
  using NCollection_DynamicArray<T>::blockCount;
  using NCollection_DynamicArray<T>::availableSize;
  using NCollection_DynamicArray<T>::blockAt;
  using NCollection_DynamicArray<T>::ensureBlockCount;
  using NCollection_DynamicArray<T>::setUsedSize;
};
static std::string rawValues(const std::vector<ParameterItem*>& slots) {
  std::string output;
  for(const auto* slot:slots) { if(!output.empty()) output+=',';output+=std::to_string(slot->value); }
  return output.empty()?"-":output;
}
static void itemRow(const std::string& name,const ParameterEngine<ParameterItem>& target,
                    const ParameterArena& arena,const std::vector<ParameterItem*>& slots) {
  std::cout<<name<<' '<<target.Size()<<' '<<target.blockCount()<<' '<<target.availableSize()<<' '
           <<arena.allocated<<' '<<arena.freed<<' '<<ParameterItem::constructed<<' '
           <<ParameterItem::destroyed<<' '<<rawValues(slots)<<'\n';
}
static std::vector<ParameterItem*> appendItems(ParameterEngine<ParameterItem>& target,int first,int count) {
  std::vector<ParameterItem*> slots;
  for(int index=0;index<count;++index) slots.push_back(&target.EmplaceAppend(first+index));
  return slots;
}
static void resetItems() { ParameterItem::constructed=0;ParameterItem::destroyed=0; }
static void equalStride() {
  resetItems();occ::handle<ParameterArena> arena=new ParameterArena;
  ParameterEngine<ParameterItem> target(size_t(2),arena),source(size_t(2),arena);
  auto slots=appendItems(target,1,3);itemRow("equal_before",target,*arena,slots);
  target.copyParameters(source,true);itemRow("equal_forgotten",target,*arena,slots);
  target.setUsedSize(3);itemRow("equal_restored",target,*arena,slots);
  target.Clear(true);itemRow("equal_cleared",target,*arena,{});
}
static void smallerStride() {
  resetItems();occ::handle<ParameterArena> arena=new ParameterArena,other=new ParameterArena;
  ParameterEngine<ParameterItem> target(size_t(4),arena),source(size_t(2),other);
  auto slots=appendItems(target,1,6);itemRow("smaller_before",target,*arena,slots);
  target.copyParameters(source,false);itemRow("smaller_forgotten",target,*arena,slots);
  for(auto* slot:slots) slot->~ParameterItem();
  itemRow("smaller_destroyed_old",target,*arena,{});
  slots=appendItems(target,11,5);itemRow("smaller_reused",target,*arena,slots);
  target.Clear(true);itemRow("smaller_cleared",target,*arena,{});
}
static void regrowStride() {
  resetItems();occ::handle<ParameterArena> arena=new ParameterArena;
  ParameterEngine<ParameterItem> target(size_t(4),arena),small(size_t(2),arena),large(size_t(4),arena);
  auto slots=appendItems(target,1,3);target.copyParameters(small,true);
  itemRow("regrow_shrunk",target,*arena,slots);
  target.copyParameters(large,true);itemRow("regrow_expanded",target,*arena,slots);
  target.setUsedSize(3);itemRow("regrow_restored",target,*arena,slots);
  target.Clear(true);itemRow("regrow_cleared",target,*arena,{});
}
static void emptyRetained() {
  resetItems();occ::handle<ParameterArena> arena=new ParameterArena;
  ParameterEngine<ParameterItem> target(size_t(4),arena),source(size_t(2),arena);
  target.ensureBlockCount(3,sizeof(ParameterItem));target.copyParameters(source,true);
  itemRow("empty_retained_copied",target,*arena,{});
  auto slots=appendItems(target,21,2);itemRow("empty_retained_reused",target,*arena,slots);
  target.Clear(true);itemRow("empty_retained_cleared",target,*arena,{});
}
static void ownerRow(const std::string& name,const ParameterEngine<int>& target,
                     const occ::handle<NCollection_BaseAllocator>& owner,int* first,const std::string& values) {
  bool stable=target.blockCount()!=0 && target.blockAt(0)==first;
  std::cout<<name<<' '<<target.Size()<<' '<<target.blockCount()<<' '<<target.availableSize()<<' '
           <<(target.Allocator()==owner)<<' '<<stable<<' '<<values<<'\n';
}
static void ownerCase(const std::string& name,const occ::handle<NCollection_BaseAllocator>& oldOwner,
                     const occ::handle<NCollection_BaseAllocator>& newOwner) {
  ParameterEngine<int> target(size_t(2),oldOwner),source(size_t(2),newOwner);
  for(int value=1;value<=3;++value) target.Append(value);
  int* first=target.blockAt(0);int* second=target.blockAt(1);
  target.copyParameters(source,true);
  std::string values=std::to_string(first[0])+","+std::to_string(first[1])+","+std::to_string(second[0]);
  ownerRow(name+"_forgotten",target,newOwner,first,values);
  target.setUsedSize(3);ownerRow(name+"_restored",target,newOwner,first,values);
  target.Clear(true);ownerRow(name+"_cleared",target,newOwner,nullptr,"-");
}
int main() {
  equalStride();smallerStride();regrowStride();emptyRetained();
  // This oracle build uses USE_MMGR_TYPE=NATIVE: Standard and Heap both release through libc.
  ownerCase("common_to_heap",new CommonOwner,new HeapOwner);
  ownerCase("heap_to_common",new HeapOwner,new CommonOwner);
  ownerCase("heap_to_null",new HeapOwner,occ::handle<NCollection_BaseAllocator>());
}
