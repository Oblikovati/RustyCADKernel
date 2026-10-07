#include <NCollection_OrderedMap.hxx>
#include <NCollection_OrderedDataMap.hxx>
#include <NCollection_AccAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct ClearArena:NCollection_AccAllocator {
 ClearArena():NCollection_AccAllocator(4096){}
 int Active()const{int total=0;for(auto it=myBlocks.cbegin();it!=myBlocks.cend();++it)total+=it->allocCount;return total;}
};
struct ClearTrace {
 NCollection_BaseAllocator* old;NCollection_BaseAllocator* next;int baseline;bool common,active=false;char reject='-';std::string events;
 int oldRefs()const{return old->GetRefCount()-(common?baseline:0);}
 int oldActive()const{return common?-1:static_cast<ClearArena*>(old)->Active();}
 void note(char role,int id){if(active)events+=role+std::to_string(id)+"o"+std::to_string(oldRefs())+"n"+std::to_string(next->GetRefCount())+"a"+std::to_string(oldActive())+",";}
 void rejectDrop(char role){if(active&&reject==role)throw std::runtime_error("rejected noexcept payload destructor");}
};
struct ClearPayload {
 int id;char role;ClearTrace* trace;
 ~ClearPayload(){trace->note(role,id);trace->rejectDrop(role);}
};
struct ClearPolicy {
 size_t operator()(const ClearPayload&)const{return 0;}
 bool operator()(const ClearPayload& left,const ClearPayload& right)const{return left.id==right.id;}
};
using ClearSet=NCollection_OrderedMap<ClearPayload,ClearPolicy>;
using ClearMap=NCollection_OrderedDataMap<ClearPayload,ClearPayload,ClearPolicy>;
void populate(ClearSet& map,ClearTrace& trace){for(int key:{4,1,3})map.Add(ClearPayload{key,'K',&trace});}
void populate(ClearMap& map,ClearTrace& trace){for(int key:{4,1,3})map.Bind(ClearPayload{key,'K',&trace},ClearPayload{key*10,'V',&trace});}
const char* clearModes[]={"same","other","null","self"};
const char* clearStates[]={"empty","filled","retained"};
template<class Map>void invokeClear(Map& map,int mode,const occ::handle<NCollection_BaseAllocator>& old,const occ::handle<NCollection_BaseAllocator>& next) {
 switch(mode){case 0:map.Clear(old);break;case 1:map.Clear(next);break;case 2:map.Clear(occ::handle<NCollection_BaseAllocator>());break;default:map.Clear(map.Allocator());}
}
template<class Map>void snapshot(const std::string& label,const Map& map,const ClearTrace& trace,bool sameBuckets) {
 const auto& selected=map.Allocator();bool common=selected==NCollection_BaseAllocator::CommonBaseAllocator();
 std::cout<<label<<' '<<map.Size()<<' '<<map.NbBuckets()<<' '<<(map.Buckets()!=nullptr)<<' '<<sameBuckets<<' '<<(selected.get()==trace.old)<<' '<<(selected.get()==trace.next)<<' '<<common<<' '<<trace.oldRefs()<<' '<<trace.next->GetRefCount()<<' '<<trace.oldActive()<<' '<<static_cast<ClearArena*>(trace.next)->Active()<<' '<<(trace.events.empty()?"-":trace.events)<<'\n';
}
template<class Map>void normalCase(const char* family,bool common,int mode,int state,bool owned=false) {
 occ::handle<NCollection_BaseAllocator> old=common?NCollection_BaseAllocator::CommonBaseAllocator():new ClearArena;
 occ::handle<NCollection_BaseAllocator> next=new ClearArena;ClearTrace trace{old.get(),next.get(),old->GetRefCount(),common};
 Map map(ClearPolicy(),size_t(13),old);if(state!=0)populate(map,trace);if(state==2)map.Clear(false);
 std::string label=std::string(family)+"_"+(owned?"owned":(common?"common":"custom"))+"_"+clearModes[mode]+"_"+clearStates[state];
 if(common)std::cerr<<label<<"_ambient "<<trace.baseline<<'\n';
 const auto* buckets=map.Buckets();trace.active=true;
 if(owned)map.Clear(occ::handle<NCollection_BaseAllocator>(mode==0?old:next));else invokeClear(map,mode,old,next);
 snapshot(label,map,trace,map.Buckets()==buckets);
}
