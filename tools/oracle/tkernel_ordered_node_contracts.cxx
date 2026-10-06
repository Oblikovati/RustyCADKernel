// OCCT 3d097a: real ordered node inheritance, constructor order and allocator callback lifetime.
#include <NCollection_OrderedDataMap.hxx>
#include <NCollection_OrderedMap.hxx>
#include <iostream>
#include <cstdint>
#include <cstdlib>
#include <type_traits>
#include <stdexcept>
#include <string>
#include "tkernel_native_map_node_trace.hxx"
using Node = NCollection_OrderedDataMap<NodeItem,NodeItem>::OrderedDataMapNode;
void constructors() {
  NodeItem key('K'), value('V');
  NodeItem::events.clear(); { Node node(key,value,nullptr); }
  std::cout << "borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(key,NodeItem('V'),nullptr); }
  std::cout << "key_borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(NodeItem('K'),value,nullptr); }
  std::cout << "value_borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(NodeItem('K'),NodeItem('V'),nullptr); }
  std::cout << "owned " << NodeItem::events << '\n';
}
void emplacement() {
  NodeItem key('K');
  NodeItem::events.clear(); { Node node(key,std::in_place,nullptr,'V'); }
  std::cout << "emplace_borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(NodeItem('K'),std::in_place,nullptr,'V'); }
  std::cout << "emplace_owned " << NodeItem::events << '\n';
}
void links() {
  using IntNode = NCollection_OrderedDataMap<int,int>::OrderedDataMapNode;
  NCollection_ListNode tail(nullptr);
  IntNode node(1,7,&tail);
  auto* prefix = static_cast<NCollection_ListNode*>(&node);
  std::cout << "prefix " << (static_cast<void*>(prefix)==static_cast<void*>(&node)) << ' '
            << (prefix->Next()==&tail) << ' ' << node.Key() << ' ' << node.Value() << ' ';
  node.ChangeValue()=8; node.Next()=nullptr;
  std::cout << node.Value() << ' ' << (prefix->Next()==nullptr) << '\n';
}
void failed_copy(const occ::handle<NodeArena>& concrete,
                 occ::handle<NCollection_BaseAllocator>& arena,
                 NodeItem& key, NodeItem& value, char reject, const char* label) {
  NodeItem::reject=reject; NodeItem::events.clear();
  try { new(arena) Node(key,value,nullptr); } catch(const std::runtime_error&) {}
  std::cout << label << ' ' << NodeItem::events << ' ' << concrete->active << '\n';
}
void allocated_nodes() {
  occ::handle<NodeArena> concrete = new NodeArena;
  occ::handle<NCollection_BaseAllocator> arena = concrete;
  NodeItem::arena=concrete.get();
  NodeItem key('K'), value('V'); NCollection_ListNode tail(nullptr);
  NodeItem::events.clear();
  Node* node=new(arena) Node(key,value,&tail);
  std::cout << "allocation " << NodeItem::events << ' ' << NodeItem::value_live << ' '
            << NodeItem::key_live << ' ' << concrete->active << '\n';
  NodeItem::events.clear(); Node::delNode(node,arena);
  std::cout << "deletion " << NodeItem::events << ' ' << concrete->active << ' ' << (tail.Next()==nullptr) << '\n';
  failed_copy(concrete,arena,key,value,'K',"key_failure");
  failed_copy(concrete,arena,key,value,'V',"value_failure");
  NodeItem::reject=0; NodeItem::arena=nullptr;
}
void set_constructors() {
  using SetNode = NCollection_OrderedMap<NodeItem>::OrderedMapNode;
  NodeItem key('K');
  NodeItem::events.clear(); { SetNode node(key,nullptr); }
  std::cout << "set_borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { SetNode node(NodeItem('K'),nullptr); }
  std::cout << "set_owned " << NodeItem::events << '\n';
  NodeItem::events.clear(); { SetNode node(std::in_place,nullptr,'K'); }
  std::cout << "set_emplaced " << NodeItem::events << '\n';
}
void set_allocation() {
  using SetNode = NCollection_OrderedMap<NodeItem>::OrderedMapNode;
  occ::handle<NodeArena> concrete = new NodeArena;
  occ::handle<NCollection_BaseAllocator> arena = concrete;
  NodeItem::arena = concrete.get(); NodeItem key('K');
  NodeItem::events.clear(); auto* node = new(arena) SetNode(key,nullptr);
  std::cout << "set_allocation " << NodeItem::events << ' ' << NodeItem::key_live << '\n';
  NodeItem::events.clear(); SetNode::delNode(node,arena);
  std::cout << "set_deletion " << NodeItem::events << ' ' << concrete->active << '\n';
  NodeItem::events.clear(); NodeItem::reject='K';
  try { new(arena) SetNode(key,nullptr); } catch(const std::runtime_error&) {}
  std::cout << "set_failure " << NodeItem::events << ' ' << concrete->active << '\n';
  NodeItem::reject=0;NodeItem::arena=nullptr;
}
struct alignas(32) OrderedWide { unsigned char value = 7; };
struct LayoutArena : NCollection_BaseAllocator {
  size_t bytes=0;
  void* Allocate(size_t amount) override { bytes=amount;return std::aligned_alloc(32,(amount+31)&~size_t(31)); }
  void Free(void* pointer) override { std::free(pointer); }
};
template<class N> void print_layout(const char* label, N* node, size_t bytes) {
  const auto start=reinterpret_cast<std::uintptr_t>(node);
  std::cout<<label<<' '<<sizeof(N)<<' '<<alignof(N)<<' '
    <<reinterpret_cast<std::uintptr_t>(&node->Key())-start<<' '
    <<reinterpret_cast<std::uintptr_t>(&node->Value())-start<<' '
    <<reinterpret_cast<std::uintptr_t>(&node->myOrderPrev)-start<<' '
    <<reinterpret_cast<std::uintptr_t>(&node->myOrderNext)-start<<' '<<bytes<<'\n';
}
template<class K,class V> void data_layout(const char* label) {
  using N=typename NCollection_OrderedDataMap<K,V>::OrderedDataMapNode;
  occ::handle<LayoutArena> concrete=new LayoutArena;
  occ::handle<NCollection_BaseAllocator> arena=concrete;
  N* node=new(arena) N(K{},V{},nullptr);
  print_layout(label,node,concrete->bytes);N::delNode(node,arena);
}
template<class K> void set_layout(const char* label) {
  using N=typename NCollection_OrderedMap<K>::OrderedMapNode;
  occ::handle<LayoutArena> concrete=new LayoutArena;
  occ::handle<NCollection_BaseAllocator> arena=concrete;
  N* node=new(arena) N(K{},nullptr);
  print_layout(label,node,concrete->bytes);N::delNode(node,arena);
}
void order_links() {
  using N=NCollection_OrderedMap<int>::OrderedMapNode;
  N first(1,nullptr),second(2,nullptr),hash(9,nullptr);
  first.Next()=&hash;first.myOrderNext=&second;second.myOrderPrev=&first;
  const N& read=first;
  NCollection_OrderedMap<int>::iterator cursor(&first);
  std::cout<<"set_links "<<(first.OrderNext()==&second)<<' '<<(read.OrderNext()==&second)
    <<' '<<(second.myOrderPrev==&first)<<' '<<*cursor<<' '<<*(++cursor)<<'\n';
  using D=NCollection_OrderedDataMap<int,int>::OrderedDataMapNode;
  D left(1,10,nullptr),right(2,20,nullptr),bucket(9,90,nullptr);
  left.Next()=&bucket;left.myOrderNext=&right;right.myOrderPrev=&left;
  NCollection_OrderedDataMap<int,int>::iterator value(&left);
  *value=11;const D& readable=left;
  std::cout<<"data_links "<<(left.OrderNext()==&right)<<' '<<(readable.OrderNext()==&right)
    <<' '<<(right.myOrderPrev==&left)<<' '<<*value<<' '<<*(++value)<<'\n';
}
void nonowning_links() {
  using N=NCollection_OrderedMap<NodeItem>::OrderedMapNode;
  occ::handle<NodeArena> concrete=new NodeArena;
  occ::handle<NCollection_BaseAllocator> arena=concrete;
  auto* first=new(arena) N(NodeItem('A'),nullptr);
  auto* second=new(arena) N(NodeItem('B'),first);
  first->myOrderNext=second;second->myOrderPrev=first;
  NodeItem::events.clear();N::delNode(first,arena);
  std::cout<<"nonowning "<<NodeItem::events<<' '<<concrete->active<<' ';
  N::delNode(second,arena);std::cout<<NodeItem::events<<' '<<concrete->active<<'\n';
}
int main() {
 constructors();emplacement();links();allocated_nodes();set_constructors();set_allocation();
 set_layout<int>("set_int");set_layout<unsigned char>("set_u8");set_layout<OrderedWide>("set_wide");
 data_layout<int,int>("data_int_int");data_layout<unsigned char,unsigned char>("data_u8_u8");
 data_layout<int,unsigned char>("data_int_u8");data_layout<unsigned char,int>("data_u8_int");
 data_layout<OrderedWide,unsigned char>("data_wide_u8");data_layout<unsigned char,OrderedWide>("data_u8_wide");
 data_layout<OrderedWide,OrderedWide>("data_wide_wide");order_links();nonowning_links();
}
