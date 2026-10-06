#include "tkernel_ubtree_trace.hxx"
#include <NCollection_IncAllocator.hxx>
#include <utility>
template<char Kind> bool operator==(const Item<Kind>& first,const Item<Kind>& second) {return first.value==second.value;}
namespace std {template<> struct hash<Object> {size_t operator()(const Object& value)const noexcept {return size_t(value.value);}};}
#include <NCollection_EBTree.hxx>
using Editable=NCollection_EBTree<Object,Bounds>;
using Allocator=occ::handle<NCollection_BaseAllocator>;
static const Editable *targetView=nullptr,*sourceView=nullptr;
static const NCollection_BaseAllocator* oldTargetAllocator=nullptr;
static const Node* sourceRoot=nullptr;
static bool allocatorStable=true,sourceStable=true;
static void observe_phase() {
  allocatorStable &= targetView->Allocator().get()==oldTargetAllocator;
  sourceStable &= (sourceView->IsEmpty()?nullptr:&sourceView->Root())==sourceRoot;
}
static void populate_editable(Editable& tree,int start) {
  for(int value=start;value<start+3;++value) tree.Add(Object(value),Bounds(value));
}
static const Node* root_of(const Editable& tree) {return tree.IsEmpty()?nullptr:&tree.Root();}
static void assignment(bool targetFull,bool sourceFull) {
  Allocator targetAllocator=new NCollection_IncAllocator(1024),sourceAllocator=new NCollection_IncAllocator(2048);
  Editable target(targetAllocator),source(sourceAllocator);if(targetFull)populate_editable(target,1);if(sourceFull)populate_editable(source,7);
  sourceRoot=root_of(source);targetView=&target;sourceView=&source;oldTargetAllocator=targetAllocator.get();allocatorStable=sourceStable=true;
  Trace::clear();Trace::observe=observe_phase;Editable& result=target.operator=(std::move(source));
  std::cout<<"assign "<<targetFull<<' '<<sourceFull<<' '<<(Trace::events.empty()?"empty":Trace::events)<<' '
   <<(&result==&target)<<' '<<(root_of(target)==sourceRoot)<<' '<<source.IsEmpty()<<' '
   <<(target.Allocator()==sourceAllocator)<<' '<<(source.Allocator()==targetAllocator)<<' '<<allocatorStable<<' '<<sourceStable<<'\n';Trace::clear();
  const bool oldMissing=!target.Contains(Object(1)),newPresent=target.Contains(Object(7)),sourceMissing=!source.Contains(Object(7));
  std::cout<<"bindings "<<targetFull<<' '<<sourceFull<<' '<<oldMissing<<' '<<(newPresent==sourceFull)<<' '<<sourceMissing<<'\n';Trace::clear();
}
static void clearing() {
  Editable tree;populate_editable(tree,1);Trace::clear();tree.Clear();
  std::cout<<"clear "<<Trace::events<<' '<<tree.IsEmpty()<<'\n';Trace::clear();
}
static void self_assignment(bool full) {
  Editable tree;if(full)populate_editable(tree,1);const Node* root=root_of(tree);const auto allocator=tree.Allocator();Trace::clear();
  Editable& result=tree.operator=(std::move(tree));
  std::cout<<"self "<<full<<' '<<(Trace::events.empty()?"empty":Trace::events)<<' '<<(&result==&tree)<<' '
   <<(root_of(tree)==root)<<' '<<(tree.Allocator()==allocator)<<'\n';Trace::clear();
}
static void clear_replace(bool full) {
  Allocator old=new NCollection_IncAllocator(1024),replacement=new NCollection_IncAllocator(2048);
  Editable tree(old);if(full)populate_editable(tree,1);Trace::clear();tree.Clear(replacement);
  std::cout<<"clear_replace "<<full<<' '<<(Trace::events.empty()?"empty":Trace::events)<<' '
    <<tree.IsEmpty()<<' '<<(tree.Allocator()==replacement)<<'\n';Trace::clear();
}
int main() {clearing();for(bool target:{false,true})for(bool source:{false,true})assignment(target,source);self_assignment(false);self_assignment(true);clear_replace(false);clear_replace(true);}
