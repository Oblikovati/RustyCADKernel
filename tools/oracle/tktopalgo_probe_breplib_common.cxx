// The readers and the shape printer of the BRepLib probes (tktopalgo_probe_breplib*.cxx);
// the Rust twin is crates/tktopalgo/examples/tp_probe/brep_lib_common.rs.
//
// Records (every number read through GaReader):
// - vertices: `n (x y z tol)...`, built with BRep_Builder; a vertex reference is an index
//   into them, -1 for a null vertex;
// - gp curve `kind ...`: 0 line P D; 1 circle P N R; 2 ellipse P N R r; 3 hyperbola P N R r;
//   4 parabola P N focal (gp_Ax2(P, N) frames);
// - Geom curve `kind ...`: 0..4 the gp curves as Geom curves; 5 Bezier n (x y z)...;
//   6 B-spline degree periodic n (x y z)... nk (knot mult)...; 7 trimmed u1 u2 sense (curve);
//   8 offset value D (curve);
// - gp 2d curve `kind ...`: 0 line P D; 1 circle P X R sense; 2 ellipse P X R r sense;
//   3 hyperbola P X R r sense; 4 parabola P X focal sense (gp_Ax22d(P, X, sense) frames);
// - Geom2d curve `kind ...`: 0..4 the gp 2d curves as Geom2d curves; 5 Bezier n (u v)...;
//   6 B-spline degree periodic n (u v)... nk (knot mult)...; 7 trimmed u1 u2 sense (curve);
// - surface: the face reader's `tp_surface` record (tktopalgo_probe_face.cxx).
//
// `tp_shape` prints a canonical digest of a shape: its sub-shapes in TopExp::MapShapes
// order, each with its type, orientation, Closed flag, identity location flag and
// children (index and orientation), then the geometry of vertices (point, tolerance),
// edges (tolerance, flags, 3d curve, curves on surfaces, vertex parameters) and faces
// (tolerance, natural restriction, surface).
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Hyperbola.hxx>
#include <Geom_Line.hxx>
#include <Geom_OffsetCurve.hxx>
#include <Geom_Parabola.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Hyperbola.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_Parabola.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Circ.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Elips.hxx>
#include <gp_Elips2d.hxx>
#include <gp_Hypr.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Lin.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Parab.hxx>
#include <gp_Parab2d.hxx>
#include <Precision.hxx>
#include <TopExp.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopTools_ShapeMapHasher.hxx>
// The dynamic type name of a handle, `null` for a null one.
void tp_type(const occ::handle<Standard_Transient>& t){ga_text(t.IsNull()?"null":t->DynamicType()->Name());}
// `n (x y z tol)...`: vertices built with BRep_Builder.
std::vector<TopoDS_Vertex> tp_vertices(GaReader& r){
 std::vector<TopoDS_Vertex> vertices(r.integer());BRep_Builder B;
 for(TopoDS_Vertex& v:vertices){gp_Pnt p=r.pnt();B.MakeVertex(v,p,r.real());}
 return vertices;}
// A vertex reference: -1 is a null vertex.
TopoDS_Vertex tp_vertex_ref(GaReader& r,const std::vector<TopoDS_Vertex>& vertices){
 int i=r.integer();return i<0?TopoDS_Vertex():vertices.at(i);}
