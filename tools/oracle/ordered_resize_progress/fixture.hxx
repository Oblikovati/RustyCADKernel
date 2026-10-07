#include <NCollection_OrderedMap.hxx>
#include <NCollection_OrderedDataMap.hxx>
#include <NCollection_AccAllocator.hxx>
#include <array>
#include <iostream>
#include <set>
#include <sstream>
#include <stdexcept>
#include <vector>
struct ResizeArena:NCollection_AccAllocator {
 ResizeArena():NCollection_AccAllocator(4096){}
 int Active()const{int count=0;for(auto it=myBlocks.cbegin();it!=myBlocks.cend();++it)count+=it->allocCount;return count;}
};
struct ResizeTrace {std::array<size_t,3> hashes;int reject=0,calls=0;bool active=false;std::string events;};
struct ResizeKey {
 int id;ResizeTrace* trace;
 ~ResizeKey(){if(trace->active)trace->events+="D"+std::to_string(id)+",";}
};
struct ResizePolicy {
 size_t operator()(const ResizeKey& key)const {
  auto& trace=*key.trace;
  if(trace.active){trace.events+="H"+std::to_string(key.id)+",";if(++trace.calls==trace.reject)throw std::runtime_error("rejected hash callback");}
  return trace.hashes[key.id-1];
 }
 bool operator()(const ResizeKey& left,const ResizeKey& right)const{return left.id==right.id;}
};
using ResizeSet=NCollection_OrderedMap<ResizeKey,ResizePolicy>;
using ResizeMap=NCollection_OrderedDataMap<ResizeKey,int,ResizePolicy>;
void insert(ResizeSet& map,ResizeTrace& trace,int id){map.Add(ResizeKey{id,&trace});}
void insert(ResizeMap& map,ResizeTrace& trace,int id){map.Bind(ResizeKey{id,&trace},id*10);}
template<class Node>std::string graph(NCollection_ListNode* const* buckets,size_t count,std::set<const void*>& reached,bool& unique) {
 std::ostringstream out;
 for(size_t i=0;i<=count;++i){
  if(!buckets[i])continue;out<<i<<":";auto* node=buckets[i];std::set<const void*> local;
  while(node){if(!local.insert(node).second){out<<"cycle";unique=false;break;}unique=reached.insert(node).second&&unique;out<<static_cast<const Node*>(node)->Key().id<<",";node=node->Next();}
  out<<";";
 }
 return out.str().empty()?"-":out.str();
}
template<class Map>auto nodes(Map& map) {
 using Node=std::remove_pointer_t<decltype(map.FirstNode())>;
 std::vector<Node*> result;for(auto* node=map.FirstNode();node;node=node->OrderNext())result.push_back(node);return result;
}
template<class Map>std::string links(Map& map) {
 std::ostringstream out;
 for(auto* node=map.FirstNode();node;node=node->OrderNext())out<<node->Key().id<<":"<<(node->Next()?static_cast<decltype(node)>(node->Next())->Key().id:0)<<",";
 return out.str();
}
