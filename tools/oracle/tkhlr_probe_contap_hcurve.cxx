// Every Contap_HCurve2dTool member with original adaptor inputs.
#include <Contap_HCurve2dTool.hxx>
#include "tkgeomalgo_probe_curve2d.cxx"
occ::handle<Adaptor2d_Curve2d> ct_arc(GaReader& r){
 occ::handle<Geom2d_Curve> c=ga_curve2d(r);if(!r.flag())return new Geom2dAdaptor_Curve(c);
 double a=r.real();return new Geom2dAdaptor_Curve(c,a,r.real());}
void ct_v2d(const gp_Vec2d& v){ga_real(v.X());ga_real(v.Y());}
void ct_hcurve_case(std::istream& input){
 GaReader r{input};typedef Contap_HCurve2dTool T;
 GA_TRY(occ::handle<Adaptor2d_Curve2d> a=ct_arc(r);double u=r.real(),u0=r.real(),u1=r.real();
  ga_real(T::FirstParameter(a));ga_real(T::LastParameter(a));ga_int(T::Continuity(a));
  for(int k=0;k<=3;++k)GA_TRY(int n=T::NbIntervals(a,GeomAbs_Shape(k));ga_int(n);NCollection_Array1<double> t(1,n+1);
   T::Intervals(a,t,GeomAbs_Shape(k));for(int i=1;i<=n+1;++i)ga_real(t(i)));
  GA_TRY(ga_flag(T::IsClosed(a)));GA_TRY(ga_flag(T::IsPeriodic(a)));GA_TRY(ga_real(T::Period(a)));
  GA_TRY(ga_pnt2d(T::Value(a,u)));GA_TRY(gp_Pnt2d p;T::D0(a,u,p);ga_pnt2d(p));GA_TRY(gp_Pnt2d p;gp_Vec2d v;T::D1(a,u,p,v);ga_pnt2d(p);ct_v2d(v));
  GA_TRY(gp_Pnt2d p;gp_Vec2d v,w;T::D2(a,u,p,v,w);ga_pnt2d(p);ct_v2d(v);ct_v2d(w));
  GA_TRY(gp_Pnt2d p;gp_Vec2d v,w,x;T::D3(a,u,p,v,w,x);ga_pnt2d(p);ct_v2d(v);ct_v2d(w);ct_v2d(x));
  for(int n=1;n<=3;++n)GA_TRY(ct_v2d(T::DN(a,u,n)));
  GA_TRY(ga_real(T::Resolution(a,1.e-3)));ga_int(T::GetType(a));
  GA_TRY(gp_Lin2d x=T::Line(a);ga_pnt2d(x.Location()));GA_TRY(ga_real(T::Circle(a).Radius()));
  GA_TRY(ga_real(T::Ellipse(a).MinorRadius()));GA_TRY(ga_real(T::Hyperbola(a).MajorRadius()));
  GA_TRY(ga_real(T::Parabola(a).Focal()));GA_TRY(occ::handle<Geom2d_BezierCurve> b=T::Bezier(a);ga_flag(b.IsNull());if(!b.IsNull())ga_int(b->NbPoles()));
  GA_TRY(occ::handle<Geom2d_BSplineCurve> b=T::BSpline(a);ga_flag(b.IsNull());if(!b.IsNull())ga_int(b->NbKnots()));
  GA_TRY(ga_int(T::NbSamples(a,u0,u1))));
 ga_end();
}
