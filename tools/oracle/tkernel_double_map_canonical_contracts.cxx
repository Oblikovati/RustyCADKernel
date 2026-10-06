// Pinned OCCT 3d097a: canonical two-chain operation order and callback-visible metadata.
#include <NCollection_DoubleMap.hxx>
#include <iostream>
#include <string>
#include <functional>
static std::string hash_events;
template<int Side> struct DoubleHash {
 size_t operator()(int key) const {hash_events+="H"+std::to_string(Side)+":"+std::to_string(key)+",";return Side==1?size_t(key):0;}
 bool operator()(int first,int second) const {hash_events+="E"+std::to_string(Side)+":"+std::to_string(first)+":"+std::to_string(second)+",";return first==second;}
};
using IntDouble=NCollection_DoubleMap<int,int,DoubleHash<1>,DoubleHash<2>>;
static void hash_row(const char* label) {std::cout<<label<<' '<<hash_events<<'\n';hash_events.clear();}
static void hash_contracts() {
 IntDouble map;map.Bind(1,5);hash_row("insert_empty");
 map.Bind(2,6);hash_row("insert_nonempty");
 map.TryBind(1,99);hash_row("duplicate_first");
 map.TryBind(3,5);hash_row("duplicate_second");
 bool missing=map.AreBound(99,5);std::cout<<"are_missing "<<missing<<' ';hash_row("trace");
 bool quirk=map.AreBound(1,5);std::cout<<"are_primary_next_quirk "<<quirk<<' ';hash_row("trace");
 map.ReSize(200);std::cout<<"resize "<<map.NbBuckets()<<' ';hash_row("trace");
 map.UnBind1(2);hash_row("unbind_first");map.UnBind2(5);hash_row("unbind_second");
}
struct ObservationArena:NCollection_BaseAllocator {
 int active=0;
 void* Allocate(size_t amount) override {++active;return NCollection_BaseAllocator::Allocate(amount);}
 void Free(void* address) override {--active;NCollection_BaseAllocator::Free(address);}
};
struct ObservationKey {
 char kind;int value;bool active=true;
 static std::function<void(char,char)> observe;
 ObservationKey(char kind_,int value_):kind(kind_),value(value_){}
 ObservationKey(const ObservationKey& source):kind(source.kind),value(source.value) {if(observe)observe('C',kind);}
 ObservationKey(ObservationKey&& source):kind(source.kind),value(source.value){source.active=false;}
 ~ObservationKey(){if(active&&observe)observe('D',kind);}
};
std::function<void(char,char)> ObservationKey::observe;
struct ObservationHash {
 static std::function<void(char)> observe;
 size_t operator()(const ObservationKey& key) const {if(observe)observe(key.kind);return 0;}
 bool operator()(const ObservationKey& a,const ObservationKey& b) const{return a.value==b.value;}
};
std::function<void(char)> ObservationHash::observe;
struct ObservedDouble:NCollection_DoubleMap<ObservationKey,ObservationKey,ObservationHash,ObservationHash> {
 using NCollection_DoubleMap::NCollection_DoubleMap;
 const NCollection_ListNode* SecondHead()const{return myData2?myData2[1]:nullptr;}
};
static std::string snapshot(const ObservedDouble& map) {
 return std::to_string(map.Size())+":"+std::to_string(map.Buckets()&&map.Buckets()[1]!=nullptr)+":"+std::to_string(map.SecondHead()!=nullptr);
}
static void observation_contracts() {
 occ::handle<ObservationArena> concrete=new ObservationArena;occ::handle<NCollection_BaseAllocator> arena=concrete;
 ObservedDouble map(size_t(1),arena);ObservationKey key('K',1),value('V',5);std::string events;
 ObservationKey::observe=[&](char event,char kind){events+=std::string(1,event)+kind+"@"+std::to_string(concrete->active)+",";};
 map.Bind(key,value);std::cout<<"borrowed_allocation "<<events<<'\n';events.clear();
 ObservationKey::observe=[&](char event,char kind){if(event=='D')events+=std::string("D")+kind+"@"+snapshot(map)+",";};
 ObservationHash::observe=[&](char kind){events+=std::string("H")+kind+"@"+snapshot(map)+",";};
 map.UnBind1(key);std::cout<<"remove_observer "<<events<<"after="<<snapshot(map)<<'\n';
 ObservationHash::observe={};ObservationKey::observe={};map.Bind(key,value);events.clear();
 ObservationKey::observe=[&](char event,char kind){if(event=='D')events+=std::string("D")+kind+"@"+snapshot(map)+",";};
 map.Clear(false);std::cout<<"clear_observer "<<events<<"after="<<snapshot(map)<<'\n';ObservationKey::observe={};
 std::cout<<"retained_tables "<<(map.Buckets()!=nullptr)<<' '<<(map.SecondHead()==nullptr)<<' '<<map.NbBuckets()<<'\n';
}
static void assign_contracts() {
 occ::handle<ObservationArena> concrete=new ObservationArena;occ::handle<NCollection_BaseAllocator> arena=concrete;
 ObservedDouble source;ObservationKey key('K',1),value('V',5);source.Bind(key,value);
 ObservedDouble destination(size_t(1),arena);std::string events;
 ObservationKey::observe=[&](char event,char kind){events+=std::string(1,event)+kind+"@"+std::to_string(concrete->active)+",";};
 ObservationHash::observe=[&](char kind){events+=std::string("H")+kind+"@"+std::to_string(concrete->active)+",";};
 destination.Assign(source);std::cout<<"assign_order "<<events<<'\n';ObservationKey::observe={};ObservationHash::observe={};
}
static void mixed_binding_contracts() {
 occ::handle<ObservationArena> concrete=new ObservationArena;occ::handle<NCollection_BaseAllocator> arena=concrete;
 ObservedDouble map(size_t(1),arena);ObservationKey key('K',1),value('V',5);std::string events;
 ObservationKey::observe=[&](char event,char kind){if(event=='C')events+=std::string(1,event)+kind+"@"+std::to_string(concrete->active)+",";};
 map.Bind(key,ObservationKey('V',5));std::cout<<"mixed_first "<<events<<'\n';map.Clear();events.clear();
 map.Bind(ObservationKey('K',1),value);std::cout<<"mixed_second "<<events<<'\n';ObservationKey::observe={};
}
int main(){hash_contracts();observation_contracts();assign_contracts();mixed_binding_contracts();}
