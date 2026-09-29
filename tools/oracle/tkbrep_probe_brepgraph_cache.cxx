// BRepGraph lock, derived-state and mesh caches (phase 2, wave 2b): a graph script
// drives the lock layer, an attached deferred layer, the derived-state cache service,
// the mesh cache with a probe driver, the mesh view and the layer and cache copies into
// a second graph, then reports both graphs. The Rust twin is
// crates/tkbrep/examples/br_probe/brep_graph_cache*.rs.
#include <BRepGraph_CacheDerivedState.hxx>
#include <BRepGraph_CacheMesh.hxx>
#include <BRepGraph_LayerLock.hxx>
#include <BRepGraph_MeshView.hxx>
#include <BRepGraph_Tool.hxx>
// Owner GUID i: f0000000-0000-0000-0000-00000000000i.
Standard_GUID bc_guid(BrReader& r){char s[40];std::snprintf(s,sizeof(s),"f0000000-0000-0000-0000-%012d",r.integer());return Standard_GUID(s);}
// A driver that logs every Fill (slot, dirty faces, dirty free edges) and fills the
// dirty faces with one triangulation and the free edges with one 3D polygon (through
// the mesh editor for the default slot, through Change*Mesh and BindFresh otherwise).
class BcDriver : public BRepGraph_CacheMesh::Driver {
public:
 BcDriver(int theIndex,uint64_t theRecipe,const occ::handle<Poly_Triangulation>& theTri,
          const occ::handle<Poly_Polygon3D>& thePoly,bool theResult)
   :myRecipe(theRecipe),myTri(theTri),myPoly(thePoly),myResult(theResult){
  char s[40];std::snprintf(s,sizeof(s),"d0000000-0000-0000-0000-%012d",theIndex);myId=Standard_GUID(s);}
 const Standard_GUID& ID() const override{return myId;}
 uint64_t RecipeHash() const override{return myRecipe;}
 bool Fill(BRepGraph& g,BRepGraph_CacheMesh::SlotId slot,const BRepGraph_CacheMesh::DirtySet& d,const Message_ProgressRange&) override{
  Events.push_back(slot);Events.push_back(d.Faces.Size());for(const auto& f:d.Faces)Events.push_back(f.Index);
  Events.push_back(d.FreeEdges.Size());for(const auto& e:d.FreeEdges)Events.push_back(e.Index);
  occ::handle<BRepGraph_CacheMesh> c=g.CacheRegistry().Find<BRepGraph_CacheMesh>();
  for(const auto& f:d.Faces){
   if(slot==BRepGraph_CacheMesh::DefaultDisplaySlot)g.Mesh().Editor().Faces().SetCachedTriangulation(f,myTri);
   else{auto& e=c->ChangeFaceMesh(slot,f);e.Triangulation=myTri;c->BindFresh(e,f);}}
  for(const auto& x:d.FreeEdges){
   if(slot==BRepGraph_CacheMesh::DefaultDisplaySlot)g.Mesh().Editor().Edges().SetCachedPolygon3D(x,myPoly);
   else{auto& e=c->ChangeEdgeMesh(slot,x);e.Polygon3D=myPoly;c->BindFresh(e,x);}}
  return myResult;}
 std::vector<long long> Events;
private:
 Standard_GUID myId;uint64_t myRecipe;occ::handle<Poly_Triangulation> myTri;occ::handle<Poly_Polygon3D> myPoly;bool myResult;
};
struct BcState{std::vector<occ::handle<BcDriver>> drivers;};
// The graphs of a request: the scripted graph and the copy target.
struct BcGraphs{BRepGraph g;BRepGraph t;};
occ::handle<BRepGraph_LayerLock> bc_lock(BRepGraph& g){return g.LayerRegistry().Ensure<BRepGraph_LayerLock>();}
occ::handle<BRepGraph_CacheDerivedState> bc_derived(BRepGraph& g){return g.CacheRegistry().Ensure<BRepGraph_CacheDerivedState>();}
occ::handle<BRepGraph_CacheMesh> bc_mesh(BRepGraph& g){return g.CacheRegistry().Ensure<BRepGraph_CacheMesh>();}
// 61-73: the lock layer. 61 SetOwner(item, guid, revision) prints the bool, 62
// SetOwner(item, guid), 63 UnsetOwner(item), 64 UnsetOwner(item, guid), 65
// ReserveOwners(n), 66 TouchOwners, 67 DispatchOnNodeRemoved, 68 DispatchOnNodeReplaced,
// 69 DispatchOnRefRemoved, 70 registry InvalidateAll, 71 Clear, 72 ScopedOwnerEdit(item,
// guid) prints HasOwner inside and after the scope, 73 a detached lock layer:
// SetOwner(item, guid, true), FindOwnerId, HasOwner.
bool bc_lock_action(BRepGraph& g,BrReader& r,int code){
 if(code<61||code>73)return false;
 occ::handle<BRepGraph_LayerLock> l=bc_lock(g);
 switch(code){
  case 61:{BRepGraph_ItemId i=bl_item(r);Standard_GUID o=bc_guid(r);br_flag(l->SetOwner(i,o,r.flag()));break;}
  case 62:{BRepGraph_ItemId i=bl_item(r);l->SetOwner(i,bc_guid(r));break;}
  case 63:l->UnsetOwner(bl_item(r));break;
  case 64:{BRepGraph_ItemId i=bl_item(r);l->UnsetOwner(i,bc_guid(r));break;}
  case 65:l->ReserveOwners(size_t(r.integer()));break;
  case 66:l->TouchOwners();break;
  case 67:g.LayerRegistry().DispatchOnNodeRemoved(bl_node(r));break;
  case 68:{BRepGraph_NodeId o=bl_node(r);g.LayerRegistry().DispatchOnNodeReplaced(o,bl_node(r));break;}
  case 69:g.LayerRegistry().DispatchOnRefRemoved(bl_ref(r));break;
  case 70:g.LayerRegistry().InvalidateAll();break;
  case 71:l->Clear();break;
  case 72:{BRepGraph_ItemId i=bl_item(r);Standard_GUID o=bc_guid(r);
   {BRepGraph_LayerLock::ScopedOwnerEdit e(*l,i,o);br_flag(l->HasOwner(i));}br_flag(l->HasOwner(i));break;}
  default:{occ::handle<BRepGraph_LayerLock> d=new BRepGraph_LayerLock();BRepGraph_ItemId i=bl_item(r);Standard_GUID o=bc_guid(r);
   br_flag(d->SetOwner(i,o,true));Standard_GUID f;br_flag(d->FindOwnerId(i,f));bl_guid(f);br_flag(d->HasOwner(i));br_int(int(d->Revision()));break;}}
 br_int(int(l->Revision()));return true;}
