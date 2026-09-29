// BRep oracle, part 2: the BRep_Tool reports of a scene built by tkbrep_probe_brep.cxx.
// Geometry is printed as registry ids, locations as chains, doubles as bits. The
// planar projection of CurveOnPlane needs ProjLib, which the port does not have: every
// query that returns a projected curve prints `P` instead (the Rust twin prints `P`
// for its Standard_NotImplemented). The Rust twin is
// crates/tkbrep/examples/br_probe/brep_report.rs.
#include <BRep_CurveOnClosedSurface.hxx>
#include <BRep_CurveOnSurface.hxx>
#include <BRep_CurveRepresentation.hxx>
#include <BRep_GCurve.hxx>
#include <BRep_PointRepresentation.hxx>
#include <BRep_TEdge.hxx>
#include <BRep_TFace.hxx>
#include <BRep_TVertex.hxx>
void br_pnt(const gp_Pnt& p){br_real(p.X());br_real(p.Y());br_real(p.Z());}
void br_pnt2d(const gp_Pnt2d& p){br_real(p.X());br_real(p.Y());}
// A curve that is not in the registry: its point at `u`.
void br_curve_value(const occ::handle<Geom_Curve>& c,double u){BR_TRY(br_pnt(c->Value(u)));}
// The accessors of one representation, each an id, a chain or an exception.
void br_representation_geometry(const BrScene& s,const occ::handle<BRep_CurveRepresentation>& c){
 const BrGeometry& G=s.geometry;
 BR_TRY(br_int(br_id(G.curves,c->Curve3D())));BR_TRY(br_int(br_id(G.surfaces,c->Surface())));
 BR_TRY(br_int(br_id(G.pcurves,c->PCurve())));BR_TRY(br_int(br_id(G.pcurves,c->PCurve2())));
 BR_TRY(br_int(br_id(G.polygons3d,c->Polygon3D())));BR_TRY(br_int(br_id(G.polygons2d,c->Polygon())));
 BR_TRY(br_int(br_id(G.polygons2d,c->Polygon2())));BR_TRY(br_int(br_id(G.triangulations,c->Triangulation())));
 BR_TRY(br_int(br_id(G.nodes,c->PolygonOnTriangulation())));BR_TRY(br_int(br_id(G.nodes,c->PolygonOnTriangulation2())));
 BR_TRY(br_int(br_id(G.surfaces,c->Surface2())));BR_TRY(br_chain(s.graph,c->Location2()));BR_TRY(br_int(c->Continuity()));}
void br_representation(const BrScene& s,const occ::handle<BRep_CurveRepresentation>& c){
 br_word(c->DynamicType()->Name());
 bool k[]={c->IsCurve3D(),c->IsCurveOnSurface(),c->IsRegularity(),c->IsCurveOnClosedSurface(),c->IsPolygon3D(),
  c->IsPolygonOnTriangulation(),c->IsPolygonOnClosedTriangulation(),c->IsPolygonOnSurface(),c->IsPolygonOnClosedSurface()};
 for(bool b:k)br_flag(b);
 br_chain(s.graph,c->Location());br_representation_geometry(s,c);
 occ::handle<BRep_GCurve> gc=occ::down_cast<BRep_GCurve>(c);
 if(!gc.IsNull()){br_real(gc->First());br_real(gc->Last());
  // D0 of a null 3D curve dereferences it natively (the port raises Standard_NullObject).
  if(!Precision::IsInfinite(gc->First())&&!(gc->IsCurve3D()&&gc->Curve3D().IsNull())){gp_Pnt p;BR_TRY(gc->D0(gc->First(),p);br_pnt(p));}}
 occ::handle<BRep_CurveOnSurface> cs=occ::down_cast<BRep_CurveOnSurface>(c);
 if(!cs.IsNull()){gp_Pnt2d a,b;cs->UVPoints(a,b);br_pnt2d(a);br_pnt2d(b);}
 occ::handle<BRep_CurveOnClosedSurface> cc=occ::down_cast<BRep_CurveOnClosedSurface>(c);
 if(!cc.IsNull()){gp_Pnt2d a,b;cc->UVPoints2(a,b);br_pnt2d(a);br_pnt2d(b);}}
void br_point_representation(const BrScene& s,const occ::handle<BRep_PointRepresentation>& p){
 const BrGeometry& G=s.geometry;br_word(p->DynamicType()->Name());
 br_flag(p->IsPointOnCurve());br_flag(p->IsPointOnCurveOnSurface());br_flag(p->IsPointOnSurface());
 br_real(p->Parameter());BR_TRY(br_real(p->Parameter2()));br_chain(s.graph,p->Location());
 BR_TRY(br_int(br_id(G.curves,p->Curve())));BR_TRY(br_int(br_id(G.pcurves,p->PCurve())));BR_TRY(br_int(br_id(G.surfaces,p->Surface())));}
