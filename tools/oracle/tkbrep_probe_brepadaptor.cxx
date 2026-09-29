// BRepAdaptor and BRepLProp oracle over a scene built by tkbrep_probe_brep.cxx: the
// adaptors of every edge (alone and on each of the faces holding it), face and wire,
// their evaluations at five parameters, the local properties and the junction
// continuity of consecutive wire edges. Face and edge pairs are taken from the faces'
// own edges. The Rust twin is crates/tkbrep/examples/br_probe/brep_adaptor.rs.
#include <BRepAdaptor_CompCurve.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Curve2d.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepLProp.hxx>
#include <BRepLProp_CLProps.hxx>
#include <BRepLProp_SLProps.hxx>
#include <BRepLProp_SurfaceTool.hxx>
void ba_vec(const gp_Vec& v){br_real(v.X());br_real(v.Y());br_real(v.Z());}
void ba_xy(double x,double y){br_real(x);br_real(y);}
// The five sample parameters of a finite range, none for an infinite one.
std::vector<double> ba_samples(double f,double l){
 if(Precision::IsInfinite(f)||Precision::IsInfinite(l))return {};
 std::vector<double> u;for(int i=0;i<=4;++i)u.push_back(f+(l-f)*i/4.0);return u;}
// The members shared by every 3D curve adaptor.
void ba_curve3d(const Adaptor3d_Curve& c){
 double f=0,l=0;BR_TRY(f=c.FirstParameter();l=c.LastParameter();br_real(f);br_real(l));
 BR_TRY(br_int(c.GetType()));BR_TRY(br_int(c.Continuity()));BR_TRY(br_flag(c.IsClosed()));
 BR_TRY(br_flag(c.IsPeriodic()));BR_TRY(br_real(c.Period()));BR_TRY(br_real(c.Resolution(1e-3)));
 for(int s=0;s<=4;++s)BR_TRY(br_int(c.NbIntervals(GeomAbs_Shape(s))));
 BR_TRY(int n=c.NbIntervals(GeomAbs_C2);NCollection_Array1<double> t(1,n+1);c.Intervals(t,GeomAbs_C2);for(double x:t)br_real(x));
 BR_TRY(br_int(c.Degree()));BR_TRY(br_flag(c.IsRational()));BR_TRY(br_int(c.NbPoles()));BR_TRY(br_int(c.NbKnots()));
 BR_TRY(br_pnt(c.Line().Location()));BR_TRY(br_real(c.Circle().Radius()));BR_TRY(br_real(c.Ellipse().MajorRadius()));
 BR_TRY(br_real(c.Hyperbola().MajorRadius()));BR_TRY(br_real(c.Parabola().Focal()));
 // Bezier() and BSpline() return a null handle for another type in a release build.
 BR_TRY(if(c.GetType()==GeomAbs_BezierCurve)br_int(c.Bezier()->NbPoles());if(c.GetType()==GeomAbs_BSplineCurve)br_int(c.BSpline()->NbPoles()));
 for(double u:ba_samples(f,l)){
  BR_TRY(br_pnt(c.Value(u)));BR_TRY(gp_Pnt p;gp_Vec v;c.D1(u,p,v);br_pnt(p);ba_vec(v));
  BR_TRY(gp_Pnt p;gp_Vec v1,v2;c.D2(u,p,v1,v2);ba_vec(v2));BR_TRY(ba_vec(c.DN(u,3)));}
 std::vector<double> s=ba_samples(f,l);if(s.empty())return;
 BR_TRY(occ::handle<Adaptor3d_Curve> t=c.Trim(s[1],s[3],1e-7);br_real(t->FirstParameter());br_real(t->LastParameter());br_pnt(t->Value(s[2])));
 BR_TRY(occ::handle<Adaptor3d_Curve> t=c.ShallowCopy();br_real(t->FirstParameter());br_pnt(t->Value(s[2])));}
