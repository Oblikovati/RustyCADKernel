#include <NCollection_DynamicArray.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct CopyArena : NCollection_BaseAllocator {
  int allocations=0, frees=0; void* last=nullptr;
  void* AllocateOptimal(size_t bytes) override {
    ++allocations;last=NCollection_BaseAllocator::AllocateOptimal(bytes);return last;
  }
  void Free(void* pointer) override { ++frees;NCollection_BaseAllocator::Free(pointer); }
};
struct CopyItem {
  static int attempts, succeeded, destroyed, throwAt;
  int value;
  explicit CopyItem(int number):value(number) {}
  CopyItem(const CopyItem& other):value(other.value) {
    ++attempts;if(attempts==throwAt) throw std::runtime_error("copy failure");++succeeded;
  }
  ~CopyItem() { ++destroyed; }
};
int CopyItem::attempts=0;int CopyItem::succeeded=0;int CopyItem::destroyed=0;int CopyItem::throwAt=0;
template<class T> struct CopyEngine : NCollection_DynamicArray<T> {
  using NCollection_DynamicArray<T>::NCollection_DynamicArray;
  using NCollection_DynamicArray<T>::copyDate;
  using NCollection_DynamicArray<T>::blockCount;
  using NCollection_DynamicArray<T>::ensureBlockCount;
};
static int valueOf(int value) { return value; }
static int valueOf(const CopyItem& value) { return value.value; }
template<class T> static std::string values(const CopyEngine<T>& engine) {
  std::string output;
  for(size_t index=0;index<engine.Size();++index) {
    if(index) output+=',';
    output+=std::to_string(valueOf(engine.Value(index)));
  }
  return output.empty()?"-":output;
}
static std::string partialValues(CopyItem* pointer,size_t count) {
  std::string output;
  for(size_t index=0;index<count;++index) {
    if(index) output+=',';
    output+=std::to_string(pointer[index].value);
  }
  return output.empty()?"-":output;
}
template<class T> static void row(const std::string& name,const CopyEngine<T>& engine,
                                  const CopyArena& arena,size_t partial=0,CopyItem* pointer=nullptr) {
  std::cout<<name<<' '<<engine.Size()<<' '<<engine.blockCount()<<' '<<arena.allocations<<' '
           <<arena.frees<<' '<<CopyItem::attempts<<' '<<CopyItem::destroyed<<' '<<partial<<' '
           <<values(engine)<<' '<<partialValues(pointer,partial)<<'\n';
}
static void resetCounters(int failure) {
  CopyItem::attempts=0;CopyItem::succeeded=0;CopyItem::destroyed=0;CopyItem::throwAt=failure;
}
static void recover(CopyEngine<CopyItem>& target,CopyArena& arena,const std::string& name) {
  const size_t count=CopyItem::succeeded-target.Size();
  auto* pointer=static_cast<CopyItem*>(arena.last);
  row(name+"_failed",target,arena,count,pointer);
  // Native copyDate leaked this unpublished allocation; the observing owner repairs that leak.
  for(size_t index=0;index<count;++index) pointer[index].~CopyItem();
  arena.Free(pointer);row(name+"_recovered",target,arena);
}
static void nontrivial(const std::string& name,int failure) {
  CopyEngine<CopyItem> source(size_t(2));
  for(int value=1;value<=3;++value) source.EmplaceAppend(value);
  source.ensureBlockCount(3,sizeof(CopyItem));
  occ::handle<CopyArena> arena=new CopyArena;CopyEngine<CopyItem> target(size_t(2),arena);
  resetCounters(failure);
  try { target.copyDate(source);row(name+"_copied",target,*arena); }
  catch(const std::runtime_error&) { recover(target,*arena,name); }
  target.Clear(true);row(name+"_cleared",target,*arena);
}
static void trivial(const std::string& name,int used,size_t blocks) {
  CopyEngine<int> source(size_t(2));
  for(int value=1;value<=used;++value) source.Append(value);
  source.ensureBlockCount(blocks,sizeof(int));
  occ::handle<CopyArena> arena=new CopyArena;CopyEngine<int> target(size_t(2),arena);
  resetCounters(0);target.copyDate(source);row(name+"_copied",target,*arena);
  target.Clear(true);row(name+"_cleared",target,*arena);
}
int main() {
  nontrivial("constructor",0);nontrivial("first_failure",1);
  nontrivial("partial_failure",2);nontrivial("second_block_failure",3);
  trivial("trivial",3,3);trivial("empty_retained",0,3);trivial("empty",0,0);
}
