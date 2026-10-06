// OCCT 3d097a: real DoubleMapNode inheritance, constructor order and allocator callback lifetime.
#include <NCollection_DoubleMap.hxx>
#include <iostream>
#include <cstdint>
#include <cstdlib>
#include <type_traits>
#include <stdexcept>
#include <string>
#include "tkernel_native_map_node_trace.hxx"
using Node = NCollection_DoubleMap<NodeItem,NodeItem>::DoubleMapNode;
void constructors() {
  NodeItem key('K'), value('V');
  NodeItem::events.clear(); { Node node(key,value,nullptr,nullptr); }
  std::cout << "borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(key,NodeItem('V'),nullptr,nullptr); }
  std::cout << "key_borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(NodeItem('K'),value,nullptr,nullptr); }
  std::cout << "value_borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(NodeItem('K'),NodeItem('V'),nullptr,nullptr); }
  std::cout << "owned " << NodeItem::events << '\n';
}
void emplacement() {
  NodeItem key('K');
  NodeItem::events.clear(); { Node node(key,std::in_place,nullptr,nullptr,'V'); }
  std::cout << "emplace_borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(NodeItem('K'),std::in_place,nullptr,nullptr,'V'); }
  std::cout << "emplace_owned " << NodeItem::events << '\n';
}
void links() {
  using IntNode = NCollection_DoubleMap<int,int>::DoubleMapNode;
  NCollection_ListNode tail(nullptr);
  IntNode node(1,7,&tail,&tail);
  auto* prefix = static_cast<NCollection_ListNode*>(&node);
  std::cout << "prefix " << (static_cast<void*>(prefix)==static_cast<void*>(&node)) << ' '
            << (prefix->Next()==&tail) << ' ' << node.Key1() << ' ' << node.Value() << ' ';
  node.ChangeValue()=8; node.Next()=nullptr;
  std::cout << node.Value() << ' ' << (prefix->Next()==nullptr) << '\n';
}
void failed_copy(const occ::handle<NodeArena>& concrete,
                 occ::handle<NCollection_BaseAllocator>& arena,
                 NodeItem& key, NodeItem& value, char reject, const char* label) {
  NodeItem::reject=reject; NodeItem::events.clear();
  try { new(arena) Node(key,value,nullptr,nullptr); } catch(const std::runtime_error&) {}
  std::cout << label << ' ' << NodeItem::events << ' ' << concrete->active << '\n';
}
void allocated_nodes() {
  occ::handle<NodeArena> concrete = new NodeArena;
  occ::handle<NCollection_BaseAllocator> arena = concrete;
  NodeItem::arena=concrete.get();
  NodeItem key('K'), value('V'); NCollection_ListNode tail(nullptr);
  NodeItem::events.clear();
  Node* node=new(arena) Node(key,value,&tail,&tail);
  std::cout << "allocation " << NodeItem::events << ' ' << NodeItem::value_live << ' '
            << NodeItem::key_live << ' ' << concrete->active << '\n';
  NodeItem::events.clear(); Node::delNode(node,arena);
  std::cout << "deletion " << NodeItem::events << ' ' << concrete->active << ' ' << (tail.Next()==nullptr) << '\n';
  failed_copy(concrete,arena,key,value,'K',"key_failure");
  failed_copy(concrete,arena,key,value,'V',"value_failure");
  NodeItem::reject=0; NodeItem::arena=nullptr;
}
struct alignas(32) DoubleWide { unsigned char value = 7; };
struct DoubleNodeArena : NCollection_BaseAllocator {
  size_t bytes=0;
  void* Allocate(size_t amount) override { bytes=amount;return std::aligned_alloc(32,(amount+31)&~size_t(31)); }
  void Free(void* pointer) override { std::free(pointer); }
};
template<class K1,class K2> void double_layout(const char* label) {
  using Node=typename NCollection_DoubleMap<K1,K2>::DoubleMapNode;
  occ::handle<DoubleNodeArena> concrete=new DoubleNodeArena;
  occ::handle<NCollection_BaseAllocator> arena=concrete;
  Node* node=new(arena) Node(K1{},K2{},nullptr,nullptr);
  const auto start=reinterpret_cast<std::uintptr_t>(node);
  std::cout<<label<<' '<<sizeof(Node)<<' '<<alignof(Node)<<' '
    <<reinterpret_cast<std::uintptr_t>(&node->Key1())-start<<' '
    <<reinterpret_cast<std::uintptr_t>(&node->Key2())-start<<' '
    <<reinterpret_cast<std::uintptr_t>(&node->Next2())-start<<' '
    <<concrete->bytes<<'\n';
  Node::delNode(node,arena);
}
void double_node_links() {
  using Node=NCollection_DoubleMap<int,int>::DoubleMapNode;
  Node first(2,20,nullptr,nullptr),second(3,30,nullptr,nullptr);
  Node node(1,10,&first,&second);const Node& readable=node;
  static_assert(std::is_same_v<decltype(readable.Key1()),const int&>);
  static_assert(std::is_same_v<decltype(node.Key1()),int&>);
  static_assert(std::is_same_v<decltype(node.Next2()),Node*&>);
  std::cout<<"links "<<(node.Next()==&first)<<' '<<(node.Next2()==&second)<<' '
    <<(&node.Key2()==&node.Value())<<' ';
  node.Key1()=11;node.ChangeValue()=12;node.Next2()=&first;
  std::cout<<node.Key1()<<' '<<node.Key2()<<' '<<(node.Next2()==&first)<<'\n';
}
void double_iterator_key() {
  NCollection_DoubleMap<int,int> map;map.Bind(1,10);
  auto iterator=map.begin();
  std::cout<<"generic_key "<<iterator.Key()<<' '<<*iterator<<' '
    <<(&iterator.Key()==iterator.operator->())<<'\n';
}
int main() {
 constructors(); emplacement(); links(); allocated_nodes();
 double_layout<int,int>("int_int");double_layout<unsigned char,unsigned char>("u8_u8");
 double_layout<int,unsigned char>("int_u8");double_layout<unsigned char,int>("u8_int");
 double_layout<DoubleWide,unsigned char>("wide_u8");double_layout<unsigned char,DoubleWide>("u8_wide");
 double_layout<DoubleWide,DoubleWide>("wide_wide");double_node_links();double_iterator_key();
}