void br_vertex_report(const BrScene& s,const TopoDS_Vertex& v){
 BR_TRY(br_pnt(BRep_Tool::Pnt(v)));br_real(BRep_Tool::Tolerance(v));
 const BRep_TVertex* tv=static_cast<const BRep_TVertex*>(v.TShape().get());
 br_real(tv->Tolerance());br_pnt(tv->Pnt());br_int(tv->Points().Extent());
 for(const occ::handle<BRep_PointRepresentation>& p:tv->Points())br_point_representation(s,p);}
// One located curve on surface result: p-curve, surface, chain, range.
void br_cos(const BrScene& s,const occ::handle<Geom2d_Curve>& c,const occ::handle<Geom_Surface>& sf,const TopLoc_Location& l,double f,double la){
 br_int(br_id(s.geometry.pcurves,c));br_int(br_id(s.geometry.surfaces,sf));br_chain(s.graph,l);br_real(f);br_real(la);}
void br_edge_lists(const BrScene& s,const TopoDS_Edge& e){
 const BrGeometry& G=s.geometry;
 {occ::handle<Geom2d_Curve> c;occ::handle<Geom_Surface> sf;TopLoc_Location l;double f=0,la=0;BRep_Tool::CurveOnSurface(e,c,sf,l,f,la);br_cos(s,c,sf,l,f,la);}
 for(int i=0;i<5;++i){occ::handle<Geom2d_Curve> c;occ::handle<Geom_Surface> sf;TopLoc_Location l;double f=0,la=0;
  BRep_Tool::CurveOnSurface(e,c,sf,l,f,la,i);br_cos(s,c,sf,l,f,la);}
 {occ::handle<Poly_Polygon2D> p;occ::handle<Geom_Surface> sf;TopLoc_Location l;BRep_Tool::PolygonOnSurface(e,p,sf,l);
  br_int(br_id(G.polygons2d,p));br_int(br_id(G.surfaces,sf));br_chain(s.graph,l);}
 for(int i=0;i<4;++i){occ::handle<Poly_Polygon2D> p;occ::handle<Geom_Surface> sf;TopLoc_Location l;BRep_Tool::PolygonOnSurface(e,p,sf,l,i);
  br_int(br_id(G.polygons2d,p));br_int(br_id(G.surfaces,sf));br_chain(s.graph,l);}
 {occ::handle<Poly_PolygonOnTriangulation> p;occ::handle<Poly_Triangulation> t;TopLoc_Location l;BRep_Tool::PolygonOnTriangulation(e,p,t,l);
  br_int(br_id(G.nodes,p));br_int(br_id(G.triangulations,t));br_chain(s.graph,l);}
 for(int i=0;i<4;++i){occ::handle<Poly_PolygonOnTriangulation> p;occ::handle<Poly_Triangulation> t;TopLoc_Location l;BRep_Tool::PolygonOnTriangulation(e,p,t,l,i);
  br_int(br_id(G.nodes,p));br_int(br_id(G.triangulations,t));br_chain(s.graph,l);}}
void br_edge_report(const BrScene& s,const TopoDS_Edge& e){
 const BrGeometry& G=s.geometry;const BRep_TEdge* te=static_cast<const BRep_TEdge*>(e.TShape().get());
 br_real(BRep_Tool::Tolerance(e));br_real(te->Tolerance());br_flag(BRep_Tool::SameParameter(e));br_flag(BRep_Tool::SameRange(e));
 br_flag(BRep_Tool::Degenerated(e));br_flag(BRep_Tool::IsGeometric(e));
 {double f=0,l=0;BRep_Tool::Range(e,f,l);br_real(f);br_real(l);}
 {TopLoc_Location l;double f=0,la=0;const occ::handle<Geom_Curve>& c=BRep_Tool::Curve(e,l,f,la);br_int(br_id(G.curves,c));br_chain(s.graph,l);br_real(f);br_real(la);}
 {double f=0,la=0;occ::handle<Geom_Curve> c=BRep_Tool::Curve(e,f,la);int id=br_id(G.curves,c);br_int(id);
  if(id==-2){if(!Precision::IsInfinite(f))br_curve_value(c,f);if(!Precision::IsInfinite(la))br_curve_value(c,la);}}
 {TopLoc_Location l;br_int(br_id(G.polygons3d,BRep_Tool::Polygon3D(e,l)));br_chain(s.graph,l);}
 br_edge_lists(s,e);
 br_flag(BRep_Tool::HasContinuity(e));br_int(BRep_Tool::MaxContinuity(e));
 br_int(te->Curves().Extent());for(const occ::handle<BRep_CurveRepresentation>& c:te->Curves())br_representation(s,c);}
