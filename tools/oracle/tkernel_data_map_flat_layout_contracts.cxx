// OCCT 3d097a: measure real derived-node tail-padding reuse on the pinned Linux ABI.
#include <NCollection_DataMap.hxx>
#include <cstdint>
#include <cstdlib>
#include <iostream>
struct ThreeBytes { unsigned char bytes[3] = {}; };
struct alignas(32) Align32 { unsigned char byte = 0; };
struct alignas(64) Align64 { unsigned char byte = 0; };
// A named allocator observes the actual node allocation request, with valid 64-byte alignment.
struct FlatLayoutArena : NCollection_BaseAllocator {
  size_t requested = 0;
  void* Allocate(size_t bytes) override {
    requested = bytes;
    return std::aligned_alloc(64, (bytes + 63) & ~size_t(63));
  }
  void Free(void* address) override { std::free(address); }
};
template<class K, class V> size_t flat_allocation_request() {
  using Node = typename NCollection_DataMap<K,V>::DataMapNode;
  occ::handle<FlatLayoutArena> concrete = new FlatLayoutArena;
  occ::handle<NCollection_BaseAllocator> allocator = concrete;
  Node* node = new(allocator) Node(K{}, V{}, nullptr);
  const size_t requested = concrete->requested;
  Node::delNode(node, allocator);
  return requested;
}
template<class K, class V> void flat_node_layout(const char* label) {
  using Node = typename NCollection_DataMap<K,V>::DataMapNode;
  Node node(K{}, V{}, nullptr);
  const auto start = reinterpret_cast<std::uintptr_t>(&node);
  std::cout << label << ' ' << sizeof(Node) << ' ' << alignof(Node) << ' '
    << reinterpret_cast<std::uintptr_t>(&node.Key()) - start << ' '
    << reinterpret_cast<std::uintptr_t>(&node.Value()) - start << ' '
    << reinterpret_cast<std::uintptr_t>(static_cast<NCollection_ListNode*>(&node)) - start << ' '
    << sizeof(NCollection_TListNode<V>) << ' ' << flat_allocation_request<K,V>() << '\n';
}
int main() {
  flat_node_layout<int, int>("int_int");
  flat_node_layout<char, int>("char_int");
  flat_node_layout<int, char>("int_char");
  flat_node_layout<char, char>("char_char");
  flat_node_layout<double, int>("double_int");
  flat_node_layout<int, double>("int_double");
  flat_node_layout<long long, char>("long_char");
  flat_node_layout<char, long long>("char_long");
  flat_node_layout<char, ThreeBytes>("char_three");
  flat_node_layout<ThreeBytes, char>("three_char");
  flat_node_layout<ThreeBytes, ThreeBytes>("three_three");
  flat_node_layout<char, Align32>("char_align32");
  flat_node_layout<Align32, char>("align32_char");
  flat_node_layout<Align32, Align32>("align32_align32");
  flat_node_layout<Align64, Align32>("align64_align32");
  flat_node_layout<Align32, Align64>("align32_align64");
}
