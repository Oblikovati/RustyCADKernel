// BRepGraph oracle, part 2: the RefsView entries, the ids, UIDs and version stamps, and
// the shapes of the graph (originals, cached and fresh reconstructions, and the
// shared-cache BRepGraphInc_Reconstruct), each reconstructed shape printed as a tree
// whose TShapes are numbered in first-seen order. The Rust twin is
// crates/tkbrep/examples/br_probe/brep_graph_views.rs.
void bg_ref_entries(const BrScene& s,const BRepGraph& g){
 const BRepGraph::RefsView& refs=g.Refs();
 br_int(refs.Shells().Nb());br_int(refs.Shells().NbActive());
 for(uint32_t i=0;i<refs.Shells().Nb();++i){const auto& e=refs.Shells().Entry(BRepGraph_ShellRefId(i));
  br_int(e.UID);br_int(e.ParentSolidId.Index);br_int(e.ChildShellId.Index);br_int(TopAbs_Orientation(e.Orientation));}
 br_int(refs.Faces().Nb());br_int(refs.Faces().NbActive());
 for(uint32_t i=0;i<refs.Faces().Nb();++i){const auto& e=refs.Faces().Entry(BRepGraph_FaceRefId(i));
  br_int(e.UID);br_int(e.ParentShellId.Index);br_int(e.ChildFaceId.Index);br_int(TopAbs_Orientation(e.Orientation));}
 br_int(refs.Wires().Nb());br_int(refs.Wires().NbActive());
 for(uint32_t i=0;i<refs.Wires().Nb();++i){const auto& e=refs.Wires().Entry(BRepGraph_WireRefId(i));
  br_int(e.UID);br_int(e.ParentFaceId.Index);br_int(e.ChildWireId.Index);br_int(TopAbs_Orientation(e.Orientation));}
 br_int(refs.Vertices().Nb());br_int(refs.Vertices().NbActive());
 for(uint32_t i=0;i<refs.Vertices().Nb();++i){const auto& e=refs.Vertices().Entry(BRepGraph_VertexRefId(i));
  br_int(e.UID);br_int(e.ChildVertexId.Index);br_int(e.ParentEdgeId.Index);br_int(TopAbs_Orientation(e.Orientation));}
 br_int(refs.Solids().Nb());br_int(refs.Solids().NbActive());
 for(uint32_t i=0;i<refs.Solids().Nb();++i){const auto& e=refs.Solids().Entry(BRepGraph_SolidRefId(i));
  br_int(e.UID);br_int(e.ParentCompSolidId.Index);br_int(e.ChildSolidId.Index);br_int(TopAbs_Orientation(e.Orientation));}
 br_int(refs.Children().Nb());br_int(refs.Children().NbActive());
 for(uint32_t i=0;i<refs.Children().Nb();++i){const auto& e=refs.Children().Entry(BRepGraph_ChildRefId(i));
  br_int(e.UID);br_int(e.ParentCompoundId.Index);bg_node(e.ChildNodeId);br_int(TopAbs_Orientation(e.Orientation));br_chain(s.graph,e.LocalLocation);}
 br_int(refs.Occurrences().Nb());br_int(refs.Occurrences().NbActive());
 for(uint32_t i=0;i<refs.Occurrences().Nb();++i){const auto& e=refs.Occurrences().Entry(BRepGraph_OccurrenceRefId(i));
  br_int(e.UID);br_int(e.ParentProductId.Index);br_int(e.ChildOccurrenceId.Index);br_chain(s.graph,e.LocalLocation);}}
// The grouped IdsOf lists per parent and the reverse lists per child node.
void bg_ref_groups(const BRepGraph& g){
 const BRepGraph::RefsView& refs=g.Refs();const BRepGraph::TopoView& t=g.Topo();
 for(uint32_t i=0;i<t.Solids().Nb();++i)bg_list(refs.Shells().IdsOf(BRepGraph_SolidId(i)));
 for(uint32_t i=0;i<t.Shells().Nb();++i)bg_list(refs.Faces().IdsOf(BRepGraph_ShellId(i)));
 for(uint32_t i=0;i<t.Faces().Nb();++i)bg_list(refs.Wires().IdsOf(BRepGraph_FaceId(i)));
 for(uint32_t i=0;i<t.CompSolids().Nb();++i)bg_list(refs.Solids().IdsOf(BRepGraph_CompSolidId(i)));
 for(uint32_t i=0;i<t.Compounds().Nb();++i)bg_list(refs.Children().IdsOf(BRepGraph_CompoundId(i)));
 for(uint32_t i=0;i<t.Products().Nb();++i)bg_list(refs.Occurrences().IdsOf(BRepGraph_ProductId(i)));
 for(int k=0;k<12;++k){if(!BRepGraph_NodeId::IsValidKind(BRepGraph_NodeId::Kind(k)))continue;
  for(uint32_t i=0;i<t.Gen().Nb(BRepGraph_NodeId::Kind(k));++i){BRepGraph_NodeId n(BRepGraph_NodeId::Kind(k),i);
   bg_list(refs.Children().IdsReferencing(n));bg_list(refs.Occurrences().IdsReferencing(n));
   for(int step=-1;step<4;++step)bg_ref(refs.Gen().RefAtStep(n,step));}}}
