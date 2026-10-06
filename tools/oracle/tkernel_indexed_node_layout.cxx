#include <NCollection_IndexedMap.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <NCollection_DataMap.hxx>
#include <iostream>
#include <cstdlib>
// Probe-only -fno-access-control exposes exact pinned nested types, not alternate layouts.
struct LayoutArena : NCollection_BaseAllocator {
 size_t requested=0;
 void* Allocate(size_t bytes) override { requested=bytes;return std::aligned_alloc(32,(bytes+31)&~size_t(31)); }
 void Free(void* address) override { std::free(address); }
};
template<class N,class... Args> size_t allocation_request(Args&&... args) {
 occ::handle<LayoutArena> concrete=new LayoutArena;
 occ::handle<NCollection_BaseAllocator> arena=concrete;
 N* node=new(arena) N(std::forward<Args>(args)...);
 const size_t requested=concrete->requested;
 N::delNode(node,arena);
 return requested;
}
struct alignas(32) Wide { unsigned char value=7; };
template<class K> void map_row(const char* label) {
 using N=typename NCollection_IndexedMap<K>::IndexedMapNode;
 N node(K{},1,nullptr); const char* start=reinterpret_cast<const char*>(&node);
 std::cout<<label<<' '<<sizeof(N)<<' '<<alignof(N)<<' '<<reinterpret_cast<const char*>(&node.Key1())-start<<' '<<reinterpret_cast<const char*>(&node.Index())-start<<' '<<allocation_request<N>(K{},1,nullptr)<<'\n';
}
template<class K,class V> void data_row(const char* label) {
 using N=typename NCollection_IndexedDataMap<K,V>::IndexedDataMapNode;
 N node(K{},1,V{},nullptr);const char* start=reinterpret_cast<const char*>(&node);
 std::cout<<label<<' '<<sizeof(N)<<' '<<alignof(N)<<' '<<reinterpret_cast<const char*>(&node.Key1())-start<<' '<<reinterpret_cast<const char*>(&node.Value())-start<<' '<<reinterpret_cast<const char*>(&node.Index())-start<<' '<<allocation_request<N>(K{},1,V{},nullptr)<<'\n';
}
int main() {
 map_row<unsigned char>("map_u8");map_row<int>("map_i32");map_row<Wide>("map_wide");
 data_row<unsigned char,unsigned char>("data_u8_u8");data_row<int,unsigned char>("data_i32_u8");data_row<unsigned char,int>("data_u8_i32");data_row<int,int>("data_i32_i32");
 data_row<Wide,unsigned char>("data_wide_u8");data_row<unsigned char,Wide>("data_u8_wide");data_row<Wide,Wide>("data_wide_wide");
}