void br_face_report(const BrScene& s,const TopoDS_Face& f){
 const BrGeometry& G=s.geometry;const BRep_TFace* tf=static_cast<const BRep_TFace*>(f.TShape().get());
 {TopLoc_Location l;br_int(br_id(G.surfaces,BRep_Tool::Surface(f,l)));br_chain(s.graph,l);}
 {occ::handle<Geom_Surface> c=BRep_Tool::Surface(f);int id=br_id(G.surfaces,c);br_int(id);if(id==-2)BR_TRY(br_pnt(c->Value(0.25,0.75)));}
 br_real(BRep_Tool::Tolerance(f));br_real(tf->Tolerance());br_flag(BRep_Tool::NaturalRestriction(f));br_flag(BRep_Tool::IsGeometric(f));
 const int purposes[]={0,1,2,4,8,16,17,32,48};
 for(int p:purposes){TopLoc_Location l;br_int(br_id(G.triangulations,BRep_Tool::Triangulation(f,l,Poly_MeshPurpose(p))));br_chain(s.graph,l);}
 {TopLoc_Location l;const NCollection_List<occ::handle<Poly_Triangulation>>& ts=BRep_Tool::Triangulations(f,l);br_int(ts.Extent());
  for(const occ::handle<Poly_Triangulation>& t:ts)br_int(br_id(G.triangulations,t));br_chain(s.graph,l);}
 br_int(tf->NbTriangulations());br_int(br_id(G.triangulations,tf->ActiveTriangulation()));}
// A p-curve query that may project: `P` when the result is a projected curve.
void br_pcurve_result(const BrScene& s,const occ::handle<Geom2d_Curve>& c,double f,double l,bool stored){
 if(!c.IsNull()&&!stored){br_word("P");return;}
 br_int(br_id(s.geometry.pcurves,c));br_real(f);br_real(l);br_flag(stored);}
// "Search the vertex in the edge" of BRep_Tool::Parameter(V, E, S, L).
bool br_vertex_at_end(const TopoDS_Vertex& v,const TopoDS_Edge& e){
 TopoDS_Shape vf;
 for(TopoDS_Iterator it(e.Oriented(TopAbs_FORWARD));it.More();it.Next())
  if(v.IsSame(it.Value())&&(vf.IsNull()||it.Value().Orientation()==v.Orientation()))vf=it.Value();
 return !vf.IsNull()&&(vf.Orientation()==TopAbs_FORWARD||vf.Orientation()==TopAbs_REVERSED);}
// Parameter(V, E, S, L) prints `P` when it reaches the projection (INTERNAL vertex
// and a projected p-curve), else its value or exception.
void br_parameter_on_surface(const TopoDS_Vertex& v,const TopoDS_Edge& e,const occ::handle<Geom_Surface>& sf,const TopLoc_Location& l){
 if(!br_vertex_at_end(v,e)){bool stored=true;double f,la;occ::handle<Geom2d_Curve> c=BRep_Tool::CurveOnSurface(e,sf,l,f,la,&stored);
  if(!c.IsNull()&&!stored){br_word("P");return;}}
 BR_TRY(br_real(BRep_Tool::Parameter(v,e,sf,l)));}
void br_edge_on_surface(const BrScene& s,const TopoDS_Edge& e,const occ::handle<Geom_Surface>& sf,const TopLoc_Location& l){
 const BrGeometry& G=s.geometry;
 {bool stored=true;double f=0,la=0;occ::handle<Geom2d_Curve> c;BR_TRY(c=BRep_Tool::CurveOnSurface(e,sf,l,f,la,&stored);br_pcurve_result(s,c,f,la,stored));}
 {double f=0,la=0;occ::handle<Geom2d_Curve> c;BR_TRY(c=BRep_Tool::CurveOnPlane(e,sf,l,f,la);br_pcurve_result(s,c,f,la,false));}
 br_int(br_id(G.polygons2d,BRep_Tool::PolygonOnSurface(e,sf,l)));br_flag(BRep_Tool::IsClosed(e,sf,l));
 {double f=0,la=0;BRep_Tool::Range(e,sf,l,f,la);br_real(f);br_real(la);}
 {gp_Pnt2d a,b;BR_TRY(BRep_Tool::UVPoints(e,sf,l,a,b);br_pnt2d(a);br_pnt2d(b));}}
