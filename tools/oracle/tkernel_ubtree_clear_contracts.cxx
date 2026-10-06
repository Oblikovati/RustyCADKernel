#include "tkernel_ubtree_trace.hxx"
#include <NCollection_HeapAllocator.hxx>
#include <NCollection_IncAllocator.hxx>
#include <NCollection_AccAllocator.hxx>
#include <NCollection_AlignedAllocator.hxx>
using Allocator=occ::handle<NCollection_BaseAllocator>;
static Tree* observedTree=nullptr;
static const NCollection_BaseAllocator* original=nullptr;
static bool retained=true,rootPresent=true;
static Allocator policy(int index) {
  switch(index) {
    case 0:return NCollection_BaseAllocator::CommonBaseAllocator();
    case 1:return Allocator(NCollection_HeapAllocator::GlobalHeapAllocator().get());
    case 2:return new NCollection_IncAllocator(1024);
    case 3:return new NCollection_AccAllocator(1024);
    default:return new NCollection_AlignedAllocator(32);
  }
}
static void observe_allocator() {
  retained &= observedTree->Allocator().get()==original;
  rootPresent &= !observedTree->IsEmpty();
}
static void clear_populated(int index,bool replace) {
  Allocator allocator=policy(index),replacement=policy(2);Tree tree(allocator);populate(tree);
  observedTree=&tree;original=allocator.get();retained=rootPresent=true;
  Trace::clear();Trace::observe=observe_allocator;
  tree.Clear(replace?replacement:Allocator());
  std::cout<<"clear "<<index<<' '<<replace<<' '<<Trace::events<<' '<<retained<<' '<<rootPresent<<' '
           <<tree.IsEmpty()<<' '<<(tree.Allocator()==(replace?replacement:allocator))<<'\n';Trace::clear();
}
static void clear_empty(bool replace) {
  Allocator allocator=policy(0),replacement=policy(2);Tree tree(allocator);Trace::clear();
  tree.Clear(replace?replacement:Allocator());tree.Clear();
  std::cout<<"empty "<<replace<<' '<<(Trace::events.empty()?"empty":Trace::events)<<' '
           <<tree.IsEmpty()<<' '<<(tree.Allocator()==(replace?replacement:allocator))<<'\n';Trace::clear();
}
int main() {
  for(int index=0;index!=5;++index) { clear_populated(index,false);clear_populated(index,true); }
  clear_empty(false);clear_empty(true);
}