gp_Ax2 tp_ax2(GaReader& r){gp_Pnt p=r.pnt();return gp_Ax2(p,r.dir());}
gp_Ax22d tp_ax22d(GaReader& r){gp_Pnt2d p=r.pnt2d();gp_Dir2d x=r.dir2d();return gp_Ax22d(p,x,true);}
gp_Lin tp_lin(GaReader& r){gp_Pnt p=r.pnt();return gp_Lin(p,r.dir());}
gp_Circ tp_circ(GaReader& r){gp_Ax2 a=tp_ax2(r);return gp_Circ(a,r.real());}
gp_Elips tp_elips(GaReader& r){gp_Ax2 a=tp_ax2(r);double major=r.real();return gp_Elips(a,major,r.real());}
gp_Hypr tp_hypr(GaReader& r){gp_Ax2 a=tp_ax2(r);double major=r.real();return gp_Hypr(a,major,r.real());}
gp_Parab tp_parab(GaReader& r){gp_Ax2 a=tp_ax2(r);return gp_Parab(a,r.real());}
// The frame of a 2d conic, with its sense read after the radii.
gp_Ax22d tp_sensed_ax22d(GaReader& r,const gp_Pnt2d& p,const gp_Dir2d& x){return gp_Ax22d(p,x,r.flag());}
gp_Lin2d tp_lin2d(GaReader& r){gp_Pnt2d p=r.pnt2d();return gp_Lin2d(p,r.dir2d());}
gp_Circ2d tp_circ2d(GaReader& r){gp_Pnt2d p=r.pnt2d();gp_Dir2d x=r.dir2d();double radius=r.real();return gp_Circ2d(tp_sensed_ax22d(r,p,x),radius);}
gp_Elips2d tp_elips2d(GaReader& r){gp_Pnt2d p=r.pnt2d();gp_Dir2d x=r.dir2d();double major=r.real(),minor=r.real();return gp_Elips2d(tp_sensed_ax22d(r,p,x),major,minor);}
gp_Hypr2d tp_hypr2d(GaReader& r){gp_Pnt2d p=r.pnt2d();gp_Dir2d x=r.dir2d();double major=r.real(),minor=r.real();return gp_Hypr2d(tp_sensed_ax22d(r,p,x),major,minor);}
gp_Parab2d tp_parab2d(GaReader& r){gp_Pnt2d p=r.pnt2d();gp_Dir2d x=r.dir2d();double focal=r.real();return gp_Parab2d(tp_sensed_ax22d(r,p,x),focal);}
// `nk (knot mult)...`.
void tp_curve_knots(GaReader& r,NCollection_Array1<double>& knots,NCollection_Array1<int>& mults){
 int n=r.integer();knots.Resize(1,n,false);mults.Resize(1,n,false);
 for(int i=1;i<=n;++i){knots(i)=r.real();mults(i)=r.integer();}}
occ::handle<Geom_Curve> tp_geom_curve(GaReader& r);
occ::handle<Geom_Curve> tp_spline_curve(GaReader& r,int kind){
 if(kind==5){NCollection_Array1<gp_Pnt> poles(1,r.integer());for(gp_Pnt& p:poles)p=r.pnt();return new Geom_BezierCurve(poles);}
 int degree=r.integer();bool periodic=r.flag();NCollection_Array1<gp_Pnt> poles(1,r.integer());for(gp_Pnt& p:poles)p=r.pnt();
 NCollection_Array1<double> knots(1,1);NCollection_Array1<int> mults(1,1);tp_curve_knots(r,knots,mults);
 return new Geom_BSplineCurve(poles,knots,mults,degree,periodic);}
occ::handle<Geom_Curve> tp_geom_curve(GaReader& r){
 int kind=r.integer();
 if(kind==0)return new Geom_Line(tp_lin(r));
 if(kind==1)return new Geom_Circle(tp_circ(r));
 if(kind==2)return new Geom_Ellipse(tp_elips(r));
 if(kind==3)return new Geom_Hyperbola(tp_hypr(r));
 if(kind==4)return new Geom_Parabola(tp_parab(r));
 if(kind==7){double u1=r.real(),u2=r.real();bool sense=r.flag();return new Geom_TrimmedCurve(tp_geom_curve(r),u1,u2,sense);}
 if(kind==8){double offset=r.real();gp_Dir d=r.dir();return new Geom_OffsetCurve(tp_geom_curve(r),offset,d);}
 return tp_spline_curve(r,kind);}
occ::handle<Geom2d_Curve> tp_geom2d_curve(GaReader& r);
occ::handle<Geom2d_Curve> tp_spline_curve2d(GaReader& r,int kind){
 if(kind==5){NCollection_Array1<gp_Pnt2d> poles(1,r.integer());for(gp_Pnt2d& p:poles)p=r.pnt2d();return new Geom2d_BezierCurve(poles);}
 int degree=r.integer();bool periodic=r.flag();NCollection_Array1<gp_Pnt2d> poles(1,r.integer());for(gp_Pnt2d& p:poles)p=r.pnt2d();
 NCollection_Array1<double> knots(1,1);NCollection_Array1<int> mults(1,1);tp_curve_knots(r,knots,mults);
 return new Geom2d_BSplineCurve(poles,knots,mults,degree,periodic);}
occ::handle<Geom2d_Curve> tp_geom2d_curve(GaReader& r){
 int kind=r.integer();
 if(kind==0)return new Geom2d_Line(tp_lin2d(r));
 if(kind==1)return new Geom2d_Circle(tp_circ2d(r));
 if(kind==2)return new Geom2d_Ellipse(tp_elips2d(r));
 if(kind==3)return new Geom2d_Hyperbola(tp_hypr2d(r));
 if(kind==4)return new Geom2d_Parabola(tp_parab2d(r));
 if(kind==7){double u1=r.real(),u2=r.real();bool sense=r.flag();return new Geom2d_TrimmedCurve(tp_geom2d_curve(r),u1,u2,sense);}
 return tp_spline_curve2d(r,kind);}
