// GeomAPI_PointsToBSplineSurface probe operation (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom_api_surface.rs.
#include <GeomAPI_PointsToBSplineSurface.hxx>
#include <Geom_BSplineSurface.hxx>
// A B-spline surface: degrees, periodicity, rationality, poles (and weights), knots and
// multiplicities in both directions.
void ga_gapi_bsurface_out(const occ::handle<Geom_BSplineSurface>& s){
 ga_flag(s.IsNull());if(s.IsNull())return;
 ga_int(s->UDegree());ga_int(s->VDegree());ga_flag(s->IsUPeriodic());ga_flag(s->IsVPeriodic());
 bool rat=s->IsURational()||s->IsVRational();ga_flag(rat);ga_int(s->NbUPoles());ga_int(s->NbVPoles());
 for(int i=1;i<=s->NbUPoles();i++)for(int j=1;j<=s->NbVPoles();j++){ga_pnt(s->Pole(i,j));if(rat)ga_real(s->Weight(i,j));}
 ga_int(s->NbUKnots());for(int i=1;i<=s->NbUKnots();i++){ga_real(s->UKnot(i));ga_int(s->UMultiplicity(i));}
 ga_int(s->NbVKnots());for(int i=1;i<=s->NbVKnots();i++){ga_real(s->VKnot(i));ga_int(s->VMultiplicity(i));}
}
// `mode rl ru cl cu values args`: GeomAPI_PointsToBSplineSurface on a grid of points (x y z
// per node, row by row) or of heights (modes 3, 6 and 10); mode 0 Init(P, DegMin, DegMax, C,
// Tol), 1 Init(P, ParType, DegMin, DegMax, C, Tol, periodic), 2 Init(P, W1, W2, W3, DegMax,
// C, Tol), 3 Init(Z, X0, dX, Y0, dY, DegMin, DegMax, C, Tol), 4 Interpolate(P, periodic), 5
// Interpolate(P, ParType, periodic), 6 Interpolate(Z, X0, dX, Y0, dY), 7-10 the constructors
// of modes 0, 1 (not periodic), 2 and 3.
void ga_gapi_p2bss_case(std::istream& input){
 GaReader r{input};int mode=r.integer();int rl=r.integer(),ru=r.integer(),cl=r.integer(),cu=r.integer();
 bool heights=mode==3||mode==6||mode==10;
 NCollection_Array2<gp_Pnt> pts(rl,ru,cl,cu);NCollection_Array2<double> zs(rl,ru,cl,cu);
 for(int i=rl;i<=ru;i++)for(int j=cl;j<=cu;j++){if(heights)zs(i,j)=r.real();else pts(i,j)=r.pnt();}
 std::unique_ptr<GeomAPI_PointsToBSplineSurface> f(new GeomAPI_PointsToBSplineSurface());
 GA_TRY(
  if(mode==0||mode==7){int a=r.integer(),b=r.integer(),s=r.integer();double t=r.real();
   if(mode==0)f->Init(pts,a,b,ga_shape(s),t);else f.reset(new GeomAPI_PointsToBSplineSurface(pts,a,b,ga_shape(s),t));}
  if(mode==1||mode==8){Approx_ParametrizationType pt=Approx_ParametrizationType(r.integer());int a=r.integer(),b=r.integer(),s=r.integer();
   double t=r.real();bool per=mode==1&&r.flag();
   if(mode==1)f->Init(pts,pt,a,b,ga_shape(s),t,per);else f.reset(new GeomAPI_PointsToBSplineSurface(pts,pt,a,b,ga_shape(s),t));}
  if(mode==2||mode==9){double w1=r.real(),w2=r.real(),w3=r.real();int b=r.integer(),s=r.integer();double t=r.real();
   if(mode==2)f->Init(pts,w1,w2,w3,b,ga_shape(s),t);else f.reset(new GeomAPI_PointsToBSplineSurface(pts,w1,w2,w3,b,ga_shape(s),t));}
  if(mode==3||mode==6||mode==10){double x0=r.real(),dx=r.real(),y0=r.real(),dy=r.real();
   if(mode==6)f->Interpolate(zs,x0,dx,y0,dy);
   else{int a=r.integer(),b=r.integer(),s=r.integer();double t=r.real();
    if(mode==3)f->Init(zs,x0,dx,y0,dy,a,b,ga_shape(s),t);else f.reset(new GeomAPI_PointsToBSplineSurface(zs,x0,dx,y0,dy,a,b,ga_shape(s),t));}}
  if(mode==4){bool per=r.flag();f->Interpolate(pts,per);}
  if(mode==5){Approx_ParametrizationType pt=Approx_ParametrizationType(r.integer());bool per=r.flag();f->Interpolate(pts,pt,per);});
 ga_flag(f->IsDone());GA_TRY(ga_gapi_bsurface_out(f->Surface()));GA_TRY(occ::handle<Geom_BSplineSurface> s=*f;ga_flag(s.IsNull()));
 ga_end();
}
