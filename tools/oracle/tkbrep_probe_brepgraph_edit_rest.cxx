// The wire, face, reference, generic, deferred, guard and supplement actions of
// `br_bedit`, its dispatch and its reports (tkbrep_probe_brepgraph_edit.cxx).
// 182-198: wires and faces. 182 CheckCoEdgeOrder(coedges), 183 CheckCoEdgeOrder(wire,
// coedges), 184 CheckAppendCoEdge, 185 CheckReplaceEdge(wire, old, new, reversed), 186
// ReplaceEdge, 187 RemoveCoEdge, 188 Wires().Reverse, 189 SetCoEdgeOrder, 190
// SetRefOrientation(wref, o), 191 SetRefChildWireId, 192 Faces().SetTolerance, 193
// SetSurface, 194 ClearSurface, 195 SetPersistentTriangulation, 196
// ClearPersistentTriangulation, 197 SetRefOrientation(fref, o), 198 SetRefFaceId.
bool be_wire_face_action(BRepGraph& g,const BrScene& s,BrReader& r,int code){
 if(code<182||code>198)return false;
 BRepGraph::EditorView::WireOps& w=g.Editor().Wires();BRepGraph::EditorView::FaceOps& f=g.Editor().Faces();const BrGeometry& G=s.geometry;
 switch(code){
  case 182:br_int(int(w.CheckCoEdgeOrder(be_ids<BRepGraph_CoEdgeId>(r))));return true;
  case 183:{auto x=be_id<BRepGraph_WireId>(r);br_int(int(w.CheckCoEdgeOrder(x,be_ids<BRepGraph_CoEdgeId>(r))));return true;}
  case 184:{auto x=be_id<BRepGraph_WireId>(r);br_int(int(w.CheckAppendCoEdge(x,be_id<BRepGraph_CoEdgeId>(r))));return true;}
  case 185:{auto x=be_id<BRepGraph_WireId>(r);auto a=be_id<BRepGraph_EdgeId>(r),b=be_id<BRepGraph_EdgeId>(r);br_int(int(w.CheckReplaceEdge(x,a,b,r.flag())));return true;}
  case 186:{auto x=be_id<BRepGraph_WireId>(r);auto a=be_id<BRepGraph_EdgeId>(r),b=be_id<BRepGraph_EdgeId>(r);w.ReplaceEdge(x,a,b,r.flag());break;}
  case 187:{auto x=be_id<BRepGraph_WireId>(r);br_flag(w.RemoveCoEdge(x,be_id<BRepGraph_CoEdgeId>(r)));return true;}
  case 188:w.Reverse(be_id<BRepGraph_WireId>(r));break;
  case 189:{auto x=be_id<BRepGraph_WireId>(r);br_flag(w.SetCoEdgeOrder(x,be_ids<BRepGraph_CoEdgeId>(r)));return true;}
  case 190:{auto x=be_id<BRepGraph_WireRefId>(r);w.SetRefOrientation(x,be_o(r));break;}
  case 191:{auto x=be_id<BRepGraph_WireRefId>(r);w.SetRefChildWireId(x,be_id<BRepGraph_WireId>(r));break;}
  case 192:{auto x=be_id<BRepGraph_FaceId>(r);f.SetTolerance(x,r.real());break;}
  case 193:{auto x=be_id<BRepGraph_FaceId>(r);f.SetSurface(x,br_at(G.surfaces,r.integer()));break;}
  case 194:f.ClearSurface(be_id<BRepGraph_FaceId>(r));break;
  case 195:{auto x=be_id<BRepGraph_FaceId>(r);f.SetPersistentTriangulation(x,br_at(G.triangulations,r.integer()));break;}
  case 196:f.ClearPersistentTriangulation(be_id<BRepGraph_FaceId>(r));break;
  case 197:{auto x=be_id<BRepGraph_FaceRefId>(r);f.SetRefOrientation(x,be_o(r));break;}
  default:{auto x=be_id<BRepGraph_FaceRefId>(r);f.SetRefFaceId(x,be_id<BRepGraph_FaceId>(r));break;}}
 br_word("-");return true;}
