// BRepGraph oracle, part 1: a BRep scene of tkbrep_probe_brep.cxx ingested into a
// BRepGraph (BRepGraphInc_Populate and Shapes().Add), then its definitions,
// relations and derived queries printed through the public TopoView. Ids print as
// `kind index`, geometry as scene registry ids (-2 for a transformed copy, followed by
// a point of it). The Rust twin is crates/tkbrep/examples/br_probe/brep_graph.rs.
#include <BRepGraph.hxx>
#include <BRepGraph_RefsView.hxx>
#include <BRepGraph_ShapesView.hxx>
#include <BRepGraph_TopoView.hxx>
#include <BRepGraph_UIDsView.hxx>
#include <BRepGraph_UsagePath.hxx>
#include <BRepGraph_VersionStamp.hxx>
#include <BRepGraphInc_Populate.hxx>
#include <BRepGraphInc_Reconstruct.hxx>
void bg_node(const BRepGraph_NodeId& n){br_int(int(n.NodeKind));br_int(n.Index);}
void bg_ref(const BRepGraph_RefId& r){br_int(int(r.RefKind));br_int(r.Index);}
void bg_hash(size_t h){std::cout<<uint64_t(h)<<' ';}
template<class V> void bg_list(const V& v){br_int(v.Size());for(const auto& i:v)br_int(i.Index);}
void bg_base(const BRepGraphInc::BaseDef& b){br_int(b.UID);br_int(b.OwnGen);br_int(b.SubtreeGen);br_int(b.LastPropWave);}
void bg_curve(const BrScene& s,const occ::handle<Geom_Curve>& c){
 int id=br_id(s.geometry.curves,c);br_int(id);if(id==-2)BR_TRY(br_pnt(c->Value(0.5)));}
void bg_surface(const BrScene& s,const occ::handle<Geom_Surface>& c){
 int id=br_id(s.geometry.surfaces,c);br_int(id);if(id==-2)BR_TRY(br_pnt(c->Value(0.25,0.75)));}
void bg_pcurve(const BrScene& s,const occ::handle<Geom2d_Curve>& c){
 int id=br_id(s.geometry.pcurves,c);br_int(id);if(id==-2)BR_TRY(br_pnt2d(c->Value(0.5)));}
void bg_triangulation(const BrScene& s,const occ::handle<Poly_Triangulation>& t){
 int id=br_id(s.geometry.triangulations,t);br_int(id);if(id==-2){br_int(t->NbNodes());br_pnt(t->Node(1));}}
// Shapes().Add options from bits: 1 CreateAutoProduct, 2 Flatten, 4 Parallel, 8 TrackAddedNodes.
BRepGraph::ShapesView::Options bg_options(int bits){
 BRepGraph::ShapesView::Options o;o.CreateAutoProduct=(bits&1)!=0;o.Flatten=(bits&2)!=0;
 o.Parallel=(bits&4)!=0;o.TrackAddedNodes=(bits&8)!=0;return o;}
void bg_add_result(const BRepGraph::ShapesView::Result& res){
 br_int(int(res.Status));bg_node(res.TopologyRoot);br_int(res.Product.Index);br_int(res.Occurrence.Index);
 bg_ref(res.InsertedRef);br_int(res.AddedNodes.Extent());}
// Actions, each printing its result or the exception it raised: 0 Perform s par,
// 1 Append s par, 2 AppendFlattened s par, 3 Shapes().Add s bits, 4 Shapes().Add s kind
// index bits (under a parent), 5 Shapes().ClearCached kind index.
void bg_run(BRepGraph& g,const BrScene& s,BrReader& r,int code){
  switch(code){
   case 0:{TopoDS_Shape sh=s.graph.shape(r.integer());br_int(int(BRepGraphInc_Populate::Perform(g,sh,r.flag())));break;}
   case 1:{TopoDS_Shape sh=s.graph.shape(r.integer());br_int(int(BRepGraphInc_Populate::Append(g,sh,r.flag())));break;}
   case 2:{TopoDS_Shape sh=s.graph.shape(r.integer());bool p=r.flag();NCollection_LinearVector<BRepGraph_NodeId> roots;
    br_int(int(BRepGraphInc_Populate::AppendFlattened(g,sh,p,roots)));br_int(roots.Size());for(const auto& n:roots)bg_node(n);break;}
   case 3:{TopoDS_Shape sh=s.graph.shape(r.integer());bg_add_result(g.Shapes().Add(sh,bg_options(r.integer())));break;}
   case 4:{TopoDS_Shape sh=s.graph.shape(r.integer());int k=r.integer();BRepGraph_NodeId parent(BRepGraph_NodeId::Kind(k),r.integer());
    bg_add_result(g.Shapes().Add(sh,parent,bg_options(r.integer())));break;}
   default:{int k=r.integer();g.Shapes().ClearCached(BRepGraph_NodeId(BRepGraph_NodeId::Kind(k),r.integer()));br_word("-");break;}}}