void br_edge_on_face(const BrScene& s,const TopoDS_Edge& e,const TopoDS_Face& f){
 const BrGeometry& G=s.geometry;
 {bool stored=true;double a=0,b=0;occ::handle<Geom2d_Curve> c;BR_TRY(c=BRep_Tool::CurveOnSurface(e,f,a,b,&stored);br_pcurve_result(s,c,a,b,stored));}
 br_int(br_id(G.polygons2d,BRep_Tool::PolygonOnSurface(e,f)));br_flag(BRep_Tool::IsClosed(e,f));
 {double a=0,b=0;BRep_Tool::Range(e,f,a,b);br_real(a);br_real(b);}
 {gp_Pnt2d a,b;BR_TRY(BRep_Tool::UVPoints(e,f,a,b);br_pnt2d(a);br_pnt2d(b));}}
// Every shape pair and every registry surface, triangulation and first location.
void br_pairs_report(const BrScene& s){
 const BrGraph& g=s.graph;const BrGeometry& G=s.geometry;int nl=std::min<int>(3,g.locs.size());
 for(const TopoDS_Shape& a:g.shapes){
  if(a.IsNull()||a.ShapeType()!=TopAbs_EDGE)continue;const TopoDS_Edge& e=TopoDS::Edge(a);
  for(int l=0;l<nl;++l){
   for(const occ::handle<Geom_Surface>& sf:G.surfaces)br_edge_on_surface(s,e,sf,g.locs[l]);
   for(const occ::handle<Poly_Triangulation>& t:G.triangulations){
    br_int(br_id(G.nodes,BRep_Tool::PolygonOnTriangulation(e,t,g.locs[l])));br_flag(BRep_Tool::IsClosed(e,t,g.locs[l]));}}
  for(const TopoDS_Shape& b:g.shapes){
   if(b.IsNull()||b.ShapeType()!=TopAbs_FACE)continue;const TopoDS_Face& f=TopoDS::Face(b);br_edge_on_face(s,e,f);
   for(const TopoDS_Shape& c:g.shapes){
    if(c.IsNull()||c.ShapeType()!=TopAbs_FACE)continue;
    br_flag(BRep_Tool::HasContinuity(e,f,TopoDS::Face(c)));br_int(BRep_Tool::Continuity(e,f,TopoDS::Face(c)));}}}
 for(const TopoDS_Shape& a:g.shapes){
  if(a.IsNull()||a.ShapeType()!=TopAbs_VERTEX)continue;const TopoDS_Vertex& v=TopoDS::Vertex(a);
  for(const TopoDS_Shape& b:g.shapes){
   if(b.IsNull())continue;
   if(b.ShapeType()==TopAbs_FACE)BR_TRY(br_pnt2d(BRep_Tool::Parameters(v,TopoDS::Face(b))));
   if(b.ShapeType()!=TopAbs_EDGE)continue;const TopoDS_Edge& e=TopoDS::Edge(b);
   double p=0;bool found=false;BR_TRY(found=BRep_Tool::Parameter(v,e,p);br_flag(found);br_real(found?p:0.0));
   for(const TopoDS_Shape& c:g.shapes){
    if(c.IsNull()||c.ShapeType()!=TopAbs_FACE)continue;TopLoc_Location l;const occ::handle<Geom_Surface>& sf=BRep_Tool::Surface(TopoDS::Face(c),l);
    br_parameter_on_surface(v,e,sf,l);}}}}
void br_shape_report(const BrScene& s,const TopoDS_Shape& sh){
 br_desc(s.graph,sh);if(sh.IsNull())return;
 BR_TRY(br_flag(BRep_Tool::IsClosed(sh)));
 for(int t:{4,6,7})BR_TRY(br_real(BRep_Tool::MaxTolerance(sh,TopAbs_ShapeEnum(t))));
 switch(sh.ShapeType()){
  case TopAbs_VERTEX:br_vertex_report(s,TopoDS::Vertex(sh));break;
  case TopAbs_EDGE:br_edge_report(s,TopoDS::Edge(sh));break;
  case TopAbs_FACE:br_face_report(s,TopoDS::Face(sh));break;
  default:break;}}
// `br_brep script`: every shape report, the pair reports, then the triangulation purposes.
void br_brep_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);
 for(const TopoDS_Shape& sh:s.graph.shapes)br_shape_report(s,sh);
 br_pairs_report(s);
 for(const occ::handle<Poly_Triangulation>& t:s.geometry.triangulations)br_int(t->MeshPurpose());
 br_end();}
// `br_brep_dump script`: the masked JSON dumps of every TShape at four depths.
void br_brep_dump_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);
 for(const TopoDS_Shape& sh:s.graph.shapes){
  if(sh.IsNull()){br_word("N");continue;}
  for(int depth=-1;depth<=2;++depth){std::ostringstream o;sh.TShape()->DumpJson(o,depth);br_text(br_masked(o.str()));}}
 br_end();}
