// BRepGraph layers and caches (phase 2, area B): a graph script drives the layer
// registry, the history, supplement and deferred layers, a logging layer and a test
// cache service, then reports every layer. The Rust twin is
// crates/tkbrep/examples/br_probe/brep_graph_layers*.rs.
#include <BRepGraph_Cache.hxx>
#include <BRepGraph_CacheIterator.hxx>
#include <BRepGraph_CacheRegistry.hxx>
#include <BRepGraph_LayerDeferred.hxx>
#include <BRepGraph_LayerHistory.hxx>
#include <BRepGraph_LayerIterator.hxx>
#include <BRepGraph_LayerRegistry.hxx>
#include <BRepGraph_LayerTopoSupplement.hxx>
#include <BRepGraph_SupplementIterator.hxx>
#include <BRepGraph_UIDsView.hxx>
#include <BRepTools_History.hxx>
// A node, reference or item read as `kind index` (items as `domain kind index`,
// domain 0 node, 1 reference); UIDs as `kind counter`, item UIDs as `domain kind counter`.
BRepGraph_NodeId bl_node(BrReader& r){int k=r.integer();return BRepGraph_NodeId(BRepGraph_NodeId::Kind(k),uint32_t(r.integer()));}
BRepGraph_RefId bl_ref(BrReader& r){int k=r.integer();return BRepGraph_RefId(BRepGraph_RefId::Kind(k),uint32_t(r.integer()));}
BRepGraph_ItemId bl_item(BrReader& r){return r.integer()==0?BRepGraph_ItemId(bl_node(r)):BRepGraph_ItemId(bl_ref(r));}
BRepGraph_UID bl_uid(BrReader& r){int k=r.integer();return BRepGraph_UID(BRepGraph_NodeId::Kind(k),uint32_t(r.integer()));}
BRepGraph_ItemUID bl_item_uid(BrReader& r){int d=r.integer();int k=r.integer();size_t c=size_t(r.integer());
 return d==0?BRepGraph_ItemUID::Node(BRepGraph_NodeId::Kind(k),c):BRepGraph_ItemUID::Reference(BRepGraph_RefId::Kind(k),c);}
