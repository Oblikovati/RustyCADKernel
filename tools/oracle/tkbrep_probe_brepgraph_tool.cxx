// BRepGraph oracle, part 5: every BRepGraph_Tool member over every vertex, edge,
// coedge, face, wire and shell (and their references, the first past the end
// included), with usages placed at the scene's last location. Adaptors print their
// parameter range and a point at mid-range (a surface at (0.25, 0.75)). The Rust twin
// is crates/tkbrep/examples/br_probe/brep_graph_tool.rs.
#include <Adaptor3d_CurveOnSurface.hxx>
#include <BRepGraph_Tool.hxx>
void bt_curve_adaptor(const GeomAdaptor_TransformedCurve& a){
 bool has=a.IsCurveOnSurface()||!a.GeomCurve().IsNull();br_flag(a.Is3DCurve());br_flag(has);
 if(has)BR_TRY(double f=a.FirstParameter();double l=a.LastParameter();br_real(f);br_real(l);br_pnt(a.Value(0.5*(f+l))));}
void bt_surface_adaptor(const GeomAdaptor_TransformedSurface& a){
 bool has=!a.GeomSurfaceOriginal().IsNull();br_flag(has);br_flag(a.HasTrsf());
 if(has)BR_TRY(double u1=a.FirstUParameter();double u2=a.LastUParameter();double v1=a.FirstVParameter();double v2=a.LastVParameter();
  br_real(u1);br_real(u2);br_real(v1);br_real(v2);br_pnt(a.Value(0.25,0.75)));}
void bt_pcurve_adaptor(const Geom2dAdaptor_Curve& a){
 br_flag(a.IsInitialized());if(a.IsInitialized())BR_TRY(double f=a.FirstParameter();double l=a.LastParameter();br_real(f);br_real(l);br_pnt2d(a.Value(0.5*(f+l))));}
void bt_curve_on_surface(const occ::handle<Adaptor3d_CurveOnSurface>& c){
 br_flag(c.IsNull());if(!c.IsNull())BR_TRY(double f=c->FirstParameter();double l=c->LastParameter();br_real(f);br_real(l);br_pnt(c->Value(0.5*(f+l))));}
void bt_bounds(double u1,double u2,double v1,double v2){br_real(u1);br_real(u2);br_real(v1);br_real(v2);}
void bt_vertices(const BRepGraph& g,const TopLoc_Location& l){
 using T=BRepGraph_Tool::Vertex;
 for(uint32_t i=0;i<g.Topo().Vertices().Nb();++i){BRepGraph_VertexId v(i);
  br_pnt(T::Pnt(g,v));br_real(T::Tolerance(g,v));br_int(T::NbEdges(g,v));br_pnt(T::Pnt(g,BRepGraph_Tool::VertexUsage{v,l,TopAbs_FORWARD}));}
 for(uint32_t i=0;i<=g.Refs().Vertices().Nb();++i){BRepGraph_VertexRefId r(i);BRepGraph_Tool::VertexUsage u=T::Usage(g,r);
  br_flag(u.IsValid());br_int(u.DefId.Index);br_int(u.Orientation);br_pnt(T::Pnt(g,r));br_real(T::Tolerance(g,r));}
 br_end();}
void bt_edge_lookups(const BRepGraph& g,BRepGraph_EdgeId e){
 using T=BRepGraph_Tool::Edge;
 BRepGraph_VertexId v0=BRepGraph_Tool::Vertex::Usage(g,T::StartVertexId(g,e)).DefId,v1=BRepGraph_Tool::Vertex::Usage(g,T::EndVertexId(g,e)).DefId;
 br_int(T::FindByVertices(g,v0,v1).Index);br_int(T::FindByVertices(g,v1,v0).Index);br_int(T::FindByVertices(g,v1,v0,true).Index);
 std::vector<BRepGraph_FaceId> faces;for(BRepGraph_FacesOfEdge it(g,e);it.More();it.Next())faces.push_back(it.CurrentId());
 faces.push_back(BRepGraph_FaceId(g.Topo().Faces().Nb()));
 for(BRepGraph_FaceId f:faces){br_int(T::FindCoEdgeId(g,e,f).Index);br_int(T::FindPCurveCoEdgeId(g,e,f).Index);
  for(TopAbs_Orientation o:{TopAbs_FORWARD,TopAbs_REVERSED}){br_int(T::FindCoEdgeId(g,e,f,o).Index);br_int(T::FindPCurveCoEdgeId(g,e,f,o).Index);}
  br_flag(T::IsSeamOnFace(g,e,f));}}