void ba_curve2d(const Adaptor2d_Curve2d& c){
 double f=0,l=0;BR_TRY(f=c.FirstParameter();l=c.LastParameter();br_real(f);br_real(l));
 BR_TRY(br_int(c.GetType()));BR_TRY(br_int(c.Continuity()));BR_TRY(br_flag(c.IsClosed()));BR_TRY(br_flag(c.IsPeriodic()));
 BR_TRY(br_real(c.Resolution(1e-3)));BR_TRY(br_int(c.NbIntervals(GeomAbs_C1)));BR_TRY(br_int(c.NbSamples()));
 for(double u:ba_samples(f,l)){BR_TRY(gp_Pnt2d p=c.Value(u);ba_xy(p.X(),p.Y()));BR_TRY(gp_Pnt2d p;gp_Vec2d v;c.D1(u,p,v);ba_xy(v.X(),v.Y()));}
 std::vector<double> s=ba_samples(f,l);if(s.empty())return;
 BR_TRY(occ::handle<Adaptor2d_Curve2d> t=c.Trim(s[1],s[3],1e-7);br_real(t->FirstParameter());gp_Pnt2d p=t->Value(s[2]);ba_xy(p.X(),p.Y()));
 BR_TRY(occ::handle<Adaptor2d_Curve2d> t=c.ShallowCopy();br_real(t->LastParameter()));}
// The curve local properties at the samples.
void ba_clprops(const BRepAdaptor_Curve& c){
 double f=0,l=0;BR_TRY(f=c.FirstParameter();l=c.LastParameter());
 for(double u:ba_samples(f,l)){BR_TRY(BRepLProp_CLProps p(c,u,2,1e-7);br_pnt(p.Value());bool t=p.IsTangentDefined();br_flag(t);
  if(t){gp_Dir d;p.Tangent(d);ba_vec(gp_Vec(d));}BR_TRY(br_real(p.Curvature()));
  BR_TRY(gp_Dir n;p.Normal(n);ba_vec(gp_Vec(n)));BR_TRY(gp_Pnt o;p.CentreOfCurvature(o);br_pnt(o)));}}
void ba_edge(const BrGraph& g,const TopoDS_Edge& e){
 BR_TRY(BRepAdaptor_Curve c(e);br_real(c.Tolerance());br_flag(c.Is3DCurve());br_flag(c.IsCurveOnSurface());br_desc(g,c.Edge());
  ba_curve3d(c);ba_clprops(c));}
void ba_edge_on_face(const BrGraph& g,const TopoDS_Edge& e,const TopoDS_Face& f){
 BR_TRY(BRepAdaptor_Curve c(e,f);br_flag(c.Is3DCurve());br_flag(c.IsCurveOnSurface());ba_curve3d(c));
 BR_TRY(BRepAdaptor_Curve2d c(e,f);br_desc(g,c.Edge());br_desc(g,c.Face());ba_curve2d(c));}
void ba_surface_props(const BRepAdaptor_Surface& s,double u,double v){
 BR_TRY(BRepLProp_SLProps p(s,u,v,2,1e-7);br_pnt(p.Value());bool n=p.IsNormalDefined();br_flag(n);if(n)ba_vec(gp_Vec(p.Normal()));
  bool k=p.IsCurvatureDefined();br_flag(k);
  if(k){br_real(p.MaxCurvature());br_real(p.MinCurvature());br_real(p.MeanCurvature());br_real(p.GaussianCurvature());br_flag(p.IsUmbilic());});
 BR_TRY(gp_Pnt p;BRepLProp_SurfaceTool::Value(s,u,v,p);br_pnt(p));
 BR_TRY(gp_Pnt p;gp_Vec a,b,c,d,e;BRepLProp_SurfaceTool::D2(s,u,v,p,a,b,c,d,e);ba_vec(e));}