// 199-216: references and generic operations. 199 Shells().SetRefOrientation(sref, o),
// 200 SetRefChildShellId, 201 Solids().SetRefOrientation(soref, o), 202
// SetRefChildSolidId, 203 Compounds().ReplaceChild(cref, node), 204
// CompSolids().ReplaceSolid(soref, solid), 205 Occurrences().SetRefLocalLocation(oref,
// location), 206 SetChildNodeId(occurrence, node), 207 SetRefChildOccurrenceId, 208
// Gen().RemoveNode, 209 ReplaceNode, 210 RemoveSubgraph, 211 RemoveRef(ref), 212
// RemoveRef(parent, ref, prune), 213 SetChildRefOrientation, 214 SetChildRefChildNodeId,
// 215 SetChildRefLocalLocation, 216 CleanupRemovedReferences.
bool be_ref_action(BRepGraph& g,const BrScene& s,BrReader& r,int code){
 if(code<199||code>216)return false;
 BRepGraph::EditorView& e=g.Editor();BRepGraph::EditorView::GenOps& x=e.Gen();
 switch(code){
  case 199:{auto i=be_id<BRepGraph_ShellRefId>(r);e.Shells().SetRefOrientation(i,be_o(r));break;}
  case 200:{auto i=be_id<BRepGraph_ShellRefId>(r);e.Shells().SetRefChildShellId(i,be_id<BRepGraph_ShellId>(r));break;}
  case 201:{auto i=be_id<BRepGraph_SolidRefId>(r);e.Solids().SetRefOrientation(i,be_o(r));break;}
  case 202:{auto i=be_id<BRepGraph_SolidRefId>(r);e.Solids().SetRefChildSolidId(i,be_id<BRepGraph_SolidId>(r));break;}
  case 203:{auto i=be_id<BRepGraph_ChildRefId>(r);e.Compounds().ReplaceChild(i,bl_node(r));break;}
  case 204:{auto i=be_id<BRepGraph_SolidRefId>(r);e.CompSolids().ReplaceSolid(i,be_id<BRepGraph_SolidId>(r));break;}
  case 205:{auto i=be_id<BRepGraph_OccurrenceRefId>(r);e.Occurrences().SetRefLocalLocation(i,s.graph.loc(r.integer()));break;}
  case 206:{auto i=be_id<BRepGraph_OccurrenceId>(r);e.Occurrences().SetChildNodeId(i,bl_node(r));break;}
  case 207:{auto i=be_id<BRepGraph_OccurrenceRefId>(r);e.Occurrences().SetRefChildOccurrenceId(i,be_id<BRepGraph_OccurrenceId>(r));break;}
  case 208:x.RemoveNode(bl_node(r));break;
  case 209:{BRepGraph_NodeId a=bl_node(r);x.ReplaceNode(a,bl_node(r));break;}
  case 210:x.RemoveSubgraph(bl_node(r));break;
  case 211:br_flag(x.RemoveRef(bl_ref(r)));return true;
  case 212:{BRepGraph_NodeId a=bl_node(r);BRepGraph_RefId b=bl_ref(r);br_flag(x.RemoveRef(a,b,r.flag()));return true;}
  case 213:{auto i=be_id<BRepGraph_ChildRefId>(r);x.SetChildRefOrientation(i,be_o(r));break;}
  case 214:{auto i=be_id<BRepGraph_ChildRefId>(r);x.SetChildRefChildNodeId(i,bl_node(r));break;}
  case 215:{auto i=be_id<BRepGraph_ChildRefId>(r);x.SetChildRefLocalLocation(i,s.graph.loc(r.integer()));break;}
  default:x.CleanupRemovedReferences();break;}
 br_word("-");return true;}
// 217-223: modification and deferred invalidation. 217 ApplyModification(target, label,
// replacements, one nested action run by the modifier), 218 BeginDeferredInvalidation,
// 219 EndDeferredInvalidation, 220 IsDeferredMode, 221 a DeferredScope around n nested
// actions (then IsDeferredMode), 222 CommitMutation, 223 ValidateMutationBoundary with
// its issues.
bool be_deferred_action(BRepGraph& g,const BrScene& s,BlState& st,BrReader& r,int code){
 if(code<217||code>223)return false;
 switch(code){
  case 217:{BRepGraph_NodeId t=bl_node(r);TCollection_AsciiString l=bl_label(r);auto v=bl_list<BRepGraph_NodeId>(r,bl_node);
   g.Editor().Gen().ApplyModification(t,[&](BRepGraph& m,BRepGraph_NodeId){be_action(m,s,st,r);return v;},l);break;}
  case 218:g.Editor().BeginDeferredInvalidation();break;
  case 219:g.Editor().EndDeferredInvalidation();break;
  case 220:br_flag(g.Editor().IsDeferredMode());return true;
  case 221:{int n=r.integer();{BRepGraph_DeferredScope scope(g);for(int i=0;i<n;++i)be_action(g,s,st,r);br_flag(g.Editor().IsDeferredMode());}break;}
  case 222:g.Editor().CommitMutation();break;
  default:{NCollection_LinearVector<BRepGraph::EditorView::BoundaryIssue> v;br_flag(g.Editor().ValidateMutationBoundary(&v));
   br_int(int(v.Size()));for(const auto& i:v){bg_node(i.NodeId);bl_text(i.Description);}return true;}}
 br_word("-");return true;}