// The generic reference queries, the UIDs and stamps, and a usage path per reference.
const Standard_GUID bg_fixed_guid("12345678-9abc-def0-1234-56789abcdef0");
void bg_guid(const Standard_GUID& guid){
 char text[Standard_GUID_SIZE_ALLOC];Standard_PCharacter out=text;guid.ToCString(out);br_text(std::string(text));}
void bg_stamp(const BRepGraph& g,const BRepGraph_VersionStamp& st){
 br_flag(st.IsValid());br_flag(st.IsNodeStamp());br_flag(st.IsRefStamp());br_int(st.myMutationGen);br_int(st.myGeneration);
 bg_hash(st.HashValue());bg_hash(st.ItemUID().HashValue());br_flag(st.IsSameItem(st));br_flag(g.UIDs().IsStale(st));
 bg_guid(st.ToGUID(bg_fixed_guid));}
void bg_ref_ids(const BrScene& s,const BRepGraph& g){
 const BRepGraph::RefsView::GenOps& gen=g.Refs().Gen();
 for(int k=0;k<7;++k){BRepGraph_RefId::Kind kind=BRepGraph_RefId::Kind(k);uint32_t nb=gen.Nb(kind);br_int(nb);
  for(uint32_t i=0;i<nb;++i){BRepGraph_RefId ref(kind,i);
   br_flag(gen.IsValid(ref));br_flag(gen.IsActive(ref));br_flag(gen.IsRemoved(ref));bg_node(gen.ChildNode(ref));
   br_chain(s.graph,gen.LocalLocation(ref));br_int(gen.Orientation(ref));bg_hash(std::hash<BRepGraph_RefId>{}(ref));
   BRepGraph_RefUID uid=g.UIDs().Of(ref);br_int(int(uid.Kind));br_int(uid.Counter);bg_hash(uid.HashValue());
   bg_ref(g.UIDs().RefIdFrom(uid));br_flag(g.UIDs().Has(uid));BRepGraph_ItemId item(ref);bg_hash(std::hash<BRepGraph_ItemId>{}(item));
   bg_stamp(g,g.UIDs().StampOf(ref));
   BRepGraph_UsagePath path(2);path.Append({gen.ChildNode(ref),ref,int(i)});path.InsertBefore(0,{gen.ChildNode(ref),BRepGraph_RefId(),-1});
   br_int(int(path.Size()));bg_hash(path.HashCode());br_flag(path==path);}
  br_flag(gen.IsValid(BRepGraph_RefId(kind,nb)));br_flag(gen.IsRemoved(BRepGraph_RefId(kind,nb)));}}
// The UIDs, items and stamps of every node, and the representation ids of the edges, coedges and faces.
void bg_node_ids(const BRepGraph& g){
 const BRepGraph::UIDsView& uids=g.UIDs();br_int(uids.Generation());
 for(int k=0;k<12;++k){if(!BRepGraph_NodeId::IsValidKind(BRepGraph_NodeId::Kind(k)))continue;
  for(uint32_t i=0;i<g.Topo().Gen().Nb(BRepGraph_NodeId::Kind(k));++i){BRepGraph_NodeId n(BRepGraph_NodeId::Kind(k),i);
   BRepGraph_UID uid=uids.Of(n);br_int(int(uid.Kind));br_int(uid.Counter);bg_hash(uid.HashValue());bg_hash(std::hash<BRepGraph_NodeId>{}(n));
   bg_node(uids.NodeIdFrom(uid));br_flag(uids.Has(uid));BRepGraph_ItemId item(n);bg_hash(std::hash<BRepGraph_ItemId>{}(item));
   BRepGraph_ItemUID iuid=uids.Of(item);bg_hash(iuid.HashValue());br_flag(uids.Has(iuid));
   BRepGraph_ItemId back=uids.ItemIdFrom(iuid);br_int(int(back.ItemDomain()));br_int(back.RawKind());br_int(back.Index());
   bg_stamp(g,uids.StampOf(n));bg_stamp(g,uids.StampOf(item));}}
 for(uint32_t i=0;i<g.Topo().Faces().Nb();++i){BRepGraph_FaceId f(i);bg_hash(std::hash<BRepGraph_FaceId>{}(f));
  BRepGraph_FaceSurfaceRepId rep=g.Topo().Faces().Definition(f).SurfaceRepId;
  bg_hash(std::hash<BRepGraph_FaceSurfaceRepId>{}(rep));bg_hash(std::hash<BRepGraph_RepId>{}(BRepGraph_RepId(rep)));bg_stamp(g,uids.StampOf(BRepGraph_RepId(rep)));}
 for(uint32_t i=0;i<g.Topo().Edges().Nb();++i){BRepGraph_EdgeCurve3DRepId rep=g.Topo().Edges().Definition(BRepGraph_EdgeId(i)).Curve3DRepId;
  bg_hash(std::hash<BRepGraph_EdgeCurve3DRepId>{}(rep));bg_stamp(g,uids.StampOf(BRepGraph_RepId(rep)));}
 for(uint32_t i=0;i<g.Topo().CoEdges().Nb();++i){BRepGraph_CoEdgeCurve2DRepId rep=g.Topo().CoEdges().Definition(BRepGraph_CoEdgeId(i)).Curve2DRepId;
  bg_hash(std::hash<BRepGraph_RepId>{}(BRepGraph_RepId(rep)));bg_stamp(g,uids.StampOf(BRepGraph_RepId(rep)));}}
