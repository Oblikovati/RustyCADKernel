// Geom2dAPI probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom2d_api.rs. Curves are read with ga_curve2d
// (tkgeomalgo_probe_geom2dint.cxx).
#include <Approx_ParametrizationType.hxx>
#include <GeomAbs_Shape.hxx>
#include <Geom2dAPI_ExtremaCurveCurve.hxx>
#include <Geom2dAPI_Interpolate.hxx>
#include <Geom2dAPI_PointsToBSpline.hxx>
#include <Geom2dAPI_ProjectPointOnCurve.hxx>
// A B-spline curve: degree, periodic, rational, poles (and weights), knots and mults.
void ga_bspline_out(const occ::handle<Geom2d_BSplineCurve>& c){
 ga_flag(c.IsNull());if(c.IsNull())return;
 ga_int(c->Degree());ga_flag(c->IsPeriodic());ga_flag(c->IsRational());ga_int(c->NbPoles());
 for(int i=1;i<=c->NbPoles();i++){ga_pnt2d(c->Pole(i));if(c->IsRational())ga_real(c->Weight(i));}
 ga_int(c->NbKnots());for(int i=1;i<=c->NbKnots();i++){ga_real(c->Knot(i));ga_int(c->Multiplicity(i));}
}
// `curve1 curve2 u1min u1max u2min u2max`: every member of Geom2dAPI_ExtremaCurveCurve.
void ga_g2api_extrema_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c1=ga_curve2d(r);occ::handle<Geom2d_Curve> c2=ga_curve2d(r);
  double a=r.real(),b=r.real(),c=r.real(),d=r.real();Geom2dAPI_ExtremaCurveCurve e(c1,c2,a,b,c,d);
  int n=e.NbExtrema();ga_int(n);GA_TRY(ga_int(int(e)));
  for(int i=0;i<=n+1;i++){GA_TRY(gp_Pnt2d p,q;e.Points(i,p,q);ga_pnt2d(p);ga_pnt2d(q));
   GA_TRY(double u=0,v=0;e.Parameters(i,u,v);ga_real(u);ga_real(v));GA_TRY(ga_real(e.Distance(i)));}
  GA_TRY(gp_Pnt2d p,q;e.NearestPoints(p,q);ga_pnt2d(p);ga_pnt2d(q));GA_TRY(double u=0,v=0;e.LowerDistanceParameters(u,v);ga_real(u);ga_real(v));
  GA_TRY(ga_real(e.LowerDistance()));GA_TRY(ga_real(double(e)));
  const Extrema_ExtCC2d& x=e.Extrema();ga_flag(x.IsDone());GA_TRY(ga_flag(x.IsParallel())));
 ga_end();
}
// `curve x y ranged umin usup`: every member of Geom2dAPI_ProjectPointOnCurve (with the
// range when `ranged`).
void ga_g2api_project_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c=ga_curve2d(r);gp_Pnt2d p=r.pnt2d();bool ranged=r.flag();double u0=r.real(),u1=r.real();
  Geom2dAPI_ProjectPointOnCurve e;ga_int(e.NbPoints());if(ranged)e.Init(p,c,u0,u1);else e.Init(p,c);
  int n=e.NbPoints();ga_int(n);ga_int(int(e));
  for(int i=0;i<=n+1;i++){GA_TRY(ga_pnt2d(e.Point(i)));GA_TRY(ga_real(e.Parameter(i)));
   GA_TRY(double u=0;e.Parameter(i,u);ga_real(u));GA_TRY(ga_real(e.Distance(i)));}
  GA_TRY(ga_pnt2d(e.NearestPoint()));GA_TRY(ga_pnt2d(gp_Pnt2d(e)));GA_TRY(ga_real(e.LowerDistanceParameter()));
  GA_TRY(ga_real(e.LowerDistance()));GA_TRY(ga_real(double(e)));ga_flag(e.Extrema().IsDone()));
 ga_end();
}
// `periodic tol n (x y)*n hasparams [np p*np] mode ...`: Geom2dAPI_Interpolate; mode 0 no
// tangent, 1 Load(t0, t1, scale), 2 Load(tangents, flags, scale) with n (tx ty flag); then
// Perform, the curve and the tangent flags as the caller sees them.
void ga_g2api_interpolate_case(std::istream& input){
 GaReader r{input};bool periodic=r.flag();double tol=r.real();int n=r.integer();
 occ::handle<NCollection_HArray1<gp_Pnt2d>> pts=new NCollection_HArray1<gp_Pnt2d>(1,n);
 for(int i=1;i<=n;i++)pts->SetValue(i,r.pnt2d());
 bool hasparams=r.flag();occ::handle<NCollection_HArray1<double>> prm;
 if(hasparams){int np=r.integer();prm=new NCollection_HArray1<double>(1,np);for(int i=1;i<=np;i++)prm->SetValue(i,r.real());}
 int mode=r.integer();
 GA_TRY(std::unique_ptr<Geom2dAPI_Interpolate> it(hasparams?new Geom2dAPI_Interpolate(pts,prm,periodic,tol):new Geom2dAPI_Interpolate(pts,periodic,tol));
  occ::handle<NCollection_HArray1<bool>> flags;
  if(mode==1){gp_Vec2d t0=ga_vec2d(r),t1=ga_vec2d(r);bool scale=r.flag();it->Load(t0,t1,scale);}
  if(mode==2){NCollection_Array1<gp_Vec2d> t(1,n);flags=new NCollection_HArray1<bool>(1,n);
   for(int i=1;i<=n;i++){t(i)=ga_vec2d(r);flags->SetValue(i,r.flag());}bool scale=r.flag();it->Load(t,flags,scale);}
  it->Perform();ga_flag(it->IsDone());GA_TRY(ga_bspline_out(it->Curve()));ga_bspline_out(*it);
  if(!flags.IsNull())for(int i=1;i<=n;i++)ga_flag(flags->Value(i)));
 ga_end();
}
GeomAbs_Shape ga_shape(int s){return GeomAbs_Shape(s);}
// `mode ...`: Geom2dAPI_PointsToBSpline; mode 0 Init(Points, DegMin, DegMax, C, Tol),
// 1 Init(Points, ParType, ...), 2 Init(Points, Params, ...), 3 Init(YValues, X0, DX, ...),
// 4 Init(Points, W1, W2, W3, DegMax, C, Tol).
void ga_g2api_points_to_bspline_case(std::istream& input){
 GaReader r{input};int mode=r.integer();int n=r.integer();
 NCollection_Array1<gp_Pnt2d> pts(1,n);NCollection_Array1<double> ys(1,n),prm(1,n);
 for(int i=1;i<=n;i++){if(mode==3)ys(i)=r.real();else pts(i)=r.pnt2d();}
 if(mode==2)for(int i=1;i<=n;i++)prm(i)=r.real();
 Geom2dAPI_PointsToBSpline f;
 GA_TRY(if(mode==0){int a=r.integer(),b=r.integer();int s=r.integer();double t=r.real();f.Init(pts,a,b,ga_shape(s),t);}
  if(mode==1){int pt=r.integer(),a=r.integer(),b=r.integer();int s=r.integer();double t=r.real();f.Init(pts,Approx_ParametrizationType(pt),a,b,ga_shape(s),t);}
  if(mode==2){int a=r.integer(),b=r.integer();int s=r.integer();double t=r.real();f.Init(pts,prm,a,b,ga_shape(s),t);}
  if(mode==3){double x0=r.real(),dx=r.real();int a=r.integer(),b=r.integer();int s=r.integer();double t=r.real();f.Init(ys,x0,dx,a,b,ga_shape(s),t);}
  if(mode==4){double w1=r.real(),w2=r.real(),w3=r.real();int b=r.integer();int s=r.integer();double t=r.real();f.Init(pts,w1,w2,w3,b,ga_shape(s),t);});
 ga_flag(f.IsDone());GA_TRY(ga_bspline_out(f.Curve()));ga_end();
}
