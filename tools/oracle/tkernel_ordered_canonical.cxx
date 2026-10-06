// Pinned OCCT 3d097a03: actual map allocations and authoritative native link topology.
#include <NCollection_OrderedMap.hxx>
#include <NCollection_AccAllocator.hxx>
#include <NCollection_OrderedDataMap.hxx>
#include <iostream>
#include <string>
#include <cstdint>
struct CanonicalArena : NCollection_BaseAllocator {
  size_t bytes=0;int active=0;
  void* Allocate(size_t amount) override { bytes=amount;++active;return NCollection_BaseAllocator::Allocate(amount); }
  void Free(void* pointer) override { --active;NCollection_BaseAllocator::Free(pointer); }
};
struct CanonicalHash {
  static std::string events;
  size_t operator()(int key) const { events+=std::to_string(key)+",";return key%2; }
  bool operator()(int left,int right) const { return left==right; }
};
std::string CanonicalHash::events;
using Set=NCollection_OrderedMap<int,CanonicalHash>;
using Map=NCollection_OrderedDataMap<int,int,CanonicalHash>;
template<class T> void graph(const char* label,T& map) {
  std::cout<<label<<' ';
  auto* node=map.FirstNode();
  decltype(node) previous=nullptr;
  bool links=true;
  while(node) {
    links &= node->myOrderPrev==previous;
    std::cout<<node->Key()<<':'<<(node->Next()?static_cast<decltype(node)>(node->Next())->Key():0)<<',';
    previous=node;node=node->OrderNext();
  }
  std::cout<<' '<<links<<'\n';
}
template<class T> void layout(const char* label,T& map,CanonicalArena& arena) {
  auto* node=map.FirstNode();auto start=reinterpret_cast<std::uintptr_t>(node);
  std::cout<<label<<' '<<sizeof(*node)<<' '<<arena.bytes<<' '
    <<reinterpret_cast<std::uintptr_t>(&node->Key())-start<<' '
    <<reinterpret_cast<std::uintptr_t>(&node->Value())-start<<' '<<arena.active<<'\n';
}
struct CanonicalAccArena : NCollection_AccAllocator {
  CanonicalAccArena():NCollection_AccAllocator(1024) {}
  size_t FreeBytes(void* pointer) { Key key;return findBlock(pointer,key)->FreeSize(); }
};
void reservations() {
  occ::handle<CanonicalAccArena> arena=new CanonicalAccArena;
  Set set(101,arena);set.Add(1);const auto before=arena->FreeBytes(set.FirstNode());
  set.Add(2);std::cout<<"set_reservation "<<before-arena->FreeBytes(set.FirstNode())<<'\n';
  Map map(101,arena);map.Bind(1,10);const auto prior=arena->FreeBytes(map.FirstNode());
  map.Bind(2,20);std::cout<<"data_reservation "<<prior-arena->FreeBytes(map.FirstNode())<<'\n';
}
int main() {
  reservations();
  occ::handle<CanonicalArena> arena=new CanonicalArena;
  Set set(101,arena);set.Add(4);layout("set_layout",set,*arena);
  for(int key:{1,2,3,6,5})set.Add(key);
  set.Remove(1);set.Add(9);graph("set_before",set);
  auto* old=set.FirstNode();CanonicalHash::events.clear();set.ReSize(200);
  std::cout<<"set_rehash "<<CanonicalHash::events<<' '<<(old==set.FirstNode())<<'\n';
  graph("set_after",set);set.Clear(false);std::cout<<"set_clear "<<arena->active<<' '<<set.NbBuckets()<<'\n';
  Map map(101,arena);map.Bind(4,40);layout("data_layout",map,*arena);
  for(int key:{1,2,3,6,5})map.Bind(key,key*10);
  map.UnBind(1);map.Bind(9,90);graph("data_before",map);
  auto* original=map.FirstNode();CanonicalHash::events.clear();map.ReSize(200);
  std::cout<<"data_rehash "<<CanonicalHash::events<<' '<<(original==map.FirstNode())<<'\n';
  graph("data_after",map);map.Clear(false);std::cout<<"data_clear "<<arena->active<<' '<<map.NbBuckets()<<'\n';
}