template<class T,class F> NCollection_LinearVector<T> bl_list(BrReader& r,F read){NCollection_LinearVector<T> v;int n=r.integer();for(int i=0;i<n;++i)v.Append(read(r));return v;}
TCollection_AsciiString bl_label(BrReader& r){return TCollection_AsciiString(r.bytes().c_str());}
void bl_print_uid(const BRepGraph_UID& u){br_int(int(u.Kind));br_int(u.Counter);}
void bl_print_item_uid(const BRepGraph_ItemUID& u){br_int(int(u.ItemDomain()));br_int(u.RawKind());br_int(u.Counter());}
template<class V> void bl_nodes(const V& v){br_int(v.Size());for(const auto& n:v)bg_node(n);}
template<class V> void bl_uids(const V& v){br_int(v.Size());for(const auto& u:v)bl_print_uid(u);}
template<class V> void bl_item_uids(const V& v){br_int(v.Size());for(const auto& u:v)bl_print_item_uid(u);}
void bl_text(const TCollection_AsciiString& t){br_text(std::string(t.ToCString()));}
void bl_guid(const Standard_GUID& g){char s[Standard_GUID_SIZE_ALLOC];g.ToCString(s);br_text(std::string(s));}
// A layer that logs its callbacks as integer codes (1 removed, 2 replaced, 3 modified,
// 4 nodes modified, 5 ref removed, 6 ref modified, 7 refs modified, 8 attached,
// 9 detached, 10 invalidate, 11 clear), with its attachment state.
class BlEventLayer : public BRepGraph_Layer {
public:
 BlEventLayer(int theIndex,int theKinds,int theRefKinds):myKinds(theKinds),myRefKinds(theRefKinds){
  char s[40];std::snprintf(s,sizeof(s),"e0000000-0000-0000-0000-%012d",theIndex);myId=Standard_GUID(s);myName=TCollection_AsciiString("Events")+theIndex;}
 const Standard_GUID& ID() const override{return myId;}
 const TCollection_AsciiString& Name() const override{return myName;}
 int SubscribedKinds() const override{return myKinds;}
 int SubscribedRefKinds() const override{return myRefKinds;}
 void OnNodeRemoved(const BRepGraph_NodeId n) noexcept override{log(1,n);}
 void OnNodeReplaced(const BRepGraph_NodeId o,const BRepGraph_NodeId n) noexcept override{log(2,o);node(n);}
 void OnNodeModified(const BRepGraph_NodeId n) noexcept override{log(3,n);}
 void OnNodesModified(const NCollection_Array1<BRepGraph_NodeId>& v) noexcept override{push(4);Events.push_back(v.Size());for(const auto& n:v)node(n);}
 void OnRefRemoved(const BRepGraph_RefId x) noexcept override{push(5);ref(x);}
 void OnRefModified(const BRepGraph_RefId x) noexcept override{push(6);ref(x);}
 void OnRefsModified(const NCollection_Array1<BRepGraph_RefId>& v) noexcept override{push(7);Events.push_back(v.Size());for(const auto& x:v)ref(x);}
 void CopyTo(const BRepGraph_CopyRemap&) const override{}
 void InvalidateAll() noexcept override{push(10);}
 void Clear() noexcept override{push(11);}
 std::vector<long long> Events;
protected:
 void OnAttached() noexcept override{push(8);}
 void OnDetached() noexcept override{push(9);}
private:
 void push(int c){Events.push_back(c);Events.push_back(IsAttached()?1:0);}
 void node(const BRepGraph_NodeId n){Events.push_back(int(n.NodeKind));Events.push_back(n.Index);}
 void ref(const BRepGraph_RefId x){Events.push_back(int(x.RefKind));Events.push_back(x.Index);}
 void log(int c,const BRepGraph_NodeId n){push(c);node(n);}
 Standard_GUID myId;TCollection_AsciiString myName;int myKinds,myRefKinds;
};
// A cache service that counts its clears and keeps bound entries (0 node OwnGen,
// 1 node SubtreeGen, 2 reference, 3 item) to report their freshness.
class BlCache : public BRepGraph_Cache {
public:
 static const Standard_GUID& GetID(){static const Standard_GUID THE_ID("c0000000-0000-0000-0000-000000000001");return THE_ID;}
 const Standard_GUID& ID() const override{return GetID();}
 const TCollection_AsciiString& Name() const override{static const TCollection_AsciiString THE_NAME("ProbeCache");return THE_NAME;}
 void Clear() noexcept override{++Clears;}
 bool Bind(int type,BrReader& r){
  if(type==2){RefEntry e;bool ok=e.BindOwnGen(*this,bl_ref(r));myRefs.push_back(e);return ok;}
  if(type==3){ItemEntry e;bool ok=e.BindOwnGen(*this,bl_item(r));myItems.push_back(e);return ok;}
  NodeEntry e;BRepGraph_NodeId n=bl_node(r);bool ok=type==0?e.BindOwnGen(*this,n):e.BindSubtreeGen(*this,n);myNodes.push_back(e);return ok;}
 void Report(const BRepGraph& g){
  br_int(Clears);br_flag(IsAttached());
  br_int(int(myNodes.size()));for(const auto& e:myNodes){br_flag(e.IsBound());br_flag(e.IsFresh(*this));bg_node(e.Node());}
  br_int(int(myRefs.size()));for(const auto& e:myRefs){br_flag(e.IsBound());br_flag(e.IsFresh(*this));bg_ref(e.Ref());}
  br_int(int(myItems.size()));for(const auto& e:myItems){br_flag(e.IsBound());br_flag(e.IsFresh(*this));br_int(int(e.Item().ItemDomain()));br_int(e.Item().RawKind());br_int(e.Item().Index());}
  for(int k=0;k<12;++k){BRepGraph_NodeId::Kind kind=BRepGraph_NodeId::Kind(k);if(!BRepGraph_NodeId::IsValidKind(kind))continue;
   for(uint32_t i=0;i<=g.Topo().Gen().Nb(kind);++i){uint32_t own=0,sub=0;BRepGraph_NodeId n(kind,i);
    br_flag(NodeOwnGen(n,own));br_int(own);br_flag(NodeSubtreeGen(n,sub));br_int(sub);}}
  // References in range only: RefOwnGen reads the removal plane unchecked.
  for(int k=0;k<7;++k){BRepGraph_RefId::Kind kind=BRepGraph_RefId::Kind(k);
   for(uint32_t i=0;i<g.Refs().Gen().Nb(kind);++i){uint32_t gen=0;BRepGraph_NodeId child;BRepGraph_RefId x(kind,i);
    br_flag(RefOwnGen(x,gen));br_int(gen);br_flag(ItemOwnGen(BRepGraph_ItemId(x),gen));br_flag(ResolveActiveRefChild(x,child));bg_node(child);}}}
 int Clears=0;
private:
 std::vector<NodeEntry> myNodes;std::vector<RefEntry> myRefs;std::vector<ItemEntry> myItems;
};
struct BlState{std::vector<occ::handle<BlEventLayer>> events;std::vector<occ::handle<BlCache>> caches;occ::handle<BRepGraph_LayerDeferred> deferred=new BRepGraph_LayerDeferred();};
occ::handle<BRepTools_History> bl_history_ops(const BrScene& s,BrReader& r){
 occ::handle<BRepTools_History> h=new BRepTools_History();int n=r.integer();
 for(int i=0;i<n;++i){int op=r.integer();TopoDS_Shape in=s.graph.shape(r.integer());TopoDS_Shape out=s.graph.shape(r.integer());
  try{if(op==0)h->AddModified(in,out);else if(op==1)h->AddGenerated(in,out);else h->Remove(in);br_word("-");}
  catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
 return h;}
NCollection_DataMap<TopoDS_Shape,BRepGraph_NodeId,TopTools_ShapeMapHasher> bl_inputs(const BRepGraph& g,BrReader& r){
 NCollection_DataMap<TopoDS_Shape,BRepGraph_NodeId,TopTools_ShapeMapHasher> m;g.Shapes().CollectHistoryInputs(bl_list<BRepGraph_NodeId>(r,bl_node).ToArray1(),m);return m;}
#include "tkbrep_probe_brepgraph_layers_actions.cxx"
#include "tkbrep_probe_brepgraph_layers_report.cxx"