void ba_surface(const BRepAdaptor_Surface& s){
 double u1=0,u2=0,v1=0,v2=0;BR_TRY(u1=s.FirstUParameter();u2=s.LastUParameter();v1=s.FirstVParameter();v2=s.LastVParameter();
  br_real(u1);br_real(u2);br_real(v1);br_real(v2));
 BR_TRY(br_int(s.GetType()));BR_TRY(br_int(s.UContinuity()));BR_TRY(br_int(s.VContinuity()));
 BR_TRY(br_flag(s.IsUClosed()));BR_TRY(br_flag(s.IsVClosed()));BR_TRY(br_flag(s.IsUPeriodic()));BR_TRY(br_flag(s.IsVPeriodic()));
 BR_TRY(br_int(s.NbUIntervals(GeomAbs_C1)));BR_TRY(br_int(s.NbVIntervals(GeomAbs_C1)));
 BR_TRY(br_real(s.UResolution(1e-3)));BR_TRY(br_real(s.VResolution(1e-3)));BR_TRY(br_real(s.Tolerance()));
 BR_TRY(br_real(s.Cylinder().Radius()));BR_TRY(br_int(BRepLProp_SurfaceTool::Continuity(s)));
 BR_TRY(double a,b,c,d;BRepLProp_SurfaceTool::Bounds(s,a,b,c,d);br_real(a);br_real(b);br_real(c);br_real(d));
 std::vector<double> us=ba_samples(u1,u2),vs=ba_samples(v1,v2);
 for(size_t i=0;i<us.size()&&i<vs.size();++i){
  BR_TRY(gp_Pnt p;gp_Vec a,b;s.D1(us[i],vs[i],p,a,b);br_pnt(p);ba_vec(a);ba_vec(b));
  BR_TRY(ba_vec(s.DN(us[i],vs[i],1,1)));ba_surface_props(s,us[i],vs[i]);}}
void ba_face(const BrGraph& g,const TopoDS_Face& f){
 for(bool restriction:{true,false})BR_TRY(BRepAdaptor_Surface s(f,restriction);br_desc(g,s.Face());ba_surface(s));
 for(TopExp_Explorer e(f,TopAbs_EDGE);e.More();e.Next())ba_edge_on_face(g,TopoDS::Edge(e.Current()),f);}
// A composite curve: the adaptor, then the edge holding each sample parameter.
void ba_comp(const BrGraph& g,const BRepAdaptor_CompCurve& c){
 br_desc(g,c.Wire());ba_curve3d(c);
 double f=0,l=0;BR_TRY(f=c.FirstParameter();l=c.LastParameter());
 for(double u:ba_samples(f,l)){TopoDS_Edge e;double w=0;BR_TRY(c.Edge(u,e,w);br_desc(g,e);br_real(w));}}
// The junction continuity of each pair of consecutive wire edges.
void ba_junctions(const TopoDS_Wire& w){
 std::vector<TopoDS_Edge> edges;for(BRepTools_WireExplorer x(w);x.More();x.Next())edges.push_back(x.Current());
 for(size_t i=0;i+1<edges.size();++i){
  BR_TRY(BRepAdaptor_Curve a(edges[i]);BRepAdaptor_Curve b(edges[i+1]);
   double u1=edges[i].Orientation()==TopAbs_REVERSED?a.FirstParameter():a.LastParameter();
   double u2=edges[i+1].Orientation()==TopAbs_REVERSED?b.LastParameter():b.FirstParameter();
   BR_TRY(br_int(BRepLProp::Continuity(a,b,u1,u2)));BR_TRY(br_int(BRepLProp::Continuity(a,b,u1,u2,1e-7,1e-3))));}}
void ba_wire(const BrGraph& g,const TopoDS_Wire& w){
 for(bool ac:{false,true})BR_TRY(BRepAdaptor_CompCurve c(w,ac);ba_comp(g,c));
 BR_TRY(BRepAdaptor_CompCurve c(w,false);double f=c.FirstParameter(),l=c.LastParameter();
  if(!Precision::IsInfinite(f)&&!Precision::IsInfinite(l)){BRepAdaptor_CompCurve t(w,false,f+(l-f)/8,l-(l-f)/8,1e-7);ba_comp(g,t);});
 BR_TRY(ba_junctions(w));}
// `br_adaptor script`: every edge and face with geometry (an adaptor dereferences a
// missing curve or surface natively), and every wire of the scene.
void br_adaptor_case(std::istream& in){
 BrReader r{in};BrScene s=br_scene(r);const BrGraph& g=s.graph;
 for(const TopoDS_Shape& sh:g.shapes){
  if(sh.IsNull())continue;
  switch(sh.ShapeType()){
   case TopAbs_EDGE:if(BRep_Tool::IsGeometric(TopoDS::Edge(sh)))ba_edge(g,TopoDS::Edge(sh));break;
   case TopAbs_FACE:if(BRep_Tool::IsGeometric(TopoDS::Face(sh)))ba_face(g,TopoDS::Face(sh));break;
   case TopAbs_WIRE:ba_wire(g,TopoDS::Wire(sh));break;
   default:break;}}
 BRepAdaptor_Curve none;br_desc(g,none.Edge());
 br_end();}
