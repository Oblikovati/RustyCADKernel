#include "tkernel_ubtree_trace.hxx"
#include <NCollection_IncAllocator.hxx>
#include <utility>
struct ObservedTree:Tree {
  using Tree::Tree;
  using Tree::ChangeLastNode;
};
static const Tree* observedSource=nullptr;
static const Tree* observedTarget=nullptr;
static const NCollection_BaseAllocator* oldAllocator=nullptr;
static bool targetStable=true;
static const Node* savedRoot=nullptr;
static bool sourceStable=true;
static const Node* root_of(const Tree& tree) { return tree.IsEmpty()?nullptr:&tree.Root(); }
static const Node* last_of(ObservedTree& tree) { return tree.IsEmpty()?nullptr:&tree.ChangeLastNode(); }
static void observe_source() {
  sourceStable &= root_of(*observedSource)==savedRoot;
  targetStable &= observedTarget->Allocator().get()==oldAllocator;
}
static void assignment(bool targetFull,bool sourceFull) {
  occ::handle<NCollection_BaseAllocator> allocator=new NCollection_IncAllocator(1024);
  ObservedTree source(allocator),target;if(sourceFull) populate(source);if(targetFull) populate(target);
  savedRoot=root_of(source);const Node* last=last_of(source);observedSource=&source;sourceStable=true;
  observedTarget=&target;oldAllocator=target.Allocator().get();targetStable=true;
  Trace::clear();Trace::observe=observe_source;
  Tree& result=static_cast<Tree&>(target).operator=(std::move(source));
  std::cout<<"assign "<<targetFull<<' '<<sourceFull<<' '<<(Trace::events.empty()?"empty":Trace::events)
           <<' '<<(&result==&target)<<' '<<(root_of(target)==savedRoot)<<' '<<(last_of(target)==last)
           <<' '<<source.IsEmpty()<<' '<<(target.Allocator()==allocator)<<' '
           <<(source.Allocator().get()==oldAllocator)<<' '<<sourceStable<<' '<<targetStable<<'\n';Trace::clear();
}
static void self_assignment(bool full) {
  ObservedTree tree;if(full) populate(tree);const Node* root=root_of(tree),*last=last_of(tree);
  const auto allocator=tree.Allocator();Trace::clear();
  Tree& result=static_cast<Tree&>(tree).operator=(std::move(tree));
  std::cout<<"self "<<full<<' '<<(Trace::events.empty()?"empty":Trace::events)<<' '
           <<(&result==&tree)<<' '<<(root_of(tree)==root)<<' '<<(last_of(tree)==last)<<' '
           <<(tree.Allocator()==allocator)<<'\n';Trace::clear();
}
static void construction(bool full) {
  occ::handle<NCollection_BaseAllocator> allocator=new NCollection_IncAllocator(1024);
  Tree source(allocator);if(full) populate(source);const Node* root=root_of(source);Trace::clear();
  Tree moved(std::move(source));
  std::cout<<"construct "<<full<<' '<<(Trace::events.empty()?"empty":Trace::events)<<' '
           <<(root_of(moved)==root)<<' '<<source.IsEmpty()<<' '<<(moved.Allocator()==allocator)
           <<' '<<source.Allocator().IsNull()<<'\n';Trace::clear();
}
int main() {
  for(bool target:{false,true})for(bool source:{false,true})assignment(target,source);
  for(bool full:{false,true}) {self_assignment(full);construction(full);}
}
