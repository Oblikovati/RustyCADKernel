// The mesh-cache and copy actions of `br_bcache` (tkbrep_probe_brepgraph_cache.cxx).
// 90-112: the mesh cache. 90 RegisterDriver(slot, new driver: index recipe tri poly
// result), 91 RegisterDriver(slot, an earlier driver), 92 UnregisterDriver, 93
// SetActiveDisplaySlot, 94 Ensure(slot), 95 Ensure(node, slot), 96 Ensure(nodes, slot),
// 97 Needs(node, slot), 98 Faces().SetCachedTriangulation(face, tri), 99 Faces().Clear,
// 100 Edges().SetCachedPolygon3D(edge, poly), 101 Edges().Clear, 102
// CoEdges().AppendCachedPolygonOnTri(coedge, nodes), 103 CoEdges().SetCachedPolygon2D
// (coedge, poly2d), 104 CoEdges().Clear, 105 BumpFaceMeshGeneration(face, slot), 106
// Clear, 107 ChangeFaceMesh(slot, face) = tri + BindFresh, 108 ClearFaceMesh(slot,
// face), 109 ChangeEdgeMesh(slot, edge) = poly + BindFresh, 110 ChangeCoEdgeMesh(slot,
// coedge) = poly2d + BindFresh, 111 ClearEdgeMesh(slot, edge), 112 ClearCoEdgeMesh.
// Geometry arguments are scene registry indices (-1 is a null handle).
bool bc_mesh_driver_action(BRepGraph& g,const BrScene& s,BcState& st,BrReader& r,int code){
 if(code<90||code>97)return false;
 occ::handle<BRepGraph_CacheMesh> c=bc_mesh(g);
 switch(code){
  case 90:{uint32_t slot=uint32_t(r.integer());int i=r.integer();uint64_t recipe=uint64_t(r.integer());int t=r.integer(),p=r.integer();
   occ::handle<BcDriver> d=new BcDriver(i,recipe,br_at(s.geometry.triangulations,t),br_at(s.geometry.polygons3d,p),r.flag());
   st.drivers.push_back(d);c->RegisterDriver(slot,d);break;}
  case 91:{uint32_t slot=uint32_t(r.integer());c->RegisterDriver(slot,st.drivers.at(size_t(r.integer())));break;}
  case 92:c->UnregisterDriver(uint32_t(r.integer()));break;
  case 93:c->SetActiveDisplaySlot(uint32_t(r.integer()));break;
  case 94:br_flag(c->Ensure(g,uint32_t(r.integer())));break;
  case 95:{BRepGraph_NodeId n=bl_node(r);br_flag(c->Ensure(g,n,uint32_t(r.integer())));break;}
  case 96:{auto v=bl_list<BRepGraph_NodeId>(r,bl_node);br_flag(c->Ensure(g,v.ToArray1(),uint32_t(r.integer())));break;}
  case 97:{BRepGraph_NodeId n=bl_node(r);br_flag(c->Needs(g,n,uint32_t(r.integer())));break;}
  default:return false;}
 return true;}
bool bc_mesh_editor_action(BRepGraph& g,const BrScene& s,BrReader& r,int code){
 const BrGeometry& G=s.geometry;
 switch(code){
  case 98:{BRepGraph_FaceId f(uint32_t(r.integer()));g.Mesh().Editor().Faces().SetCachedTriangulation(f,br_at(G.triangulations,r.integer()));break;}
  case 99:g.Mesh().Editor().Faces().Clear(BRepGraph_FaceId(uint32_t(r.integer())));break;
  case 100:{BRepGraph_EdgeId e(uint32_t(r.integer()));g.Mesh().Editor().Edges().SetCachedPolygon3D(e,br_at(G.polygons3d,r.integer()));break;}
  case 101:g.Mesh().Editor().Edges().Clear(BRepGraph_EdgeId(uint32_t(r.integer())));break;
  case 102:{BRepGraph_CoEdgeId x(uint32_t(r.integer()));g.Mesh().Editor().CoEdges().AppendCachedPolygonOnTri(x,br_at(G.nodes,r.integer()));break;}
  case 103:{BRepGraph_CoEdgeId x(uint32_t(r.integer()));g.Mesh().Editor().CoEdges().SetCachedPolygon2D(x,br_at(G.polygons2d,r.integer()));break;}
  case 104:g.Mesh().Editor().CoEdges().Clear(BRepGraph_CoEdgeId(uint32_t(r.integer())));break;
  default:return false;}
 return true;}