// 224-233: mutation guards, each scope printing IsDirty before its release. 224 vertex
// (v, tol, P), 225 edge (e, tol, first, last), 226 coedge (c, o, first, last), 227 face
// (f, tol), 228 a vertex guard left untouched, 229 a vertex guard marked dirty, 230 a face
// reference guard (fref, o), 231 a child reference guard (cref, o), 232 an occurrence
// reference guard (oref, location), 233 a wire reference guard (wref, o).
template<class G> void be_release(G& m){br_flag(m.IsDirty());}
// Every argument is read before the guard is taken, so a refused Mut leaves the reader
// at the next action; the orientation converts (and may raise) after Mut, as natively.
bool be_guard_action(BRepGraph& g,const BrScene& s,BrReader& r,int code){
 if(code<224||code>233)return false;
 BRepGraph::EditorView& e=g.Editor();uint32_t x=uint32_t(r.integer());
 switch(code){
  case 224:{double t=r.real();gp_Pnt p=br_point(r);auto m=e.Vertices().Mut(BRepGraph_VertexId(x));e.Vertices().SetTolerance(m,t);e.Vertices().SetPoint(m,p);be_release(m);break;}
  case 225:{double t=r.real(),f=r.real(),l=r.real();auto m=e.Edges().Mut(BRepGraph_EdgeId(x));e.Edges().SetTolerance(m,t);e.Edges().SetParamRange(m,f,l);be_release(m);break;}
  case 226:{int o=r.integer();double f=r.real(),l=r.real();auto m=e.CoEdges().Mut(BRepGraph_CoEdgeId(x));e.CoEdges().SetOrientation(m,be_parity(o));e.CoEdges().SetParamRange(m,f,l);be_release(m);break;}
  case 227:{double t=r.real();auto m=e.Faces().Mut(BRepGraph_FaceId(x));e.Faces().SetTolerance(m,t);be_release(m);break;}
  case 228:{auto m=e.Vertices().Mut(BRepGraph_VertexId(x));be_release(m);break;}
  case 229:{auto m=e.Vertices().Mut(BRepGraph_VertexId(x));m.MarkDirty();be_release(m);break;}
  case 230:{int o=r.integer();auto m=e.Faces().MutRef(BRepGraph_FaceRefId(x));e.Faces().SetRefOrientation(m,be_parity(o));be_release(m);break;}
  case 231:{int o=r.integer();auto m=e.Gen().MutChildRef(BRepGraph_ChildRefId(x));e.Gen().SetChildRefOrientation(m,be_parity(o));be_release(m);break;}
  case 232:{TopLoc_Location l=s.graph.loc(r.integer());auto m=e.Occurrences().MutRef(BRepGraph_OccurrenceRefId(x));e.Occurrences().SetRefLocalLocation(m,l);be_release(m);break;}
  default:{int o=r.integer();auto m=e.Wires().MutRef(BRepGraph_WireRefId(x));e.Wires().SetRefOrientation(m,be_parity(o));be_release(m);break;}}
 return true;}
