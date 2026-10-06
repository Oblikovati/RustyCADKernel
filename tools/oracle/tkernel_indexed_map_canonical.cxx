// OCCT3d097a native IndexedMap topology, constructor/clear and removal callback phases.
// Probe-only -fno-access-control exposes protected state without changing method bodies.
#include <NCollection_IndexedMap.hxx>
#include <iostream>
#include <sstream>
#include <string>
struct IntPolicy {
 bool collision=false;
 size_t operator()(int key) const { return collision?0:static_cast<size_t>(key); }
 bool operator()(int first,int second) const { return first==second; }
};
using Probe=NCollection_IndexedMap<int,IntPolicy>;
void dump(const std::string& label,const Probe& map) {
 std::cout<<label<<' '<<map.Size()<<' '<<map.NbBuckets()<<' '<<(map.myData1!=nullptr)<<' '<<(map.myData2!=nullptr)<<" I";
 for(size_t i=0;i<map.Size();++i) {auto* n=static_cast<Probe::IndexedMapNode*>(map.myData2[i]);std::cout<<n->Key1()<<':'<<n->Index()<<',';}
 std::cout<<" B";
 if(map.myData1) for(size_t b=0;b<=map.NbBuckets();++b) {
  auto* n=static_cast<Probe::IndexedMapNode*>(map.myData1[b]);if(!n)continue;
  std::cout<<b<<'=';for(;n;n=static_cast<Probe::IndexedMapNode*>(n->Next()))std::cout<<n->Key1()<<',';std::cout<<';';
 }
 std::cout<<'\n';
}
void variants(const Probe& source,const std::string& prefix) {
 Probe resized(source);resized.ReSize(200);dump(prefix+"grow",resized);
 Probe cleared(source);cleared.Clear(false);dump(prefix+"keep",cleared);cleared.Clear(true);dump(prefix+"release",cleared);
 Probe added(source);added.Add(99);dump(prefix+"add",added);added.Add(99);dump(prefix+"hit",added);
 for(int i=1;i<=source.Extent();++i) {
  Probe removed(source);removed.RemoveFromIndex(i);dump(prefix+"remove"+std::to_string(i),removed);
  Probe replaced(source);replaced.Substitute(i,77);dump(prefix+"sub"+std::to_string(i),replaced);
  for(int j=1;j<=source.Extent();++j) {Probe swapped(source);swapped.Swap(i,j);dump(prefix+"swap"+std::to_string(i)+"-"+std::to_string(j),swapped);}
 }
}
void topology_rows() {
 for(bool collision:{false,true})for(size_t modulus:{0,1,10,101})for(int n=0;n<=4;++n) {
  Probe map(IntPolicy{collision},modulus);for(int k=1;k<=n;++k)map.Add(k);
  std::string prefix=std::to_string(collision)+"-"+std::to_string(modulus)+"-"+std::to_string(n)+"-";
  dump(prefix+"initial",map);variants(map,prefix);
 }
}
struct ObservedKey {int value;bool tracked=false;ObservedKey(int v):value(v){};ObservedKey(const ObservedKey& o):value(o.value),tracked(true){};~ObservedKey();};
struct KeyPolicy {size_t operator()(const ObservedKey& key)const{return key.value%2;}bool operator()(const ObservedKey&a,const ObservedKey&b)const{return a.value==b.value;}};
using ObservedMap=NCollection_IndexedMap<ObservedKey,KeyPolicy>;
ObservedMap* observed=nullptr;NCollection_BaseAllocator* old_allocator=nullptr;std::string drops;
ObservedKey::~ObservedKey() {
 if(!tracked||!observed)return;
 size_t indices=0,heads=0;for(size_t i=0;i<=observed->NbBuckets();++i) {indices+=observed->myData2[i]!=nullptr;heads+=observed->myData1[i]!=nullptr;}
 drops+=std::to_string(value)+","+std::to_string(observed->Size())+","+std::to_string(indices)+","+std::to_string(heads)+","+std::to_string(observed->Allocator().get()==old_allocator)+";";
}
void observer_rows() {
 for(int mode=0;mode<3;++mode) {
  ObservedMap map;observed=&map;old_allocator=map.Allocator().get();
  for(int i=1;i<=3;++i){ObservedKey key(i);map.Add(key);}drops.clear();
  if(mode==0)map.RemoveLast();else if(mode==1)map.Clear(false);else map.Clear(occ::handle<NCollection_BaseAllocator>(new NCollection_BaseAllocator));
  std::cout<<"observer"<<mode<<' '<<drops<<' '<<map.Size()<<' '<<(map.Allocator().get()==old_allocator)<<'\n';observed=nullptr;
 }
}
int main(){topology_rows();observer_rows();}
