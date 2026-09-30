// IntPatch tool probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_patch_tools.rs: IntPatch_HInterTool,
// IntPatch_HCurve2dTool, IntPatch_ArcFunction, IntPatch_CSFunction with
// IntPatch_CurvIntSurf, and IntPatch_PointLine::CurvatureRadiusOfIntersLine. Needs the
// readers of tkgeomalgo_probe_geom2dint.cxx (ga_curve2d), tkgeomalgo_probe_intcurvesurface.cxx
// (ga_ics_surface), tkgeomalgo_probe_intsurf.cxx (ga_intsurf_quadric) and
// tkgeomalgo_probe_intpatch.cxx.
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <IntPatch_ArcFunction.hxx>
#include <IntPatch_CSFunction.hxx>
#include <IntPatch_CurvIntSurf.hxx>
#include <IntPatch_HCurve2dTool.hxx>
#include <IntPatch_HInterTool.hxx>
#include <IntPatch_PointLine.hxx>
// An arc `curve2d bounded [u0 u1]`.
occ::handle<Adaptor2d_Curve2d> ga_ip_arc(GaReader& r){
 occ::handle<Geom2d_Curve> c=ga_curve2d(r);if(!r.flag())return new Geom2dAdaptor_Curve(c);
 double a=r.real();return new Geom2dAdaptor_Curve(c,a,r.real());}
