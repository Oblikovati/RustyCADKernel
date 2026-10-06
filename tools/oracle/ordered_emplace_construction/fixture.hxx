#include <NCollection_OrderedMap.hxx>
#include <NCollection_AccAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct EmplaceArena:NCollection_AccAllocator {
 EmplaceArena():NCollection_AccAllocator(8192){}
 int Active()const{int count=0;for(auto it=myBlocks.cbegin();it!=myBlocks.cend();++it)count+=it->allocCount;return count;}
};
struct EmplaceTrace {
 EmplaceArena* arena;std::string events;bool enabled=false,rejectConstruct=false,rejectHash=false,rejectEqual=false;
 void note(const std::string& event){if(enabled)events+=event+",";}
 int construct(int id){note("C"+std::to_string(id)+"a"+std::to_string(arena->Active()));if(rejectConstruct)throw std::runtime_error("constructor");return id;}
};
struct EmplaceArgument {
 int id;EmplaceTrace* trace;
 operator int()const{return trace->construct(id);}
};
struct EmplaceKey {
 int id;EmplaceTrace* trace;
 EmplaceKey(int value,EmplaceTrace& log):id(value),trace(&log){}
 explicit EmplaceKey(const EmplaceArgument& arg):id(arg.trace->construct(arg.id)),trace(arg.trace){}
 EmplaceKey(EmplaceKey&& other):id(other.id),trace(other.trace){trace->note("M"+std::to_string(id));other.id=-id;}
 ~EmplaceKey(){trace->note("D"+std::to_string(id));}
};
int keyId(int key){return key;}
int keyId(const EmplaceKey& key){return key.id;}
struct EmplacePolicy {
 EmplaceTrace* trace;
 template<class K>size_t operator()(const K& key)const {
  int id=keyId(key);trace->note("h"+std::to_string(id));
  if(trace->rejectHash&&id==999)throw std::runtime_error("hash");return size_t(id);
 }
 template<class K>bool operator()(const K& left,const K& right)const {
  trace->note("e"+std::to_string(keyId(left))+":"+std::to_string(keyId(right)));
  if(trace->rejectEqual)throw std::runtime_error("equal");return keyId(left)==keyId(right);
 }
};
using IntegerSet=NCollection_OrderedMap<int,EmplacePolicy>;
using TrackedSet=NCollection_OrderedMap<EmplaceKey,EmplacePolicy>;
const char* names[]={"emplace","emplaced","try_emplace","try_emplaced"};
template<class Map>std::string invoke(Map& map,int operation,const EmplaceArgument& arg) {
 try {
  switch(operation){
   case 0:return "b"+std::to_string(map.Emplace(arg));
   case 1:return "r"+std::to_string(keyId(map.Emplaced(arg)));
   case 2:return "b"+std::to_string(map.TryEmplace(arg));
   default:return "r"+std::to_string(keyId(map.TryEmplaced(arg)));
  }
 }catch(const std::runtime_error&){return "throw";}
}
struct EmplaceCase {const char* label;int count,key;bool constructor,hash,equal;};
EmplaceCase scenarios[]={
 {"empty_ok",0,7,false,false,false},{"empty_construct",0,7,true,false,false},{"empty_hash",0,999,false,true,false},
 {"insert",3,999,false,false,false},{"duplicate",3,7,false,false,false},{"construct",3,7,true,false,false},
 {"hash",3,999,false,true,false},{"equal",3,7,false,false,true},
 {"grow",102,999,false,false,false},{"grow_construct",102,999,true,false,false}
};
void populate(IntegerSet& map,int count){if(count==3){for(int key:{4,1,7})map.Add(key);return;}for(int key=0;key<count;++key)map.Add(key);}
template<class Map>std::string order(const Map& map) {
 std::string result;for(const auto& key:map)result+=std::to_string(keyId(key))+",";return result.empty()?"-":result;
}
template<class Map>void snapshot(const std::string& label,const std::string& result,const Map& map,const EmplaceTrace& trace,bool sameFirst) {
 std::cout<<label<<' '<<result<<' '<<map.Size()<<' '<<map.NbBuckets()<<' '<<(map.Buckets()!=nullptr)<<' '<<trace.arena->Active()<<' '<<sameFirst<<' '<<trace.events<<' '<<order(map)<<'\n';
}
void configure(EmplaceTrace& trace,const EmplaceCase& test){trace.enabled=true;trace.rejectConstruct=test.constructor;trace.rejectHash=test.hash;trace.rejectEqual=test.equal;}
void runInteger(int operation,const EmplaceCase& test) {
 occ::handle<EmplaceArena> arena=new EmplaceArena;EmplaceTrace trace{arena.get()};IntegerSet map(EmplacePolicy{&trace},size_t(0),arena);
 populate(map,test.count);const auto* first=map.FirstNode();configure(trace,test);
 std::string result=invoke(map,operation,EmplaceArgument{test.key,&trace});
 snapshot(std::string(names[operation])+"_"+test.label,result,map,trace,map.FirstNode()==first);
}
void runTracked(int operation,const EmplaceCase& test) {
 occ::handle<EmplaceArena> arena=new EmplaceArena;EmplaceTrace trace{arena.get()};TrackedSet map(EmplacePolicy{&trace},size_t(0),arena);
 map.Add(EmplaceKey(7,trace));const auto* first=map.FirstNode();configure(trace,test);
 std::string result=invoke(map,operation,EmplaceArgument{test.key,&trace});
 snapshot(std::string(names[operation])+"_tracked_"+test.label,result,map,trace,map.FirstNode()==first);
}
