// The reports of `br_bcache` (tkbrep_probe_brepgraph_cache.cxx), printed for the
// scripted graph and then the copy target. Geometry prints as scene registry ids.
// The registries: every layer's name and revision, every cache service's name.
void bc_registries(const BRepGraph& g){
 const BRepGraph_LayerRegistry& reg=g.LayerRegistry();br_int(reg.NbLayers());
 for(uint32_t i=0;i<reg.NbLayers();++i){occ::handle<BRepGraph_Layer> l=reg.Layer(i);bl_text(l->Name());br_int(l->Revision());}
 const BRepGraph_CacheRegistry& caches=g.CacheRegistry();br_int(caches.NbCaches());
 for(BRepGraph_CacheIterator it=caches.CacheIter();it.More();it.Next())bl_text(it.Value()->Name());}
// One owner query: HasOwner, then FindOwnerId (the exception, the flag and the GUID).
template<class Id> void bc_owner(const BRepGraph_LayerLock& l,const Id& x){
 br_flag(l.HasOwner(x));Standard_GUID o;bool found=false;BR_TRY(found=l.FindOwnerId(x,o));br_flag(found);if(found)bl_guid(o);}
// The lock layer: revision, HasOwners, then the owner of every node (one past each
// kind's end too) and of every reference.
void bc_lock_report(const BRepGraph& g){
 occ::handle<BRepGraph_LayerLock> l=g.LayerRegistry().Find<BRepGraph_LayerLock>();if(l.IsNull()){br_word("N");return;}
 br_int(int(l->Revision()));br_flag(l->HasOwners());
 for(int k=0;k<12;++k){BRepGraph_NodeId::Kind kind=BRepGraph_NodeId::Kind(k);if(!BRepGraph_NodeId::IsValidKind(kind))continue;
  for(uint32_t i=0;i<=g.Topo().Gen().Nb(kind);++i)bc_owner(*l,BRepGraph_NodeId(kind,i));}
 for(int k=0;k<7;++k){BRepGraph_RefId::Kind kind=BRepGraph_RefId::Kind(k);
  for(uint32_t i=0;i<g.Refs().Gen().Nb(kind);++i)bc_owner(*l,BRepGraph_RefId(kind,i));}}
// The attached deferred layer: revision and entries in map order.
void bc_deferred_report(const BRepGraph& g){
 occ::handle<BRepGraph_LayerDeferred> d=g.LayerRegistry().Find<BRepGraph_LayerDeferred>();if(d.IsNull()){br_word("N");return;}
 br_int(int(d->Revision()));br_flag(d->HasDeferredItems());
 d->ForEachDeferred([](const BRepGraph_ItemId i,const BRepGraph_LayerDeferred::Entry& e){bl_deferred_entry(i,e);return true;});br_word("|");}
// The derived-state service: every edge, coedge, wire and shell query (one past the end
// too), answered from the entries the actions left or computed now.
void bc_derived_report(const BRepGraph& g){
 occ::handle<BRepGraph_CacheDerivedState> c=g.CacheRegistry().Find<BRepGraph_CacheDerivedState>();if(c.IsNull()){br_word("N");return;}
 for(uint32_t i=0;i<=g.Topo().Edges().Nb();++i){br_flag(c->IsDegenerated(BRepGraph_EdgeId(i)));br_flag(c->IsClosed(BRepGraph_EdgeId(i)));}
 for(uint32_t i=0;i<=g.Topo().CoEdges().Nb();++i){br_flag(c->SameRange(BRepGraph_CoEdgeId(i)));
  bool same=false;BR_TRY(same=c->SameParameter(BRepGraph_CoEdgeId(i)));br_flag(same);}
 for(uint32_t i=0;i<=g.Topo().Wires().Nb();++i){bool closed=false;br_flag(c->GetWireIsClosed(BRepGraph_WireId(i),closed));br_flag(closed);}
 for(uint32_t i=0;i<=g.Topo().Shells().Nb();++i)br_flag(c->IsShellClosed(BRepGraph_ShellId(i)));}