void bt_edges(const BrScene& s,const BRepGraph& g){
 using T=BRepGraph_Tool::Edge;
 for(uint32_t i=0;i<g.Topo().Edges().Nb();++i){BRepGraph_EdgeId e(i);
  br_real(T::Tolerance(g,e));br_flag(T::Degenerated(g,e));br_flag(T::IsClosed(g,e));std::pair<double,double> r=T::Range(g,e);br_real(r.first);br_real(r.second);
  br_int(T::StartVertexId(g,e).Index);br_int(T::EndVertexId(g,e).Index);br_flag(T::HasCurve(g,e));bg_curve(s,T::Curve(g,e));
  BR_TRY(bt_curve_adaptor(T::CurveAdaptor(g,e)));br_int(T::NbFaces(g,e));br_flag(T::IsManifold(g,e));br_flag(T::IsBoundary(g,e));bt_edge_lookups(g,e);}
 br_end();}
void bt_co_edge_usage(const BrScene& s,const BRepGraph& g,BRepGraph_CoEdgeId c,const TopLoc_Location& l){
 using T=BRepGraph_Tool::Edge;const BRepGraph_Tool::CoEdgeUsage u{c,l,TopAbs_REVERSED};
 BR_TRY(bg_curve(s,T::Curve(g,u)));BR_TRY(bt_curve_adaptor(T::CurveAdaptor(g,u)));
 BR_TRY(bt_pcurve_adaptor(BRepGraph_Tool::CoEdge::PCurveAdaptor(g,u)));
 BRepGraph_FaceId own=BRepGraph_Tool::CoEdge::FaceOf(g,c);
 for(BRepGraph_FaceId f:{own,BRepGraph_FaceId(0)})if(f.IsValid(g.Topo().Faces().Nb()))BR_TRY(bt_curve_on_surface(T::CurveOnSurface(g,u,f)));}
void bt_co_edges(const BrScene& s,const BRepGraph& g,const TopLoc_Location& l){
 using T=BRepGraph_Tool::CoEdge;
 for(uint32_t i=0;i<g.Topo().CoEdges().Nb();++i){BRepGraph_CoEdgeId c(i);
  br_int(T::Orientation(g,c));br_flag(T::IsReversed(g,c));br_int(T::EdgeOf(g,c).Index);br_int(T::FaceOf(g,c).Index);
  br_int(T::SeamPair(g,c).Index);br_flag(T::IsSeam(g,c));br_flag(T::HasPCurve(g,c));BR_TRY(br_flag(T::SameParameter(g,c)));
  br_flag(T::SameRange(g,c));bg_pcurve(s,T::PCurve(g,c));std::pair<double,double> r=T::Range(g,c);br_real(r.first);br_real(r.second);
  BR_TRY(std::pair<gp_Pnt2d,gp_Pnt2d> uv=T::UVPoints(g,c);br_pnt2d(uv.first);br_pnt2d(uv.second));
  BR_TRY(bt_pcurve_adaptor(T::PCurveAdaptor(g,c)));bt_co_edge_usage(s,g,c,l);}
 br_end();}
void bt_face(const BrScene& s,const BRepGraph& g,BRepGraph_FaceId f,const TopLoc_Location& l){
 using T=BRepGraph_Tool::Face;const BRepGraph_Tool::FaceUsage u{f,l,TopAbs_FORWARD};
 br_real(T::Tolerance(g,f));br_flag(T::HasSurface(g,f));BR_TRY(br_int(T::OuterWire(g,f).Index));bg_surface(s,T::Surface(g,f));
 BR_TRY(bt_surface_adaptor(T::SurfaceAdaptor(g,f)));BR_TRY(bt_surface_adaptor(T::SurfaceAdaptor(g,u)));
 BR_TRY(bt_surface_adaptor(T::SurfaceAdaptor(g,f,0.0,1.0,-1.0,2.0)));BR_TRY(bt_surface_adaptor(T::SurfaceAdaptor(g,u,0.0,1.0,-1.0,2.0)));
 br_int(T::NbWires(g,f));double b[4];T::Bounds(g,f,b[0],b[1],b[2],b[3]);bt_bounds(b[0],b[1],b[2],b[3]);}
