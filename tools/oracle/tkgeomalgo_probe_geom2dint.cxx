// Geom2dInt probe operations and the 2D curve reader shared with the Geom2dAPI probe
// (tkgeomalgo_probe.cxx); the Rust twin is crates/tkgeomalgo/examples/ga_probe/geom2d_int.rs
// (curve reader: ga_probe/curve2d.rs).
//
// A 2D curve is read recursively as in tools/oracle/tkgeombase_probe_extrema_pc2d.cxx:
// 0 line P D | 1 circle P XD sense R | 2 ellipse P XD sense A B | 3 hyperbola P XD sense A B |
// 4 parabola P XD sense F | 5 bezier rational n (x y w)*n | 6 bspline rational periodic degree
// nk (k m)*nk np (x y w)*np | 7 trimmed u1 u2 sense adjust <curve> | 8 offset d notcheck <curve>.
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Hyperbola.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_Parabola.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom2dInt_Geom2dCurveTool.hxx>
#include <Geom2dInt_IntConicCurveOfGInter.hxx>
#include <Geom2dInt_TheIntConicCurveOfGInter.hxx>
#include <Geom2dInt_TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter.hxx>
#include <Geom2dInt_MyImpParToolOfTheIntersectorOfTheIntConicCurveOfGInter.hxx>
#include <Geom2dInt_TheProjPCurOfGInter.hxx>
#include <Standard_ProgramError.hxx>
#include <math_Matrix.hxx>
#include <math_Vector.hxx>
#include "tkgeomalgo_probe_curve2d.cxx"
typedef Geom2dInt_Geom2dCurveTool GaTool;
void ga_v2d(const gp_Vec2d& v){ga_real(v.X());ga_real(v.Y());}
// `curve u0 u1 u eps`: every member of Geom2dInt_Geom2dCurveTool on the adaptor of the curve.
void ga_g2int_tool_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c=ga_curve2d(r);double u0=r.real(),u1=r.real(),u=r.real(),eps=r.real();
  Geom2dAdaptor_Curve a(c);ga_int(GaTool::GetType(a));GA_TRY(ga_int(GaTool::NbSamples(a)));GA_TRY(ga_int(GaTool::NbSamples(a,u0,u1)));
  ga_real(GaTool::EpsX(a));GA_TRY(ga_real(GaTool::EpsX(a,eps)));ga_real(GaTool::FirstParameter(a));ga_real(GaTool::LastParameter(a));
  GA_TRY(ga_pnt2d(GaTool::Value(a,u)));GA_TRY(gp_Pnt2d p;GaTool::D0(a,u,p);ga_pnt2d(p));
  GA_TRY(gp_Pnt2d p;gp_Vec2d t;GaTool::D1(a,u,p,t);ga_pnt2d(p);ga_v2d(t));
  GA_TRY(gp_Pnt2d p;gp_Vec2d t,n;GaTool::D2(a,u,p,t,n);ga_pnt2d(p);ga_v2d(t);ga_v2d(n));
  GA_TRY(gp_Pnt2d p;gp_Vec2d t,n,v;GaTool::D3(a,u,p,t,n,v);ga_pnt2d(p);ga_v2d(t);ga_v2d(n);ga_v2d(v));
  GA_TRY(ga_v2d(GaTool::DN(a,u,2)));
  GA_TRY(int nb=GaTool::NbIntervals(a);ga_int(nb);NCollection_Array1<double> tab(1,nb+1);GaTool::Intervals(a,tab);
   for(int i=1;i<=nb+1;i++)ga_real(tab(i));double x=0,y=0;GaTool::GetInterval(a,1,tab,x,y);ga_real(x);ga_real(y));
  GA_TRY(ga_int(GaTool::Degree(a)));
  GA_TRY(gp_Lin2d l=GaTool::Line(a);ga_pnt2d(l.Location()));GA_TRY(ga_real(GaTool::Circle(a).Radius()));
  GA_TRY(ga_real(GaTool::Ellipse(a).MinorRadius()));GA_TRY(ga_real(GaTool::Parabola(a).Focal()));
  GA_TRY(ga_real(GaTool::Hyperbola(a).MajorRadius())));
 ga_end();
}
// `curve x y low high`: both Geom2dInt_TheProjPCurOfGInter::FindParameter.
void ga_g2int_proj_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c=ga_curve2d(r);gp_Pnt2d p=r.pnt2d();double low=r.real(),high=r.real();
  Geom2dAdaptor_Curve a(c);GA_TRY(ga_real(Geom2dInt_TheProjPCurOfGInter::FindParameter(a,p,1e-7)));
  GA_TRY(ga_real(Geom2dInt_TheProjPCurOfGInter::FindParameter(a,p,low,high,1e-7))));
 ga_end();
}
// `conic domain curve domain tolconf tol reversed which`: Geom2dInt_IntConicCurveOfGInter
// (which 0) or Geom2dInt_TheIntConicCurveOfGInter (which 1); the curve domain points are on
// the curve.
void ga_g2int_conic_curve_case(std::istream& input){
 GaReader r{input};
 GA_TRY(GaConic a=ga_read_conic(r);GaDomainSpec sa=ga_read_domain(r);occ::handle<Geom2d_Curve> c=ga_curve2d(r);
  GaDomainSpec sb=ga_read_domain(r);double tc=r.real(),t=r.real();bool rev=r.flag();int which=r.integer();
  Geom2dAdaptor_Curve ac(c);IntRes2d_Domain da=ga_conic_domain(sa,a);IntRes2d_Domain db=ga_domain(sb,[&](double u){return ac.Value(u);});
  if(which==0){Geom2dInt_IntConicCurveOfGInter i;i.SetReversedParameters(rev);
   switch(a.kind){case 0:i.Perform(a.lin(),da,ac,db,tc,t);break;case 1:i.Perform(a.circ(),da,ac,db,tc,t);break;
    case 2:i.Perform(a.elips(),da,ac,db,tc,t);break;case 3:i.Perform(a.parab(),da,ac,db,tc,t);break;default:i.Perform(a.hypr(),da,ac,db,tc,t);}
   ga_inter(i);}
  else{Geom2dInt_TheIntConicCurveOfGInter i;i.SetReversedParameters(rev);
   switch(a.kind){case 0:i.Perform(a.lin(),da,ac,db,tc,t);break;case 1:i.Perform(a.circ(),da,ac,db,tc,t);break;
    case 2:i.Perform(a.elips(),da,ac,db,tc,t);break;case 3:i.Perform(a.parab(),da,ac,db,tc,t);break;default:i.Perform(a.hypr(),da,ac,db,tc,t);}
   ga_inter(i);});
 ga_end();
}
// `curve1 curve2 x1 x2 conic u`: TheDistBetweenPCurves at (x1, x2), then the implicit-parametric
// function of the conic along curve1 at u.
void ga_g2int_functions_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c1=ga_curve2d(r);occ::handle<Geom2d_Curve> c2=ga_curve2d(r);
  double x1=r.real(),x2=r.real();GaConic k=ga_read_conic(r);double u=r.real();
  Geom2dAdaptor_Curve a1(c1),a2(c2);Geom2dInt_TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter f(a1,a2);
  math_Vector x(1,2);x(1)=x1;x(2)=x2;math_Vector v(1,2,-3.);math_Matrix d(1,2,1,2,-4.);
  ga_int(f.NbVariables());ga_int(f.NbEquations());
  GA_TRY(ga_flag(f.Value(x,v));ga_real(v(1));ga_real(v(2)));
  GA_TRY(ga_flag(f.Derivatives(x,d));for(int i=1;i<=2;i++)for(int j=1;j<=2;j++)ga_real(d(i,j)));
  GA_TRY(ga_flag(f.Values(x,v,d));ga_real(v(1));ga_real(v(2));for(int i=1;i<=2;i++)for(int j=1;j<=2;j++)ga_real(d(i,j)));
  Geom2dInt_MyImpParToolOfTheIntersectorOfTheIntConicCurveOfGInter g(ga_itool(k),a1);double fv=-1.,fd=-2.;
  GA_TRY(ga_flag(g.Value(u,fv));ga_real(fv));GA_TRY(ga_flag(g.Derivative(u,fd));ga_real(fd));
  GA_TRY(fv=-1.;fd=-2.;ga_flag(g.Values(u,fv,fd));ga_real(fv);ga_real(fd)));
 ga_end();
}
