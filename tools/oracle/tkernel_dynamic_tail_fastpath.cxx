#include <NCollection_DynamicArray.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct TailArena : NCollection_BaseAllocator {
  int live=0;
  void* AllocateOptimal(size_t bytes) override { ++live;return NCollection_BaseAllocator::AllocateOptimal(bytes); }
  void Free(void* pointer) override { --live;NCollection_BaseAllocator::Free(pointer); }
};
template<class T> struct TailEngine : NCollection_DynamicArray<T> {
  using NCollection_DynamicArray<T>::NCollection_DynamicArray;
  using NCollection_DynamicArray<T>::ensureBlockCount;
  using NCollection_DynamicArray<T>::setUsedSize;
  using NCollection_DynamicArray<T>::blockCount;
};
static void growthRow(const std::string& name,const TailEngine<int>& target) {
  std::cout<<name<<' '<<target.Size()<<' '<<target.blockCount()<<' ';
  for(size_t index=0;index<target.Size();++index) { if(index)std::cout<<',';std::cout<<target.Value(index); }
  if(target.IsEmpty())std::cout<<'-';std::cout<<'\n';
}
static void growthRows() {
  TailEngine<int> target(size_t(2));target.ensureBlockCount(4,sizeof(int));
  for(int index=0;index<6;++index) { target.SetValue(index,index+10);growthRow("grow"+std::to_string(index),target); }
  target.Clear(false);growthRow("grow_clear",target);
  target.SetValue(7,99);growthRow("grow_reuse",target);
}
struct MoveTrace { int fail=1;std::string destroyed; };
struct MoveItem {
  int value;MoveTrace* trace;
  MoveItem(int input,MoveTrace* state):value(input),trace(state) {}
  ~MoveItem() noexcept(false) {
    if(!trace->destroyed.empty())trace->destroyed+=',';trace->destroyed+=std::to_string(value);
    if(trace->fail==value) { trace->fail=-1;throw std::runtime_error("destructor"); }
  }
};
static void moveRow(const char* name,const TailEngine<MoveItem>& target,
                    const TailEngine<MoveItem>& source,const TailArena& arena,const MoveTrace& trace) {
  std::cout<<name<<' '<<target.Size()<<' '<<source.Size()<<' '<<arena.live<<' '
           <<(source.IsEmpty()?-1:source.Value(0).value)<<' '<<trace.destroyed<<'\n';
}
static void moveRows() {
  occ::handle<TailArena> arena=new TailArena;MoveTrace trace;
  TailEngine<MoveItem> target(size_t(2),arena),source(size_t(2),arena);
  target.EmplaceAppend(1,&trace);source.EmplaceAppend(7,&trace);
  try { target.Assign(std::move(source)); }catch(const std::runtime_error&) {}
  moveRow("move_receiver_failed",target,source,*arena,trace);
  // The sole receiver lifetime ended before throw. Repair metadata before later Clear/destruction.
  target.setUsedSize(0);target.Clear(true);moveRow("move_receiver_repaired",target,source,*arena,trace);
  target.EmplaceAppend(2,&trace);target.Assign(std::move(source));
  moveRow("move_receiver_succeeded",target,source,*arena,trace);
  target.Clear(true);moveRow("move_receiver_cleared",target,source,*arena,trace);
}
int main() { growthRows();moveRows(); }