void bc_stamp(const BRepGraph_CacheMesh::EntryStamp& s){br_int((long long)s.RecipeHash);br_int(s.SlotGeneration);}
// One slot of the mesh cache: every face, edge and coedge entry (fresh and raw).
void bc_mesh_slot(const BrScene& s,const BRepGraph& g,const BRepGraph_CacheMesh& c,uint32_t slot){
 const BrGeometry& G=s.geometry;
 for(uint32_t i=0;i<=g.Topo().Faces().Nb();++i){BRepGraph_FaceId f(i);const auto* e=c.FindFaceMesh(slot,f);
  if(e==nullptr)br_word("N");else{br_int(br_id(G.triangulations,e->Triangulation));bc_stamp(e->Stamp);br_int(e->MeshGeneration);bg_node(e->Node());}
  const auto* raw=c.findFaceEntryRaw(slot,f);if(raw==nullptr)br_word("N");else{br_int(br_id(G.triangulations,raw->Triangulation));br_int(raw->MeshGeneration);}}
 for(uint32_t i=0;i<=g.Topo().Edges().Nb();++i){const auto* e=c.FindEdgeMesh(slot,BRepGraph_EdgeId(i));
  if(e==nullptr)br_word("N");else{br_int(br_id(G.polygons3d,e->Polygon3D));bc_stamp(e->Stamp);}}
 for(uint32_t i=0;i<=g.Topo().CoEdges().Nb();++i){BRepGraph_CoEdgeId x(i);br_flag(c.HasCoEdgeMesh(slot,x));
  const auto* p=c.FindCoEdgePolygon2D(slot,x);if(p==nullptr)br_word("N");else br_int(br_id(G.polygons2d,p->Polygon2D));
  const auto* t=c.FindCoEdgePolygonOnTri(slot,x);if(t==nullptr)br_word("N");else{br_int(t->PolygonsOnTri.Size());for(const auto& n:t->PolygonsOnTri)br_int(br_id(G.nodes,n));}
  const auto* raw=c.findCoEdgeEntryRaw(slot,x);if(raw==nullptr){br_word("N");continue;}
  br_int(br_id(G.polygons2d,raw->Polygon2D));br_int(raw->PolygonsOnTri.Size());br_int(raw->BoundFaceId.Index);br_int(raw->FaceMeshGeneration);
  br_flag(raw->CoEdgeStamp.IsBound());br_flag(raw->FaceTopologyStamp.IsBound());bc_stamp(raw->SlotStamp);}}
// The mesh cache: the active slot, four slot states, three slots' entries and the Needs
// answers of every face and compound.
void bc_mesh_report(const BrScene& s,BRepGraph& g){
 occ::handle<BRepGraph_CacheMesh> c=g.CacheRegistry().Find<BRepGraph_CacheMesh>();if(c.IsNull()){br_word("N");return;}
 br_int(c->ActiveDisplaySlot());
 for(uint32_t slot=0;slot<4;++slot){auto st=c->State(slot);br_int(st.Slot);br_int((long long)st.RecipeHash);br_int(st.Generation);br_flag(st.HasDriver);br_flag(c->DriverOf(slot).IsNull());}
 for(uint32_t slot=0;slot<3;++slot)bc_mesh_slot(s,g,*c,slot);
 for(uint32_t i=0;i<g.Topo().Faces().Nb();++i){bool needs=false;BR_TRY(needs=c->Needs(g,BRepGraph_FaceId(i),0));br_flag(needs);}
 for(uint32_t i=0;i<g.Topo().Compounds().Nb();++i){bool needs=false;BR_TRY(needs=c->Needs(g,BRepGraph_CompoundId(i),0));br_flag(needs);}}
#include "tkbrep_probe_brepgraph_cache_view.cxx"
