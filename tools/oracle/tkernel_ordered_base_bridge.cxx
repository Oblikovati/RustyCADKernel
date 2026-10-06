// Pinned OCCT 3d097a03: inherited queries and generic cursors read actual ordered nodes.
#include <NCollection_OrderedMap.hxx>
#include <NCollection_OrderedDataMap.hxx>

#include <iostream>
#include <type_traits>
struct BridgeHash {
  size_t operator()(int key) const { return key%2; }
  bool operator()(int a,int b) const { return a==b; }
};
struct BridgeArena : NCollection_BaseAllocator {};
struct BridgeBase : NCollection_BaseMap {
  template<class C,class N,bool Key> using Bucket=BasicIterator<C,N,int,int,Key,true,false>;
};
using Set=NCollection_OrderedMap<int,BridgeHash>;
using Map=NCollection_OrderedDataMap<int,int,BridgeHash>;
template<class T> void queries(const char* label,const T& map) {
  const NCollection_BaseMap& base=map;
  std::cout<<label<<' '<<base.NbBuckets()<<' '<<base.Extent()<<' '<<base.Length()<<' '
    <<base.Size()<<' '<<base.IsEmpty()<<' '<<(base.Buckets()==nullptr)<<' '<<(map.FirstNode()==nullptr)<<'\n';
}
template<class I> void keys(const char* label,I iterator) {
  std::cout<<label<<' ';
  while(iterator.More()) { std::cout<<iterator.Key()<<':'<<*iterator<<',';iterator.Next(); }
  std::cout<<'\n';
}
void set_contracts() {
  occ::handle<NCollection_BaseAllocator> arena=new BridgeArena;
  Set set(0,arena);queries("set_empty",set);
  for(int key:{4,1,2,3,6,5})set.Add(key);
  queries("set_full",set);keys("set_order",Set::const_iterator(set));
  using Bucket=BridgeBase::Bucket<Set,Set::OrderedMapNode,true>;
  keys("set_buckets",Bucket(set));auto iterator=set.begin();
  std::cout<<"set_identity "<<(&iterator.Key()==&*iterator)<<' '
    <<(iterator.Node()==set.FirstNode())<<' '<<(set.Buckets()==static_cast<const Set&>(set).Buckets())
    <<' '<<(set.Allocator()==arena)<<' '<<std::is_same_v<decltype(*iterator),const int&><<'\n';
  auto* node=set.FirstNode();auto* buckets=set.Buckets();set.ReSize(200);
  std::cout<<"set_resize "<<(node==set.FirstNode())<<' '<<(buckets!=set.Buckets())<<'\n';
  buckets=set.Buckets();set.Clear(false);queries("set_retained",set);
  std::cout<<"set_zeroed "<<(buckets==set.Buckets())<<' ';
  bool empty=true;for(size_t i=0;i<=set.NbBuckets();++i)empty&=set.Buckets()[i]==nullptr;
  std::cout<<empty<<'\n';set.Clear(true);queries("set_released",set);
}
void data_contracts() {
  Map map(0);queries("data_empty",map);for(int key:{4,1,2,3,6,5})map.Bind(key,key*10);
  queries("data_full",map);keys("data_order",Map::const_iterator(map));
  using Bucket=BridgeBase::Bucket<Map,Map::OrderedDataMapNode,false>;
  keys("data_buckets",Bucket(map));
  for(auto iterator=map.begin();iterator.More();iterator.Next())*iterator+=iterator.Key();
  keys("data_mutated",Map::const_iterator(map));auto iterator=map.begin();
  std::cout<<"data_identity "<<(iterator.Node()==map.FirstNode())<<' '
    <<(map.Buckets()==static_cast<const Map&>(map).Buckets())<<' '
    <<std::is_same_v<decltype(*iterator),int&><<' '<<std::is_same_v<decltype(iterator.Key()),const int&><<'\n';
  auto* node=map.FirstNode();auto* buckets=map.Buckets();map.ReSize(200);
  std::cout<<"data_resize "<<(node==map.FirstNode())<<' '<<(buckets!=map.Buckets())<<'\n';
  map.Clear(false);queries("data_retained",map);map.Clear(true);queries("data_released",map);
}
int main() { set_contracts();data_contracts(); }
