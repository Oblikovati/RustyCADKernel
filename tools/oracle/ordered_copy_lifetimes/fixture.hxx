#include <NCollection_OrderedMap.hxx>
#include <NCollection_OrderedDataMap.hxx>
#include <NCollection_AccAllocator.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <algorithm>
#include <stdexcept>
#include <string>
struct CopyArena:NCollection_AccAllocator {
 CopyArena():NCollection_AccAllocator(4096){}
 int Active() const {
  int count=0;
  for(auto it=myBlocks.cbegin();it!=myBlocks.cend();++it) count+=it->allocCount;
  return count;
 }
};
struct CopyTrace {
 NCollection_BaseAllocator* arena;std::string events;int rejectCopy=0,copies=0;bool rejectPolicy=false,rejectHash=false,rejectEqual=false,capture=false;void* retained=nullptr;
 void note(const std::string& event) {
  events+=event+"r"+std::to_string(arena->GetRefCount())+"a"+std::to_string(dynamic_cast<CopyArena*>(arena)?static_cast<CopyArena*>(arena)->Active():0)+",";
 }
};
struct CopyPayload {
 int id;CopyTrace* trace;
 CopyPayload(int value,CopyTrace& log):id(value),trace(&log){}
 CopyPayload(const CopyPayload& other):id(other.id),trace(other.trace) {
  trace->note("C"+std::to_string(id));
  if(++trace->copies==trace->rejectCopy)throw std::runtime_error("copy");
  if(id==4&&trace->capture){trace->retained=trace->arena->Allocate(0);trace->arena->Free(trace->retained);}
 }
 CopyPayload(CopyPayload&& other):id(other.id),trace(other.trace){other.id=-999;}
 CopyPayload& operator=(CopyPayload&& other){id=other.id;other.id=-999;return *this;}
 ~CopyPayload(){trace->note("D"+std::to_string(id));}
};
struct CopyPolicy {
 CopyTrace* trace;
 explicit CopyPolicy(CopyTrace& log):trace(&log){}
 CopyPolicy(CopyPolicy&& other):trace(other.trace){}
 CopyPolicy(const CopyPolicy& other):trace(other.trace) {
  trace->note("HC");if(trace->rejectPolicy)throw std::runtime_error("policy");
 }
 size_t operator()(const CopyPayload& key)const {
  trace->note("h"+std::to_string(key.id));
  if(trace->rejectHash&&key.id==1)throw std::runtime_error("hash");return 0;
 }
 bool operator()(const CopyPayload&a,const CopyPayload&b)const {
  trace->note("e"+std::to_string(a.id)+":"+std::to_string(b.id));
  if(trace->rejectEqual)throw std::runtime_error("equal");return a.id==b.id;
 }
 ~CopyPolicy(){trace->note("HD");}
};
using CopySet=NCollection_OrderedMap<CopyPayload,CopyPolicy>;
using CopyMap=NCollection_OrderedDataMap<CopyPayload,CopyPayload,CopyPolicy>;
void populate(CopySet& map,CopyTrace& trace){map.Add(CopyPayload(4,trace));map.Add(CopyPayload(1,trace));}
void populate(CopyMap& map,CopyTrace& trace){map.Bind(CopyPayload(4,trace),CopyPayload(40,trace));map.Bind(CopyPayload(1,trace),CopyPayload(10,trace));}
template<class Map>void runCopy(const char* prefix,const char* label,int rejected,bool policy,bool hash,bool equal,bool empty=false) {
 occ::handle<CopyArena> arena=new CopyArena;CopyTrace trace{arena.get()};
 {
  Map source(CopyPolicy(trace),size_t(101),arena);if(!empty)populate(source,trace);
  trace.events.clear();trace.rejectCopy=rejected;trace.rejectPolicy=policy;trace.rejectHash=hash;trace.rejectEqual=equal;
  std::cout<<prefix<<'_'<<label<<' ';
  try {Map copy(source);std::cout<<"ok "<<copy.Size()<<' '<<copy.NbBuckets()<<' ';}
  catch(const std::runtime_error&){std::cout<<"throw ";}
  std::cout<<arena->GetRefCount()<<' '<<arena->Active()<<' '<<trace.events<<'\n';
  trace.events.clear();trace.rejectCopy=0;trace.rejectPolicy=false;trace.rejectHash=false;trace.rejectEqual=false;
 }
 std::cout<<prefix<<'_'<<label<<"_source_drop "<<arena->GetRefCount()<<' '<<arena->Active()<<' '<<trace.events<<'\n';
}
template<class Map>void runCopies(const char* prefix,int payloadCount) {
 runCopy<Map>(prefix,"copy_ok",0,false,false,false);
 runCopy<Map>(prefix,"copy_empty",0,false,false,false,true);
 runCopy<Map>(prefix,"copy_policy",0,true,false,false);
 runCopy<Map>(prefix,"copy_hash",0,false,true,false);
 runCopy<Map>(prefix,"copy_equal",0,false,false,true);
 for(int fail=1;fail<=payloadCount;++fail)runCopy<Map>(prefix,("copy_payload"+std::to_string(fail)).c_str(),fail,false,false,false);
}

template<class Map,class Node>void reclaimCopy(const char* prefix) {
 occ::handle<CopyArena> arena=new CopyArena;CopyTrace trace{arena.get()};
 {
  Map source(CopyPolicy(trace),size_t(101),arena);populate(source,trace);
  trace.events.clear();trace.rejectHash=true;trace.capture=true;
  try{Map copy(source);}catch(const std::runtime_error&){}
  std::cout<<prefix<<"_reclaim_failure throw "<<arena->GetRefCount()<<' '<<arena->Active()<<' '<<trace.events<<'\n';
  trace.rejectHash=false;
 }
 trace.events.clear();
 {occ::handle<NCollection_BaseAllocator> owner=arena;Node::delNode(static_cast<NCollection_ListNode*>(trace.retained),owner);}
 std::cout<<prefix<<"_reclaimed "<<arena->GetRefCount()<<' '<<arena->Active()<<' '<<trace.events<<'\n';
}

template<class Map>void resetCopy(const char* prefix) {
 occ::handle<NCollection_IncAllocator> arena=new NCollection_IncAllocator(4096);
 CopyTrace trace{arena.get()};void* first=arena->Allocate(0);
 {
  Map source(CopyPolicy(trace),size_t(101),arena);populate(source,trace);
  trace.events.clear();trace.rejectHash=true;
  try{Map copy(source);}catch(const std::runtime_error&){}
  std::cout<<prefix<<"_reset_failure throw "<<arena->GetRefCount()<<" 0 "<<trace.events<<'\n';
  trace.events.clear();trace.rejectHash=false;
 }
 size_t drops=std::count(trace.events.begin(),trace.events.end(),'D')-1;
 trace.events.clear();arena->Reset(false);void* reused=arena->Allocate(0);
 std::cout<<prefix<<"_reset "<<arena->GetRefCount()<<' '<<(first==reused)<<' '<<drops<<' '<<trace.events.size()<<'\n';
}
