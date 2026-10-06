// OCCT 3d097a: real DataMapNode inheritance, constructor order and allocator callback lifetime.
#include <NCollection_DataMap.hxx>
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
using Node = NCollection_DataMap<NodeItem,NodeItem>::DataMapNode;
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
  using IntNode = NCollection_DataMap<int,int>::DataMapNode;
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
int main() { constructors(); emplacement(); links(); allocated_nodes(); }