// A reconstructed shape as a tree; a TShape seen before prints `S` and its number.
void bg_shape(const BrScene& s,std::vector<TopoDS_Shape>& seen,const TopoDS_Shape& sh){
 if(sh.IsNull()){br_word("N");return;}
 br_int(sh.ShapeType());br_int(sh.Orientation());br_chain(s.graph,sh.Location());br_int(s.graph.tshape_id(sh));
 for(size_t i=0;i<seen.size();++i)if(seen[i].IsPartner(sh)){br_word("S");br_int(int(i));return;}
 seen.push_back(sh);br_flag(sh.Closed());br_int(sh.NbChildren());
 switch(sh.ShapeType()){
  case TopAbs_VERTEX:br_vertex_report(s,TopoDS::Vertex(sh));break;
  case TopAbs_EDGE:br_edge_report(s,TopoDS::Edge(sh));break;
  case TopAbs_FACE:br_face_report(s,TopoDS::Face(sh));break;
  default:break;}
 for(TopoDS_Iterator it(sh,false,false);it.More();it.Next())bg_shape(s,seen,it.Value());}
void bg_tree(const BrScene& s,const TopoDS_Shape& sh){std::vector<TopoDS_Shape> seen;bg_shape(s,seen,sh);}
// Every node: its original, Shapes().Shape twice (cache hit), Reconstruct; then one
// shared-cache pass of BRepGraphInc_Reconstruct::Node and a FaceWithCache.
void bg_shapes(const BrScene& s,BRepGraph& g){
 for(const TopoDS_Shape& sh:s.graph.shapes){bg_node(g.Shapes().FindNode(sh));br_flag(g.Shapes().HasNode(sh));}
 for(int k=0;k<12;++k){if(!BRepGraph_NodeId::IsValidKind(BRepGraph_NodeId::Kind(k)))continue;
  for(uint32_t i=0;i<g.Topo().Gen().Nb(BRepGraph_NodeId::Kind(k));++i){BRepGraph_NodeId n(BRepGraph_NodeId::Kind(k),i);
   br_flag(g.Shapes().HasOriginal(n));br_desc(s.graph,g.Shapes().Original(n));
   TopoDS_Shape first;BR_TRY(first=g.Shapes().Shape(n);bg_tree(s,first));
   BR_TRY(br_flag(g.Shapes().Shape(n).IsSame(first)));BR_TRY(bg_tree(s,g.Shapes().Reconstruct(n)));}}
 BRepGraphInc_Reconstruct::Cache cache;std::vector<TopoDS_Shape> seen;
 for(int k=0;k<8;++k)for(uint32_t i=0;i<g.Topo().Gen().Nb(BRepGraph_NodeId::Kind(k));++i)
  BR_TRY(bg_shape(s,seen,BRepGraphInc_Reconstruct::Node(g,BRepGraph_NodeId(BRepGraph_NodeId::Kind(k),i),cache)));
 if(g.Topo().Faces().Nb()>0){BRepGraphInc_Reconstruct::Cache fresh;BR_TRY(bg_tree(s,BRepGraphInc_Reconstruct::FaceWithCache(g,BRepGraph_FaceId(0),fresh)));}}
// `br_bgraph script nactions (action)...`: the actions, then the graph reports.
void br_bgraph_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);BRepGraph g;
 int n=r.integer();for(int i=0;i<n;++i)bg_action(g,s,r);
 br_flag(g.IsEmpty());br_flag(g.ValidateRelations());bg_list(g.RootProductIds());
 bg_topo_report(s,g);bg_ref_entries(s,g);bg_ref_groups(g);bg_ref_ids(s,g);bg_node_ids(g);bg_shapes(s,g);
 br_end();}