void ga_ip_v2d(const gp_Vec2d& v){ga_real(v.X());ga_real(v.Y());}
// `surface narcs arcs x y`: the sampling of the surface and the arc members of the tool
// (on the arcs, the fixture vertex 1 and the 2d point (x, y)).
void ga_ip_hinter_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);typedef IntPatch_HInterTool T;
  ga_flag(T::SingularOnUMin(s));ga_flag(T::SingularOnUMax(s));ga_flag(T::SingularOnVMin(s));ga_flag(T::SingularOnVMax(s));
  GA_TRY(ga_int(T::NbSamplesU(s,0.,1.)));GA_TRY(ga_int(T::NbSamplesV(s,0.,1.)));
  GA_TRY(T tool;int n=tool.NbSamplePoints(s);ga_int(n);for(int i=0;i<=n+1;++i){double u,v;tool.SamplePoint(s,i,u,v);ga_real(u);ga_real(v);});
  int na=r.integer();std::vector<occ::handle<Adaptor2d_Curve2d>> arcs;for(int i=0;i<na;++i)arcs.push_back(ga_ip_arc(r));
  arcs.push_back(fx.arc(1));gp_Pnt2d p=r.pnt2d();
  for(const occ::handle<Adaptor2d_Curve2d>& a:arcs){
   GA_TRY(ga_int(T::NbSamplesOnArc(a)));GA_TRY(double f,l;T::Bounds(a,f,l);ga_real(f);ga_real(l));
   GA_TRY(double prm=0;gp_Pnt2d q;bool ok=T::Project(a,p,prm,q);ga_flag(ok);if(ok){ga_real(prm);ga_pnt2d(q);});
   GA_TRY(ga_real(T::Tolerance(fx.vertex(1),a)));GA_TRY(ga_real(T::Parameter(fx.vertex(1),a)));
   ga_flag(T::HasBeenSeen(a));ga_int(T::NbPoints(a));GA_TRY(gp_Pnt x;double t,u;T::Value(a,1,x,t,u));ga_flag(T::IsVertex(a,1));
   GA_TRY(occ::handle<Adaptor3d_HVertex> v;T::Vertex(a,1,v));ga_int(T::NbSegments(a));GA_TRY(int k;T::HasFirstPoint(a,1,k));
   GA_TRY(int k;T::HasLastPoint(a,1,k));ga_flag(T::IsAllSolution(a));});
 ga_end();
}
// `arc u u0 u1`: every member of IntPatch_HCurve2dTool on the arc.
void ga_ip_hc2d_case(std::istream& input){
 GaReader r{input};typedef IntPatch_HCurve2dTool T;
 GA_TRY(occ::handle<Adaptor2d_Curve2d> a=ga_ip_arc(r);double u=r.real(),u0=r.real(),u1=r.real();
  ga_real(T::FirstParameter(a));ga_real(T::LastParameter(a));ga_int(T::Continuity(a));
  for(int k=0;k<=3;++k)GA_TRY(int n=T::NbIntervals(a,GeomAbs_Shape(k));ga_int(n);NCollection_Array1<double> t(1,n+1);
   T::Intervals(a,t,GeomAbs_Shape(k));for(int i=1;i<=n+1;++i)ga_real(t(i)));
  GA_TRY(ga_flag(T::IsClosed(a)));GA_TRY(ga_flag(T::IsPeriodic(a)));GA_TRY(ga_real(T::Period(a)));
  GA_TRY(ga_pnt2d(T::Value(a,u)));GA_TRY(gp_Pnt2d p;T::D0(a,u,p);ga_pnt2d(p));GA_TRY(gp_Pnt2d p;gp_Vec2d v;T::D1(a,u,p,v);ga_pnt2d(p);ga_ip_v2d(v));
  GA_TRY(gp_Pnt2d p;gp_Vec2d v,w;T::D2(a,u,p,v,w);ga_pnt2d(p);ga_ip_v2d(v);ga_ip_v2d(w));
  GA_TRY(gp_Pnt2d p;gp_Vec2d v,w,x;T::D3(a,u,p,v,w,x);ga_pnt2d(p);ga_ip_v2d(v);ga_ip_v2d(w);ga_ip_v2d(x));
  for(int n=1;n<=3;++n)GA_TRY(ga_ip_v2d(T::DN(a,u,n)));
  GA_TRY(ga_real(T::Resolution(a,1.e-3)));ga_int(T::GetType(a));
  GA_TRY(gp_Lin2d x=T::Line(a);ga_pnt2d(x.Location()));GA_TRY(ga_real(T::Circle(a).Radius()));
  GA_TRY(ga_real(T::Ellipse(a).MinorRadius()));GA_TRY(ga_real(T::Hyperbola(a).MajorRadius()));
  GA_TRY(ga_real(T::Parabola(a).Focal()));GA_TRY(occ::handle<Geom2d_BezierCurve> b=T::Bezier(a);ga_flag(b.IsNull());if(!b.IsNull())ga_int(b->NbPoles()));
  GA_TRY(occ::handle<Geom2d_BSplineCurve> b=T::BSpline(a);ga_flag(b.IsNull());if(!b.IsNull())ga_int(b->NbKnots()));
  GA_TRY(ga_int(T::NbSamples(a,u0,u1))));
 ga_end();
}
// `surface quadric arc nx xs`: IntPatch_ArcFunction along the arc at each x, then its
// saved points.
void ga_ip_arcf_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);IntSurf_Quadric q=ga_intsurf_quadric(r);occ::handle<Adaptor2d_Curve2d> a=ga_ip_arc(r);
  IntPatch_ArcFunction f;f.SetQuadric(q);f.Set(s);f.Set(a);ga_int(f.Quadric().TypeQuadric());ga_flag(f.Arc()==a);ga_flag(f.Surface()==s);
  GA_TRY(ga_int(f.NbSamples()));int n=r.integer();
  for(int i=0;i<n;++i){double x=r.real();
   GA_TRY(double v=0;ga_flag(f.Value(x,v));ga_real(v);ga_pnt(f.LastComputedPoint()));
   GA_TRY(double d=0;ga_flag(f.Derivative(x,d));ga_real(d);ga_pnt(f.LastComputedPoint()));
   GA_TRY(double v=0,d=0;ga_flag(f.Values(x,v,d));ga_real(v);ga_real(d));ga_int(f.GetStateNumber());}
  for(int i=0;i<=n+1;++i)GA_TRY(ga_pnt(f.Valpoint(i)));
  f.Set(a);GA_TRY(f.Valpoint(1)));
 ga_end();
}
// `s1 arc s2 nx (u v w)* u v w tol margin`: IntPatch_CSFunction at each point, then
// IntPatch_CurvIntSurf from (u, v, w).
void ga_ip_csf_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);occ::handle<Adaptor2d_Curve2d> a=ga_ip_arc(r);
  occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);IntPatch_CSFunction f(s1,a,s2);
  ga_int(f.NbVariables());ga_int(f.NbEquations());ga_flag(f.AuxillarSurface()==s1);ga_flag(f.AuxillarCurve()==a);
  ga_pnt(f.Point());ga_real(f.Root());int n=r.integer();
  for(int i=0;i<n;++i){math_Vector x(1,3),fx(1,3);math_Matrix d(1,3,1,3);x(1)=r.real();x(2)=r.real();x(3)=r.real();
   GA_TRY(ga_flag(f.Value(x,fx));for(int k=1;k<=3;++k)ga_real(fx(k));ga_pnt(f.Point());ga_real(f.Root()));
   GA_TRY(ga_flag(f.Derivatives(x,d));for(int k=1;k<=3;++k)for(int j=1;j<=3;++j)ga_real(d(k,j)));
   GA_TRY(ga_flag(f.Values(x,fx,d));for(int k=1;k<=3;++k)ga_real(fx(k));for(int k=1;k<=3;++k)for(int j=1;j<=3;++j)ga_real(d(k,j)));}
  double u=r.real(),v=r.real(),w=r.real(),tol=r.real(),margin=r.real();
  GA_TRY(IntPatch_CurvIntSurf e(u,v,w,f,tol,margin);ga_flag(e.IsDone());GA_TRY(ga_flag(e.IsEmpty()));GA_TRY(ga_pnt(e.Point()));
   GA_TRY(double p,q;e.ParameterOnSurface(p,q);ga_real(p);ga_real(q));GA_TRY(ga_real(e.ParameterOnCurve()));ga_real(e.Function().Root()));
  IntPatch_CurvIntSurf e2(f,tol);ga_flag(e2.IsDone());GA_TRY(ga_flag(e2.IsEmpty()));GA_TRY(ga_pnt(e2.Point())));
 ga_end();
}
// `s1 s2 n (x y z u1 v1 u2 v2)*`: the curvature radius of the intersection line at each point.
void ga_ip_curvature_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r),s2=ga_ics_surface(r);int n=r.integer();
  for(int i=0;i<n;++i){gp_Pnt x=r.pnt();double a=r.real(),b=r.real(),c=r.real(),d=r.real();IntSurf_PntOn2S p;p.SetValue(x,a,b,c,d);
   GA_TRY(ga_real(IntPatch_PointLine::CurvatureRadiusOfIntersLine(s1,s2,p)));});
 ga_end();
}