// 234-238: 234 Supplement().Attach(node, kind, shape), 235 AttachTo<Kind>(owner kind 0
// vertex 1 edge 2 face 3 solid 4 compsolid 5 shell 6 compound, id, shape, attachment kind
// or -1 for the default), 236 RemoveAttachment(uid), 237 Shapes().RemoveShape(shape),
// 238 Mesh().Editor().PromoteToPersistent().
uint64_t be_attach_to(BRepGraph_SupplementEditor& e,int owner,uint32_t id,const TopoDS_Shape& sh,int kind){
 using AK=BRepGraph_LayerTopoSupplement::AttachmentKind;
 switch(owner){
  case 0:return kind<0?e.AttachToVertex(BRepGraph_VertexId(id),sh):e.AttachToVertex(BRepGraph_VertexId(id),sh,AK(kind));
  case 1:return kind<0?e.AttachToEdge(BRepGraph_EdgeId(id),sh):e.AttachToEdge(BRepGraph_EdgeId(id),sh,AK(kind));
  case 2:return kind<0?e.AttachToFace(BRepGraph_FaceId(id),sh):e.AttachToFace(BRepGraph_FaceId(id),sh,AK(kind));
  case 3:return kind<0?e.AttachToSolid(BRepGraph_SolidId(id),sh):e.AttachToSolid(BRepGraph_SolidId(id),sh,AK(kind));
  case 4:return kind<0?e.AttachToCompSolid(BRepGraph_CompSolidId(id),sh):e.AttachToCompSolid(BRepGraph_CompSolidId(id),sh,AK(kind));
  case 5:return kind<0?e.AttachToShell(BRepGraph_ShellId(id),sh):e.AttachToShell(BRepGraph_ShellId(id),sh,AK(kind));
  default:return kind<0?e.AttachToCompound(BRepGraph_CompoundId(id),sh):e.AttachToCompound(BRepGraph_CompoundId(id),sh,AK(kind));}}
bool be_supplement_action(BRepGraph& g,const BrScene& s,BrReader& r,int code){
 switch(code){
  case 234:{BRepGraph_NodeId n=bl_node(r);auto k=BRepGraph_LayerTopoSupplement::AttachmentKind(r.integer());br_int(g.Editor().Supplement().Attach(n,k,s.graph.shape(r.integer())));return true;}
  case 235:{int o=r.integer();uint32_t id=uint32_t(r.integer());TopoDS_Shape sh=s.graph.shape(r.integer());
   BRepGraph_SupplementEditor e=g.Editor().Supplement();br_int(be_attach_to(e,o,id,sh,r.integer()));return true;}
  case 236:br_flag(g.Editor().Supplement().RemoveAttachment(uint64_t(r.integer())));return true;
  case 237:br_flag(g.Shapes().RemoveShape(s.graph.shape(r.integer())));return true;
  case 238:g.Mesh().Editor().PromoteToPersistent();br_word("-");return true;
  default:return false;}}
// Codes 0-5 ingest (`bg_run`), 60 clears the graph, 98-104 are the mesh-cache writes of
// `br_bcache`, 120-238 the editor actions above, the rest the `br_blayer` actions and the
// `br_bcache` lock actions (61-73).
void be_action(BRepGraph& g,const BrScene& s,BlState& st,BrReader& r){
 int code=r.integer();
 try{
  if(code<6){bg_run(g,s,r,code);return;}
  if(code==60){g.Clear();br_word("-");return;}
  if(bc_mesh_editor_action(g,s,r,code)){br_word("-");return;}
  if(be_add_action(g,s,r,code)||be_container_action(g,r,code)||be_edge_action(g,s,r,code)||be_co_edge_action(g,s,r,code)
     ||be_wire_face_action(g,s,r,code)||be_ref_action(g,s,r,code)||be_deferred_action(g,s,st,r,code)
     ||be_guard_action(g,s,r,code)||be_supplement_action(g,s,r,code))return;
  if(bl_history_action(g,r,code)||bl_registry_action(g,st,r,code)||bl_supplement_action(g,s,r,code)||bl_cache_action(g,st,r,code)
     ||bc_lock_action(g,r,code))return;
  br_word("?");
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
// `br_bedit script nactions (action)...`: the actions, then the graph, the editor state,
// the layers, the caches, the mesh cache and view and the shapes; after the graph is
// gone, every logging layer's event count.
void br_bedit_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);BlState st;
 {BRepGraph g;int n=r.integer();for(int i=0;i<n;++i)be_action(g,s,st,r);
  br_flag(g.IsEmpty());br_flag(g.ValidateRelations());bg_list(g.RootProductIds());
  bg_topo_report(s,g);bg_ref_entries(s,g);bg_ref_groups(g);
  br_flag(g.Editor().IsDeferredMode());
  bl_registry(g,st);bl_history_report(g);bl_supplement_report(s,g);bl_cache_report(g,st);
  bc_mesh_report(s,g);bc_view_report(s,g);bg_shapes(s,g);}
 for(const auto& l:st.events){br_int(int(l->Events.size()));}
 br_end();}