void bg_action(BRepGraph& g,const BrScene& s,BrReader& r){
 int code=r.integer();
 try{bg_run(g,s,r,code);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
// The per-kind definitions with their relations and derived queries.
void bg_vertices(const BRepGraph& g){
 const auto& ops=g.Topo().Vertices();br_int(ops.Nb());br_int(ops.NbActive());
 for(uint32_t i=0;i<ops.Nb();++i){BRepGraph_VertexId v(i);const auto& d=ops.Definition(v);
  bg_base(d);br_pnt(d.Point);br_real(d.Tolerance);bg_list(ops.Edges(v));}}
void bg_edges(const BrScene& s,const BRepGraph& g){
 const auto& ops=g.Topo().Edges();br_int(ops.Nb());br_int(ops.NbActive());
 for(uint32_t i=0;i<ops.Nb();++i){BRepGraph_EdgeId e(i);const auto& d=ops.Definition(e);
  bg_base(d);br_int(d.Curve3DRepId.Index);br_real(d.Tolerance);br_int(d.StartVertexRefId.Index);br_int(d.EndVertexRefId.Index);
  br_int(d.Polygon3DRepId.Index);bg_list(ops.Relations(e).CoEdgeIds);br_int(ops.NbFaces(e));bg_curve(s,ops.Curve3D(e));}}
void bg_co_edges(const BrScene& s,const BRepGraph& g){
 const auto& ops=g.Topo().CoEdges();br_int(ops.Nb());br_int(ops.NbActive());
 for(uint32_t i=0;i<ops.Nb();++i){BRepGraph_CoEdgeId c(i);const auto& d=ops.Definition(c);
  bg_base(d);br_int(d.ParentWireId.Index);br_int(d.ChildEdgeId.Index);br_int(d.FaceId.Index);br_int(TopAbs_Orientation(d.Orientation));
  br_int(d.Curve2DRepId.Index);br_int(d.Polygon2DRepId.Index);br_int(d.PolygonOnTriRepId.Index);
  br_int(ops.Edge(c).Index);br_int(ops.Face(c).Index);br_int(ops.Wire(c).Index);bg_pcurve(s,ops.Curve2D(c));}}
void bg_faces(const BrScene& s,const BRepGraph& g){
 const auto& ops=g.Topo().Faces();br_int(ops.Nb());br_int(ops.NbActive());
 for(uint32_t i=0;i<ops.Nb();++i){BRepGraph_FaceId f(i);const auto& d=ops.Definition(f);
  bg_base(d);br_int(d.SurfaceRepId.Index);br_int(d.TriangulationRepId.Index);br_real(d.Tolerance);
  bg_list(ops.Relations(f).WireRefIds);bg_list(ops.Relations(f).ParentFaceRefIds);
  bg_surface(s,ops.Surface(f));bg_triangulation(s,ops.ActiveTriangulation(f));}}
void bg_containers(const BRepGraph& g){
 const BRepGraph::TopoView& t=g.Topo();
 br_int(t.Wires().Nb());br_int(t.Wires().NbActive());
 for(uint32_t i=0;i<t.Wires().Nb();++i){BRepGraph_WireId w(i);bg_base(t.Wires().Definition(w));
  bg_list(t.Wires().Relations(w).CoEdgeIds);bg_list(t.Wires().Relations(w).ParentWireRefIds);}
 br_int(t.Shells().Nb());br_int(t.Shells().NbActive());
 for(uint32_t i=0;i<t.Shells().Nb();++i){BRepGraph_ShellId x(i);bg_base(t.Shells().Definition(x));
  bg_list(t.Shells().Relations(x).FaceRefIds);bg_list(t.Shells().Relations(x).ParentShellRefIds);}
 br_int(t.Solids().Nb());br_int(t.Solids().NbActive());
 for(uint32_t i=0;i<t.Solids().Nb();++i){BRepGraph_SolidId x(i);bg_base(t.Solids().Definition(x));
  bg_list(t.Solids().Relations(x).ShellRefIds);bg_list(t.Solids().Relations(x).ParentSolidRefIds);}
 br_int(t.Compounds().Nb());br_int(t.Compounds().NbActive());
 for(uint32_t i=0;i<t.Compounds().Nb();++i){BRepGraph_CompoundId x(i);bg_base(t.Compounds().Definition(x));bg_list(t.Compounds().Relations(x).ChildRefIds);}
 br_int(t.CompSolids().Nb());br_int(t.CompSolids().NbActive());
 for(uint32_t i=0;i<t.CompSolids().Nb();++i){BRepGraph_CompSolidId x(i);bg_base(t.CompSolids().Definition(x));bg_list(t.CompSolids().Relations(x).SolidRefIds);}}
void bg_assembly(const BrScene& s,const BRepGraph& g){
 const BRepGraph::TopoView& t=g.Topo();
 br_int(t.Products().Nb());br_int(t.Products().NbActive());
 for(uint32_t i=0;i<t.Products().Nb();++i){BRepGraph_ProductId p(i);bg_base(t.Products().Definition(p));
  bg_list(t.Products().Relations(p).OccurrenceRefIds);bg_node(t.Products().ShapeRoot(p));bg_node(t.Products().ShapeRootNode(p));
  br_flag(t.Products().IsAssembly(p));br_flag(t.Products().IsPart(p));int n=int(t.Products().NbComponents(p));br_int(n);
  for(int c=0;c<=n;++c)br_int(t.Products().Component(p,c).Index);}
 br_int(t.Occurrences().Nb());br_int(t.Occurrences().NbActive());
 for(uint32_t i=0;i<t.Occurrences().Nb();++i){BRepGraph_OccurrenceId o(i);const auto& d=t.Occurrences().Definition(o);
  bg_base(d);bg_node(d.ChildNodeId);bg_list(t.Occurrences().Relations(o).ParentOccurrenceRefIds);
  br_int(t.Occurrences().Product(o).Index);br_int(t.Occurrences().ParentProduct(o).Index);br_chain(s.graph,t.Occurrences().OccurrenceLocation(o));}}
// The generic node queries over every kind slot, the reserved kind 9 included; the
// first index past the end only reaches the validity queries (the others read the
// storage planes unchecked natively).
void bg_generic(const BRepGraph& g){
 const BRepGraph::TopoView::GenOps& gen=g.Topo().Gen();br_int(gen.NbNodes());
 for(int k=0;k<12;++k){
  BRepGraph_NodeId::Kind kind=BRepGraph_NodeId::Kind(k);uint32_t nb=gen.Nb(kind);br_int(nb);
  for(uint32_t i=0;i<nb;++i){BRepGraph_NodeId n(kind,i);
   br_flag(gen.IsValid(n));br_flag(gen.IsActive(n));br_flag(gen.IsRemoved(n));br_flag(gen.TopoEntity(n)!=nullptr);
   br_flag(gen.HasCompoundParents(n));br_flag(gen.HasOccurrenceParents(n));bg_list(gen.CompoundRefIds(n));bg_list(gen.OccurrenceRefIds(n));}
  BRepGraph_NodeId past(kind,nb);br_flag(gen.IsValid(past));br_flag(gen.IsActive(past));br_flag(gen.IsRemoved(past));}
 const BRepGraph::TopoView::GeometryOps& geo=g.Topo().Geometry();
 br_int(geo.NbFaceSurfaces());br_int(geo.NbEdgeCurves3D());br_int(geo.NbCoEdgeCurves2D());
 br_int(geo.NbActiveFaceSurfaces());br_int(geo.NbActiveEdgeCurves3D());br_int(geo.NbActiveCoEdgeCurves2D());}
void bg_topo_report(const BrScene& s,const BRepGraph& g){
 bg_vertices(g);bg_edges(s,g);bg_co_edges(s,g);bg_faces(s,g);bg_containers(g);bg_assembly(s,g);bg_generic(g);}
