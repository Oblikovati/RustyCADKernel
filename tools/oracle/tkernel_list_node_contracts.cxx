// Pinned OCCT ListNode/TListNode pointer, value, construction, and callback contracts.
#include <NCollection_TListNode.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct NodeAllocator : NCollection_BaseAllocator {
  std::string events;
  void* Allocate(size_t count) override { events+='A'; return NCollection_BaseAllocator::Allocate(count); }
  void Free(void* pointer) override { events+='F'; NCollection_BaseAllocator::Free(pointer); }
};
struct TrackedValue {
  int value;
  NodeAllocator* allocator;
  bool fail;
  TrackedValue(int v,NodeAllocator* a,bool f=false):value(v),allocator(a),fail(f) { allocator->events+='V'; }
  TrackedValue(const TrackedValue& other):value(other.value),allocator(other.allocator),fail(other.fail) {
    allocator->events+='C'; if(fail) throw std::runtime_error("copy failure");
  }
  ~TrackedValue() { allocator->events+='D'; }
};
static void links() {
  NCollection_ListNode tail(nullptr),head(&tail);
  const NCollection_ListNode& readable=head;
  std::cout<<"next "<<(readable.Next()==&tail)<<' '<<(tail.Next()==nullptr);
  head.Next()=nullptr;std::cout<<' '<<(readable.Next()==nullptr)<<'\n';
}
static void values() {
  NCollection_ListNode tail(nullptr);
  NCollection_TListNode<int> node(7,&tail);
  const NCollection_TListNode<int>& readable=node;
  std::cout<<"value "<<readable.Value()<<' '<<(node.Next()==&tail)<<' '
    <<(&node.ChangeValue()==&readable.Value());
  node.ChangeValue()=8;std::cout<<' '<<readable.Value()<<'\n';
}
static void callbacks() {
  occ::handle<NodeAllocator> concrete=new NodeAllocator;
  occ::handle<NCollection_BaseAllocator> allocator=concrete;
  NCollection_ListNode tail(nullptr);
  auto* node=new(allocator) NCollection_TListNode<TrackedValue>(std::in_place,&tail,7,concrete.get());
  std::cout<<"placement "<<concrete->events<<' '<<node->Value().value<<' '<<(node->Next()==&tail)<<'\n';
  concrete->events.clear();NCollection_TListNode<TrackedValue>::delNode(node,allocator);
  std::cout<<"delete "<<concrete->events<<' '<<(tail.Next()==nullptr)<<'\n';
  TrackedValue value(8,concrete.get());concrete->events.clear();
  node=new(allocator) NCollection_TListNode<TrackedValue>(value);
  std::cout<<"borrowed "<<concrete->events<<' '<<node->Value().value<<'\n';
  NCollection_TListNode<TrackedValue>::delNode(node,allocator);
  TrackedValue failing(9,concrete.get(),true);concrete->events.clear();
  try { node=new(allocator) NCollection_TListNode<TrackedValue>(failing); }
  catch(const std::runtime_error&) {}
  std::cout<<"failure "<<concrete->events<<'\n';
}
int main() { links();values();callbacks(); }