bool bc_mesh_slot_action(BRepGraph& g,const BrScene& s,BrReader& r,int code){
 if(code<105||code>112)return false;
 occ::handle<BRepGraph_CacheMesh> c=bc_mesh(g);const BrGeometry& G=s.geometry;
 switch(code){
  case 105:{BRepGraph_FaceId f(uint32_t(r.integer()));c->BumpFaceMeshGeneration(f,uint32_t(r.integer()));break;}
  case 106:c->Clear();break;
  case 107:{uint32_t slot=uint32_t(r.integer());BRepGraph_FaceId f(uint32_t(r.integer()));
   auto& e=c->ChangeFaceMesh(slot,f);e.Triangulation=br_at(G.triangulations,r.integer());c->BindFresh(e,f);break;}
  case 108:{uint32_t slot=uint32_t(r.integer());c->ClearFaceMesh(slot,BRepGraph_FaceId(uint32_t(r.integer())));break;}
  case 109:{uint32_t slot=uint32_t(r.integer());BRepGraph_EdgeId x(uint32_t(r.integer()));
   auto& e=c->ChangeEdgeMesh(slot,x);e.Polygon3D=br_at(G.polygons3d,r.integer());c->BindFresh(e,x);break;}
  case 110:{uint32_t slot=uint32_t(r.integer());BRepGraph_CoEdgeId x(uint32_t(r.integer()));
   auto& e=c->ChangeCoEdgeMesh(slot,x);e.Polygon2D=br_at(G.polygons2d,r.integer());c->BindFresh(e,x);break;}
  case 111:{uint32_t slot=uint32_t(r.integer());c->ClearEdgeMesh(slot,BRepGraph_EdgeId(uint32_t(r.integer())));break;}
  case 112:{uint32_t slot=uint32_t(r.integer());c->ClearCoEdgeMesh(slot,BRepGraph_CoEdgeId(uint32_t(r.integer())));break;}
  default:return false;}
 return true;}
// 113-116: copies. 113 CopyLayersTo and CopyFreshCachesTo into the target graph with the
// identity mapping (mode), 114 the same with an explicit item map (mode, count, (source
// item, target item)...), 115 CopyLayersTo of the scripted graph onto itself (Compact,
// explicit map), 116 a target-graph ingestion action (the `bg_run` codes).
NCollection_FlatDataMap<BRepGraph_ItemId,BRepGraph_ItemId> bc_item_map(BrReader& r){
 NCollection_FlatDataMap<BRepGraph_ItemId,BRepGraph_ItemId> m;int n=r.integer();
 for(int i=0;i<n;++i){BRepGraph_ItemId a=bl_item(r);m.Bind(a,bl_item(r));}return m;}
bool bc_copy_action(BcGraphs& gs,const BrScene& s,BrReader& r,int code){
 switch(code){
  case 113:{auto mode=BRepGraph_CopyRemap::Mode(r.integer());
   gs.g.LayerRegistry().CopyLayersTo(gs.t,BRepGraph_CopyRemap::MappingKind::Identity,mode);
   gs.g.CacheRegistry().CopyFreshCachesTo(gs.t,BRepGraph_CopyRemap::MappingKind::Identity,mode);break;}
  case 114:{auto mode=BRepGraph_CopyRemap::Mode(r.integer());auto m=bc_item_map(r);
   gs.g.LayerRegistry().CopyLayersTo(gs.t,m,mode);gs.g.CacheRegistry().CopyFreshCachesTo(gs.t,m,mode);break;}
  case 115:{auto m=bc_item_map(r);gs.g.LayerRegistry().CopyLayersTo(gs.g,m,BRepGraph_CopyRemap::Mode::Compact);break;}
  case 116:{int c=r.integer();bg_run(gs.t,s,r,c);return true;}
  default:return false;}
 br_word("-");return true;}
void bc_action(BcGraphs& gs,const BrScene& s,BcState& st,BrReader& r){
 int code=r.integer();
 try{
  if(code<6){bg_run(gs.g,s,r,code);return;}
  if(code==60){gs.g.Clear();br_word("-");return;}
  if(bc_lock_action(gs.g,r,code)||bc_deferred_action(gs.g,r,code)||bc_derived_action(gs.g,r,code))return;
  if(bc_mesh_driver_action(gs.g,s,st,r,code)||bc_mesh_editor_action(gs.g,s,r,code)||bc_mesh_slot_action(gs.g,s,r,code)){br_word("-");return;}
  if(bc_copy_action(gs,s,r,code))return;
  br_word("?");
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