void bt_faces(const BrScene& s,const BRepGraph& g,const TopLoc_Location& l){
 using T=BRepGraph_Tool::Face;
 for(uint32_t i=0;i<g.Topo().Faces().Nb();++i)bt_face(s,g,BRepGraph_FaceId(i),l);
 for(uint32_t i=0;i<=g.Refs().Faces().Nb();++i){BRepGraph_FaceRefId r(i);BRepGraph_Tool::FaceUsage u=T::Usage(g,r);
  br_flag(u.IsValid());br_int(u.DefId.Index);br_int(u.Orientation);br_real(T::Tolerance(g,r));br_flag(T::HasSurface(g,r));
  BR_TRY(br_int(T::OuterWire(g,r).Index));bg_surface(s,T::Surface(g,r));BR_TRY(bt_surface_adaptor(T::SurfaceAdaptor(g,r)));
  BR_TRY(bt_surface_adaptor(T::SurfaceAdaptor(g,r,0.0,1.0,-1.0,2.0)));br_int(T::NbWires(g,r));
  double b[4]={-1,-1,-1,-1};T::Bounds(g,r,b[0],b[1],b[2],b[3]);bt_bounds(b[0],b[1],b[2],b[3]);}
 br_end();}
void bt_wires_shells(const BRepGraph& g){
 using W=BRepGraph_Tool::Wire;using S=BRepGraph_Tool::Shell;
 for(uint32_t i=0;i<g.Topo().Wires().Nb();++i){BRepGraph_WireId w(i);
  br_flag(W::IsClosed(g,w));br_int(W::NbCoEdges(g,w));br_int(W::NbDistinctEdges(g,w));br_int(W::FaceOf(g,w).Index);BR_TRY(br_flag(W::IsOuter(g,w)));}
 for(uint32_t i=0;i<=g.Refs().Wires().Nb();++i){BRepGraph_WireRefId r(i);BRepGraph_Tool::WireUsage u=W::Usage(g,r);
  br_flag(u.IsValid());br_int(u.DefId.Index);br_int(u.Orientation);br_flag(W::IsClosed(g,r));br_int(W::NbCoEdges(g,r));
  br_int(W::NbDistinctEdges(g,r));br_int(W::FaceOf(g,r).Index);BR_TRY(br_flag(W::IsOuter(g,r)));}
 for(uint32_t i=0;i<g.Topo().Shells().Nb();++i){BRepGraph_ShellId x(i);br_flag(S::IsClosed(g,x));br_int(S::NbFaces(g,x));}
 for(uint32_t i=0;i<=g.Refs().Shells().Nb();++i){BRepGraph_ShellRefId r(i);BRepGraph_Tool::ShellUsage u=S::Usage(g,r);
  br_flag(u.IsValid());br_int(u.DefId.Index);br_int(u.Orientation);br_flag(S::IsClosed(g,r));br_int(S::NbFaces(g,r));}
 br_end();}
void bt_tool(const BrScene& s,const BRepGraph& g){
 const TopLoc_Location& l=s.graph.loc(int(s.graph.locs.size())-1);
 bt_vertices(g,l);bt_edges(s,g);bt_co_edges(s,g,l);bt_faces(s,g,l);bt_wires_shells(g);}
// `br_bgraph_walk script nactions (action)... nedits (edit)...`: the graph, then the
// navigation, explorer and Tool reports.
void br_bgraph_walk_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);BRepGraph g;
 int n=r.integer();for(int i=0;i<n;++i)bg_action(g,s,r);
 n=r.integer();for(int i=0;i<n;++i)bw_edit(s,g,r);br_end();
 bw_dense(g);bw_children(g);bw_reverse(g);bw_related(g);bx_explore(s,g);bt_tool(s,g);}
