#include <NCollection_OrderedMap.hxx>
#include <NCollection_OrderedDataMap.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <limits>
struct LookupTrace {
 std::string events;size_t code=0;bool rejectHash=false;int rejectEqual=0,equalities=0;
};
struct LookupPolicy {
 LookupTrace* trace;
 size_t operator()(int key)const {
  trace->events+="h"+std::to_string(key)+",";
  if(trace->rejectHash)throw std::runtime_error("hash");return trace->code;
 }
 bool operator()(int left,int right)const {
  trace->events+="e"+std::to_string(left)+":"+std::to_string(right)+",";
  if(++trace->equalities==trace->rejectEqual)throw std::runtime_error("equal");return left==right;
 }
};
template<class Base>struct Exposed:Base {
 using Base::Base;using Base::lookup;using Base::isEqual;using Base::hashCode;
};
using LookupSet=Exposed<NCollection_OrderedMap<int,LookupPolicy>>;
using LookupMap=Exposed<NCollection_OrderedDataMap<int,int,LookupPolicy>>;
void populate(LookupSet& map){for(int key:{4,1,7})map.Add(key);}
void populate(LookupMap& map){for(int key:{4,1,7})map.Bind(key,key*10);}
template<class Node>std::string nodeLabel(Node* node,Node* sentinel) {
 if(node==sentinel)return "s";if(!node)return "n";return std::to_string(node->Key());
}
struct LookupCase {const char* label;bool filled,hash;int key;bool rejectHash;int rejectEqual;};
LookupCase cases[]={
 {"empty",false,false,8,false,0},{"empty_hash",false,true,8,false,0},
 {"empty_hash_throw",false,true,8,true,0},
 {"first",true,false,7,false,0},{"first_hash",true,true,7,false,0},
 {"middle",true,false,1,false,0},{"middle_hash",true,true,1,false,0},
 {"last",true,false,4,false,0},{"last_hash",true,true,4,false,0},
 {"miss",true,false,8,false,0},{"miss_hash",true,true,8,false,0},
 {"hash_throw",true,false,8,true,0},{"hash_output_throw",true,true,8,true,0},
 {"equal1",true,false,8,false,1},{"equal1_hash",true,true,8,false,1},
 {"equal2",true,false,8,false,2},{"equal2_hash",true,true,8,false,2},
 {"equal3",true,false,8,false,3},{"equal3_hash",true,true,8,false,3}
};
template<class Map>void runLookup(const char* prefix,const LookupCase& test) {
 LookupTrace trace;Map map(LookupPolicy{&trace},size_t(101)),sentinels(LookupPolicy{&trace},size_t(101));
 populate(sentinels);if(test.filled)populate(map);
 auto* sentinel=sentinels.FirstNode();auto* node=sentinel;size_t hash=91;
 trace.events.clear();trace.equalities=0;trace.rejectHash=test.rejectHash;trace.rejectEqual=test.rejectEqual;
 std::string outcome;
 try{outcome=std::to_string(test.hash?map.lookup(test.key,node,hash):map.lookup(test.key,node));}
 catch(const std::runtime_error&){outcome="throw";}
 std::cout<<prefix<<'_'<<test.label<<' '<<outcome<<' '<<nodeLabel(node,sentinel)<<' '<<hash<<' '<<(trace.events.empty()?"-":trace.events)<<'\n';
}
template<class Map>void runHash(const char* prefix,size_t code,size_t upper) {
 LookupTrace trace;trace.code=code;Map map(LookupPolicy{&trace},size_t(101));
 size_t result=map.hashCode(-17,upper);
 std::cout<<prefix<<"_hash_"<<code<<'_'<<upper<<' '<<result<<' '<<trace.events<<'\n';
}
template<class Map>void runEqual(const char* prefix,int right,bool reject) {
 LookupTrace trace;trace.rejectEqual=reject?1:0;Map map(LookupPolicy{&trace},size_t(101));std::string result;
 try{result=std::to_string(map.isEqual(4,right));}catch(const std::runtime_error&){result="throw";}
 std::cout<<prefix<<"_equal_"<<right<<'_'<<reject<<' '<<result<<' '<<trace.events<<'\n';
}
template<class Map>void runSuite(const char* prefix) {
 for(const auto& test:cases)runLookup<Map>(prefix,test);
 for(size_t code:{size_t(0),size_t(1),std::numeric_limits<size_t>::max()})
  for(size_t upper:{size_t(1),size_t(13),std::numeric_limits<size_t>::max()})runHash<Map>(prefix,code,upper);
 runEqual<Map>(prefix,4,false);runEqual<Map>(prefix,8,false);runEqual<Map>(prefix,8,true);
}
