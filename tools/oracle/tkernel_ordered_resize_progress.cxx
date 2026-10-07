#include "ordered_resize_progress/allocation.hxx"
#include "ordered_resize_progress/fixture.hxx"
template<class Map,class Nodes>void finishResizeCase(Map& map,ResizeTrace& trace,occ::handle<ResizeArena>& arena,const Nodes& original,const std::vector<NCollection_ListNode*>& next,const std::set<const void*>& reached,bool unique,bool threw,const std::string& label) {
 using Node=std::remove_pointer_t<decltype(map.FirstNode())>;
 if(!unique){for(size_t i=0;i<original.size();++i)original[i]->Next()=next[i];}
 trace.events.clear();map.Clear(false);
 std::cout<<label<<"_clear "<<(unique?"defined":"repaired")<<' '<<arena->Active()<<' '<<allocation.freed<<' '<<trace.events<<'\n';
 trace.active=false;
 if(unique)for(auto* node:original)if(!reached.count(node))Node::delNode(node,arena);
 if(threw)Standard::Free(allocation.replacement);allocation.replacement=nullptr;
}
template<class Map>void resizeCase(const char* family,const char* pattern,std::array<size_t,3> hashes,int reject) {
 occ::handle<ResizeArena> arena=new ResizeArena;ResizeTrace trace{hashes};Map map(ResizePolicy(),size_t(101),arena);
 for(int id:{1,2,3})insert(map,trace,id);auto original=nodes(map);std::vector<NCollection_ListNode*> next;
 for(auto* node:original)next.push_back(node->Next());const auto* buckets=map.Buckets();
 allocation={true};trace.active=true;trace.reject=reject;bool threw=false;
 try{map.ReSize(size_t(1000));}catch(const std::runtime_error&){threw=true;}allocation.active=false;
 using Node=std::remove_pointer_t<decltype(map.FirstNode())>;
 std::set<const void*> reached,pending;bool unique=true,pendingUnique=true;
 auto oldGraph=graph<Node>(map.Buckets(),map.NbBuckets(),reached,unique);
 auto newGraph=graph<Node>(static_cast<NCollection_ListNode**>(allocation.replacement),1009,pending,pendingUnique);
 std::string label=std::string(family)+"_"+pattern+"_"+std::to_string(reject);
 std::cout<<label<<' '<<threw<<' '<<map.Size()<<' '<<map.NbBuckets()<<' '<<(buckets==map.Buckets())<<' '<<allocation.allocated<<' '<<allocation.freed<<' '<<allocation.bytes<<' '<<arena->Active()<<' '<<unique<<' '<<reached.size()<<' '<<links(map)<<' '<<oldGraph<<' '<<newGraph<<' '<<trace.events<<'\n';
 finishResizeCase(map,trace,arena,original,next,reached,unique,threw,label);
}
template<class Map>void resizeSuite(const char* family) {
 for(int reject=0;reject<=3;++reject){
  resizeCase<Map>(family,"same",{0,0,0},reject);
  resizeCase<Map>(family,"split",{0,101,202},reject);
  resizeCase<Map>(family,"merge",{0,1009,2018},reject);
  resizeCase<Map>(family,"mixed",{0,101,1009},reject);
 }
}
int main(){resizeSuite<ResizeSet>("set");resizeSuite<ResizeMap>("data");}