// The points of a curve at its finite ends and, on a finite range, at its middle.
template<class C> void tp_curve_points(const C& c,double f,double l){
 bool finf=Precision::IsInfinite(f),linf=Precision::IsInfinite(l);
 if(!finf)GA_TRY(ga_pnt(c->Value(f)))
 if(!linf)GA_TRY(ga_pnt(c->Value(l)))
 if(!finf&&!linf)GA_TRY(ga_pnt(c->Value(0.5*(f+l))))}
template<class C> void tp_curve2d_points(const C& c,double f,double l){
 bool finf=Precision::IsInfinite(f),linf=Precision::IsInfinite(l);
 if(!finf)GA_TRY(ga_pnt2d(c->Value(f)))
 if(!linf)GA_TRY(ga_pnt2d(c->Value(l)))
 if(!finf&&!linf)GA_TRY(ga_pnt2d(c->Value(0.5*(f+l))))}
// The 3d curve of an edge (BRep_Tool::Curve with its location applied) and its range.
void tp_edge_curve3d(const TopoDS_Edge& e){
 double f=0.,l=0.;occ::handle<Geom_Curve> c=BRep_Tool::Curve(e,f,l);tp_type(c);
 if(c.IsNull())return;ga_real(f);ga_real(l);tp_curve_points(c,f,l);}
// Every curve on surface of an edge (both of a seam), then their count.
void tp_edge_pcurves(const TopoDS_Edge& e){
 int i=1;
 for(;;++i){occ::handle<Geom2d_Curve> c;occ::handle<Geom_Surface> s;TopLoc_Location L;double f=0.,l=0.;
  BRep_Tool::CurveOnSurface(e,c,s,L,f,l,i);if(c.IsNull())break;
  tp_type(s);tp_type(c);ga_flag(L.IsIdentity());ga_real(f);ga_real(l);tp_curve2d_points(c,f,l);}
 ga_int(i-1);}
void tp_edge_geometry(const TopoDS_Edge& e){
 ga_real(BRep_Tool::Tolerance(e));ga_flag(BRep_Tool::SameParameter(e));ga_flag(BRep_Tool::SameRange(e));
 ga_flag(BRep_Tool::Degenerated(e));tp_edge_curve3d(e);tp_edge_pcurves(e);
 for(TopoDS_Iterator it(e);it.More();it.Next())GA_TRY(ga_real(BRep_Tool::Parameter(TopoDS::Vertex(it.Value()),e)))}
void tp_face_geometry(const TopoDS_Face& f){
 ga_real(BRep_Tool::Tolerance(f));ga_flag(BRep_Tool::NaturalRestriction(f));
 occ::handle<Geom_Surface> s=BRep_Tool::Surface(f);tp_type(s);
 if(!s.IsNull())GA_TRY(ga_pnt(s->Value(0.25,0.5)))}
void tp_sub_shape(const TopoDS_Shape& sh,const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& m){
 ga_int(sh.ShapeType());ga_int(sh.Orientation());ga_flag(sh.Closed());ga_flag(sh.Location().IsIdentity());
 ga_int(sh.NbChildren());
 for(TopoDS_Iterator it(sh);it.More();it.Next()){ga_int(m.FindIndex(it.Value()));ga_int(it.Value().Orientation());}
 if(sh.ShapeType()==TopAbs_VERTEX){ga_pnt(BRep_Tool::Pnt(TopoDS::Vertex(sh)));ga_real(BRep_Tool::Tolerance(TopoDS::Vertex(sh)));}
 if(sh.ShapeType()==TopAbs_EDGE)tp_edge_geometry(TopoDS::Edge(sh));
 if(sh.ShapeType()==TopAbs_FACE)tp_face_geometry(TopoDS::Face(sh));}
// The digest of a shape; -1 for a null shape.
void tp_shape(const TopoDS_Shape& s){
 if(s.IsNull()){ga_int(-1);return;}
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> m;TopExp::MapShapes(s,m);
 ga_int(m.Extent());ga_int(s.Orientation());
 for(int i=1;i<=m.Extent();++i)tp_sub_shape(m(i),m);}
