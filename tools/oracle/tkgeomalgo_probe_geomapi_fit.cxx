// GeomAPI probe operations over the plane conversions, the curve-surface intersection,
// the interpolation and the approximation (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom_api_fit.rs. 3D curves and surfaces are read
// with ga_ics_geom_curve and ga_ics_geom_surface, 2D curves with ga_curve2d.
#include <GeomAPI.hxx>
#include <GeomAPI_IntCS.hxx>
#include <GeomAPI_Interpolate.hxx>
#include <GeomAPI_PointsToBSpline.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <Precision.hxx>
#include <gp_Pln.hxx>
// The parameter of sample k (0..4): uniform on a finite range, -1..1 otherwise.
double ga_gapi_sample(double f,double l,int k){
 if(!Precision::IsInfinite(f)&&!Precision::IsInfinite(l))return f+(l-f)*k/4.0;return -1.0+0.5*k;}
// A 3D curve: null flag, type, bounds, periodicity and five points.
void ga_gapi_curve_out(const occ::handle<Geom_Curve>& c){
 ga_flag(c.IsNull());if(c.IsNull())return;GeomAdaptor_Curve a(c);ga_int(int(a.GetType()));
 double f=c->FirstParameter(),l=c->LastParameter();ga_real(f);ga_real(l);ga_flag(c->IsPeriodic());
 for(int k=0;k<=4;k++)GA_TRY(ga_pnt(c->Value(ga_gapi_sample(f,l,k))));
}
// A 2D curve: null flag, type, bounds, periodicity and five points.
void ga_gapi_curve2d_out(const occ::handle<Geom2d_Curve>& c){
 ga_flag(c.IsNull());if(c.IsNull())return;Geom2dAdaptor_Curve a(c);ga_int(int(a.GetType()));
 double f=c->FirstParameter(),l=c->LastParameter();ga_real(f);ga_real(l);ga_flag(c->IsPeriodic());
 for(int k=0;k<=4;k++)GA_TRY(ga_pnt2d(c->Value(ga_gapi_sample(f,l,k))));
}
gp_Pln ga_gapi_plane(GaReader& r){gp_Pnt p=r.pnt();return gp_Pln(p,r.dir());}
// `curve px py pz dx dy dz`: GeomAPI::To2d.
void ga_gapi_to2d_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);gp_Pln p=ga_gapi_plane(r);ga_gapi_curve2d_out(GeomAPI::To2d(c,p)));
 ga_end();
}
// `curve2d px py pz dx dy dz`: GeomAPI::To3d.
void ga_gapi_to3d_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c=ga_curve2d(r);gp_Pln p=ga_gapi_plane(r);ga_gapi_curve_out(GeomAPI::To3d(c,p)));
 ga_end();
}
// `curve surface mode`: GeomAPI_IntCS (mode 0 constructor, 1 default + Perform, 2 default),
// every point and segment with indices 0..N+1.
void ga_gapi_intcs_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);occ::handle<Geom_Surface> s=ga_ics_geom_surface(r);int mode=r.integer();
  std::unique_ptr<GeomAPI_IntCS> e(mode==0?new GeomAPI_IntCS(c,s):new GeomAPI_IntCS());if(mode==1)e->Perform(c,s);
  ga_flag(e->IsDone());int n=0,m=0;GA_TRY(n=e->NbPoints();ga_int(n));
  for(int i=0;i<=n+1;i++){GA_TRY(ga_pnt(e->Point(i)));GA_TRY(double u=0,v=0,w=0;e->Parameters(i,u,v,w);ga_real(u);ga_real(v);ga_real(w));}
  GA_TRY(m=e->NbSegments();ga_int(m));
  for(int i=0;i<=m+1;i++){GA_TRY(ga_gapi_curve_out(e->Segment(i)));
   GA_TRY(double a=0,b=0,d=0,f=0;e->Parameters(i,a,b,d,f);ga_real(a);ga_real(b);ga_real(d);ga_real(f));});
 ga_end();
}
// A 3D B-spline curve: degree, periodic, rational, poles (and weights), knots and mults.
void ga_gapi_bspline_out(const occ::handle<Geom_BSplineCurve>& c){
 ga_flag(c.IsNull());if(c.IsNull())return;
 ga_int(c->Degree());ga_flag(c->IsPeriodic());ga_flag(c->IsRational());ga_int(c->NbPoles());
 for(int i=1;i<=c->NbPoles();i++){ga_pnt(c->Pole(i));if(c->IsRational())ga_real(c->Weight(i));}
 ga_int(c->NbKnots());for(int i=1;i<=c->NbKnots();i++){ga_real(c->Knot(i));ga_int(c->Multiplicity(i));}
}
gp_Vec ga_gapi_vec(GaReader& r){double x=r.real(),y=r.real();return gp_Vec(x,y,r.real());}
// `periodic tol n (x y z)*n hasparams [np p*np] mode ...`: GeomAPI_Interpolate; mode 0 no
// tangent, 1 Load(t0, t1, scale), 2 Load(tangents, flags, scale) with n (tx ty tz flag);
// then Perform, the curve and the tangent flags as the caller sees them.
void ga_gapi_interpolate_case(std::istream& input){
 GaReader r{input};bool periodic=r.flag();double tol=r.real();int n=r.integer();
 occ::handle<NCollection_HArray1<gp_Pnt>> pts=new NCollection_HArray1<gp_Pnt>(1,n);
 for(int i=1;i<=n;i++)pts->SetValue(i,r.pnt());
 bool hasparams=r.flag();occ::handle<NCollection_HArray1<double>> prm;
 if(hasparams){int np=r.integer();prm=new NCollection_HArray1<double>(1,np);for(int i=1;i<=np;i++)prm->SetValue(i,r.real());}
 int mode=r.integer();
 GA_TRY(std::unique_ptr<GeomAPI_Interpolate> it(hasparams?new GeomAPI_Interpolate(pts,prm,periodic,tol):new GeomAPI_Interpolate(pts,periodic,tol));
  occ::handle<NCollection_HArray1<bool>> flags;
  if(mode==1){gp_Vec t0=ga_gapi_vec(r),t1=ga_gapi_vec(r);bool scale=r.flag();it->Load(t0,t1,scale);}
  if(mode==2){NCollection_Array1<gp_Vec> t(1,n);flags=new NCollection_HArray1<bool>(1,n);
   for(int i=1;i<=n;i++){t(i)=ga_gapi_vec(r);flags->SetValue(i,r.flag());}bool scale=r.flag();it->Load(t,flags,scale);}
  it->Perform();ga_flag(it->IsDone());GA_TRY(ga_gapi_bspline_out(it->Curve()));ga_gapi_bspline_out(*it);
  if(!flags.IsNull())for(int i=1;i<=n;i++)ga_flag(flags->Value(i)));
 ga_end();
}
// `mode n (x y z)*n [np params] args`: GeomAPI_PointsToBSpline; mode 0 Init(Points, DegMin,
// DegMax, C, Tol), 1 Init(Points, ParType, ...), 2 Init(Points, Params, ...), 4
// Init(Points, W1, W2, W3, DegMax, C, Tol); 5-8 the constructors of modes 0, 1, 2 and 4.
void ga_gapi_points_to_bspline_case(std::istream& input){
 GaReader r{input};int mode=r.integer();int n=r.integer();
 NCollection_Array1<gp_Pnt> pts(1,n);for(int i=1;i<=n;i++)pts(i)=r.pnt();
 int base=mode>=5?(mode==8?4:mode-5):mode;int np=base==2?r.integer():1;
 NCollection_Array1<double> prm(1,np);if(base==2)for(int i=1;i<=np;i++)prm(i)=r.real();
 std::unique_ptr<GeomAPI_PointsToBSpline> f(new GeomAPI_PointsToBSpline());
 GA_TRY(if(base==0){int a=r.integer(),b=r.integer();int s=r.integer();double t=r.real();
   if(mode<5)f->Init(pts,a,b,ga_shape(s),t);else f.reset(new GeomAPI_PointsToBSpline(pts,a,b,ga_shape(s),t));}
  if(base==1){int pt=r.integer(),a=r.integer(),b=r.integer();int s=r.integer();double t=r.real();
   if(mode<5)f->Init(pts,Approx_ParametrizationType(pt),a,b,ga_shape(s),t);
   else f.reset(new GeomAPI_PointsToBSpline(pts,Approx_ParametrizationType(pt),a,b,ga_shape(s),t));}
  if(base==2){int a=r.integer(),b=r.integer();int s=r.integer();double t=r.real();
   if(mode<5)f->Init(pts,prm,a,b,ga_shape(s),t);else f.reset(new GeomAPI_PointsToBSpline(pts,prm,a,b,ga_shape(s),t));}
  if(base==4){double w1=r.real(),w2=r.real(),w3=r.real();int b=r.integer();int s=r.integer();double t=r.real();
   if(mode<5)f->Init(pts,w1,w2,w3,b,ga_shape(s),t);else f.reset(new GeomAPI_PointsToBSpline(pts,w1,w2,w3,b,ga_shape(s),t));});
 ga_flag(f->IsDone());GA_TRY(ga_gapi_bspline_out(f->Curve()));ga_gapi_bspline_out(*f);ga_end();
}
