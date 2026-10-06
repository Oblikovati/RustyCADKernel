// OCCT 3d097a: node constructor/lifetime oracle; compile with -fno-access-control.
// This probe-only flag exposes the private/protected nested types without altering their bodies.
#include <NCollection_IndexedMap.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct NodeArena : NCollection_BaseAllocator {
  int active = 0;
  void* Allocate(size_t bytes) override { ++active; return NCollection_BaseAllocator::Allocate(bytes); }
  void Free(void* address) override { --active; NCollection_BaseAllocator::Free(address); }
};
struct NodeItem {
  char kind;
  static std::string events;
  static char reject;
  static NodeArena* arena;
  static bool value_live, key_live;
  explicit NodeItem(char k):kind(k) {}
  NodeItem(const NodeItem& other):kind(other.kind) {
    events += std::string("C") + kind + ",";
    if(arena) (kind=='V' ? value_live : key_live) = arena->active==1;
    if(kind==reject) throw std::runtime_error("copy rejected");
  }
  // A moved-from fixture owns no tracked resource; its destructor has no observable event.
  NodeItem(NodeItem&& other) noexcept:kind(other.kind) { other.kind='m'; }
  ~NodeItem() { if(kind!='m') events += std::string("D") + kind + ","; }
};
std::string NodeItem::events;
char NodeItem::reject = 0;
NodeArena* NodeItem::arena = nullptr;
bool NodeItem::value_live = false, NodeItem::key_live = false;
using Node = NCollection_IndexedDataMap<NodeItem,NodeItem>::IndexedDataMapNode;
void constructors() {
  NodeItem key('K'), value('V');
  NodeItem::events.clear(); { Node node(key,-7,value,nullptr); }
  std::cout << "borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(key,-7,NodeItem('V'),nullptr); }
  std::cout << "key_borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(NodeItem('K'),-7,value,nullptr); }
  std::cout << "value_borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(NodeItem('K'),-7,NodeItem('V'),nullptr); }
  std::cout << "owned " << NodeItem::events << '\n';
}
void emplacement() {
  NodeItem key('K');
  NodeItem::events.clear(); { Node node(key,-7,std::in_place,nullptr,'V'); }
  std::cout << "emplace_borrowed " << NodeItem::events << '\n';
  NodeItem::events.clear(); { Node node(NodeItem('K'),-7,std::in_place,nullptr,'V'); }
  std::cout << "emplace_owned " << NodeItem::events << '\n';
}
void links() {
 using IntNode=NCollection_IndexedDataMap<int,int>::IndexedDataMapNode;
 NCollection_ListNode tail(nullptr);IntNode node(1,-7,7,&tail);
 auto* prefix=static_cast<NCollection_ListNode*>(&node);
 std::cout<<"data_fields "<<(static_cast<void*>(prefix)==static_cast<void*>(&node))<<' '<<(prefix->Next()==&tail)<<' '<<node.Key1()<<' '<<node.Value()<<' '<<node.Index();
 node.Key1()=2;node.ChangeValue()=8;node.Index()=9;node.Next()=nullptr;
 std::cout<<' '<<node.Key1()<<' '<<node.Value()<<' '<<node.Index()<<' '<<(prefix->Next()==nullptr)<<'\n';
}
void failed_copy(const occ::handle<NodeArena>& concrete,
                 occ::handle<NCollection_BaseAllocator>& arena,
                 NodeItem& key, NodeItem& value, char reject, const char* label) {
  NodeItem::reject=reject; NodeItem::events.clear();
  try { new(arena) Node(key,-7,value,nullptr); } catch(const std::runtime_error&) {}
  std::cout << label << ' ' << NodeItem::events << ' ' << concrete->active << '\n';
}
void allocated_nodes() {
  occ::handle<NodeArena> concrete = new NodeArena;
  occ::handle<NCollection_BaseAllocator> arena = concrete;
  NodeItem::arena=concrete.get();
  NodeItem key('K'), value('V'); NCollection_ListNode tail(nullptr);
  NodeItem::events.clear();
  Node* node=new(arena) Node(key,-7,value,&tail);
  std::cout << "allocation " << NodeItem::events << ' ' << NodeItem::value_live << ' '
            << NodeItem::key_live << ' ' << concrete->active << '\n';
  NodeItem::events.clear(); Node::delNode(node,arena);
  std::cout << "deletion " << NodeItem::events << ' ' << concrete->active << ' ' << (tail.Next()==nullptr) << '\n';
  failed_copy(concrete,arena,key,value,'K',"key_failure");
  failed_copy(concrete,arena,key,value,'V',"value_failure");
  NodeItem::reject=0; NodeItem::arena=nullptr;
}


using KeyNode=NCollection_IndexedMap<NodeItem>::IndexedMapNode;
void map_constructors() {
 NodeItem key('K');NodeItem::events.clear();{KeyNode node(key,-7,nullptr);}
 std::cout<<"map_borrowed "<<NodeItem::events<<'\n';
 NodeItem::events.clear();{KeyNode node(NodeItem('K'),-7,nullptr);}
 std::cout<<"map_owned "<<NodeItem::events<<'\n';
 NodeItem::events.clear();{KeyNode node(std::in_place,-7,nullptr,'K');}
 std::cout<<"map_emplace "<<NodeItem::events<<'\n';
}
void map_links() {
 using IntNode=NCollection_IndexedMap<int>::IndexedMapNode;
 NCollection_ListNode tail(nullptr);IntNode node(7,-7,&tail);
 std::cout<<"map_fields "<<(static_cast<void*>(&node)==static_cast<void*>(static_cast<NCollection_ListNode*>(&node)))<<' '<<(node.Next()==&tail)<<' '<<(&node.Key1()==&node.Value())<<' '<<node.Index();
 node.Key1()=8;node.Index()=9;node.ChangeValue()=10;node.Next()=nullptr;
 std::cout<<' '<<node.Value()<<' '<<node.Index()<<' '<<(node.Next()==nullptr)<<'\n';
}
void map_allocation() {
 occ::handle<NodeArena> concrete=new NodeArena;occ::handle<NCollection_BaseAllocator> arena=concrete;
 NodeItem::arena=concrete.get();NodeItem key('K');NCollection_ListNode tail(nullptr);
 NodeItem::events.clear();KeyNode* node=new(arena) KeyNode(key,-7,&tail);
 std::cout<<"map_allocate "<<NodeItem::events<<' '<<NodeItem::key_live<<' '<<concrete->active<<'\n';
 NodeItem::events.clear();KeyNode::delNode(node,arena);
 std::cout<<"map_delete "<<NodeItem::events<<' '<<concrete->active<<' '<<(tail.Next()==nullptr)<<'\n';
 NodeItem::reject='K';NodeItem::events.clear();try{new(arena) KeyNode(key,-7,nullptr);}catch(const std::runtime_error&){}
 std::cout<<"map_copy_failure "<<NodeItem::events<<' '<<concrete->active<<'\n';
 NodeItem::reject=0;NodeItem::arena=nullptr;
}
int main(){constructors();emplacement();links();allocated_nodes();map_constructors();map_links();map_allocation();}