// 74-79: the attached deferred layer. 74 RegisterDeferredRepresentations (item,
// provider, key, count, (kind role name index)...), 75 the Direct variant, 76
// UnregisterDeferred, 77 BeginBulkRegistration, 78 EndBulkRegistration, 79
// ReserveDeferredItems(n).
bool bc_deferred_action(BRepGraph& g,BrReader& r,int code){
 if(code<74||code>79)return false;
 occ::handle<BRepGraph_LayerDeferred> d=g.LayerRegistry().Ensure<BRepGraph_LayerDeferred>();
 switch(code){
  case 74:case 75:{BRepGraph_ItemId i=bl_item(r);TCollection_AsciiString p=bl_label(r),k=bl_label(r);auto v=bl_representations(r);
   if(code==74)d->RegisterDeferredRepresentations(i,p,k,v.IsEmpty()?nullptr:&v.First(),v.Size());
   else d->RegisterDeferredRepresentationsDirect(i,p,k,v.IsEmpty()?nullptr:&v.First(),v.Size());break;}
  case 76:d->UnregisterDeferred(bl_item(r));break;
  case 77:d->BeginBulkRegistration();break;
  case 78:d->EndBulkRegistration();break;
  default:d->ReserveDeferredItems(size_t(r.integer()));break;}
 br_int(int(d->Revision()));return true;}
// 80-89: the derived-state cache. 80 IsDegenerated and IsClosed of an edge, 81
// SameRange, 82 SameParameter, 83 GetWireIsClosed, 84 SetWireIsClosed(wire, flag), 85
// IsShellClosed, 86 Clear, 87 Tool::Wire::IsClosed, 88 Tool::Shell::IsClosed, 89
// Tool::Edge::Degenerated and IsClosed.
bool bc_derived_action(BRepGraph& g,BrReader& r,int code){
 if(code<80||code>89)return false;
 occ::handle<BRepGraph_CacheDerivedState> c=bc_derived(g);
 switch(code){
  case 80:{BRepGraph_EdgeId e(uint32_t(r.integer()));br_flag(c->IsDegenerated(e));br_flag(c->IsClosed(e));break;}
  case 81:br_flag(c->SameRange(BRepGraph_CoEdgeId(uint32_t(r.integer()))));break;
  case 82:br_flag(c->SameParameter(BRepGraph_CoEdgeId(uint32_t(r.integer()))));break;
  case 83:{bool closed=false;br_flag(c->GetWireIsClosed(BRepGraph_WireId(uint32_t(r.integer())),closed));br_flag(closed);break;}
  case 84:{BRepGraph_WireId w(uint32_t(r.integer()));c->SetWireIsClosed(w,r.flag());break;}
  case 85:br_flag(c->IsShellClosed(BRepGraph_ShellId(uint32_t(r.integer()))));break;
  case 86:c->Clear();break;
  case 87:br_flag(BRepGraph_Tool::Wire::IsClosed(g,BRepGraph_WireId(uint32_t(r.integer()))));break;
  case 88:br_flag(BRepGraph_Tool::Shell::IsClosed(g,BRepGraph_ShellId(uint32_t(r.integer()))));break;
  default:{BRepGraph_EdgeId e(uint32_t(r.integer()));br_flag(BRepGraph_Tool::Edge::Degenerated(g,e));br_flag(BRepGraph_Tool::Edge::IsClosed(g,e));break;}}
 br_word("-");return true;}
#include "tkbrep_probe_brepgraph_cache_mesh.cxx"
#include "tkbrep_probe_brepgraph_cache_report.cxx"
