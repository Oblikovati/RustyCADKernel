#pragma once
#include <NCollection_BaseAllocator.hxx>
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
