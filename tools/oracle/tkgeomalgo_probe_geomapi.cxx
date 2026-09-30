// GeomAPI probe operations over the projections and the extrema (tkgeomalgo_probe.cxx);
// the Rust twin is crates/tkgeomalgo/examples/ga_probe/geom_api.rs. Curves and surfaces
// are read with ga_ics_geom_curve and ga_ics_geom_surface
// (tkgeomalgo_probe_intcurvesurface.cxx).
#include <GeomAPI_ExtremaCurveCurve.hxx>
#include <GeomAPI_ExtremaCurveSurface.hxx>
#include <GeomAPI_ExtremaSurfaceSurface.hxx>
#include <GeomAPI_ProjectPointOnCurve.hxx>
#include <GeomAPI_ProjectPointOnSurf.hxx>
#include <Extrema_ExtCC.hxx>
#include <Extrema_ExtCS.hxx>
#include <Extrema_ExtPC.hxx>
#include <Extrema_ExtPS.hxx>
#include <Extrema_ExtSS.hxx>
// Every accessor of a GeomAPI_ProjectPointOnCurve, indices 0..NbPoints+1.
void ga_gapi_ppc_print(const GeomAPI_ProjectPointOnCurve& e){
 int n=e.NbPoints();ga_int(n);ga_int(int(e));
 for(int i=0;i<=n+1;i++){GA_TRY(ga_pnt(e.Point(i)));GA_TRY(ga_real(e.Parameter(i)));
  GA_TRY(double u=0;e.Parameter(i,u);ga_real(u));GA_TRY(ga_real(e.Distance(i)));}
 GA_TRY(ga_pnt(e.NearestPoint()));GA_TRY(ga_pnt(gp_Pnt(e)));GA_TRY(ga_real(e.LowerDistanceParameter()));
 GA_TRY(ga_real(e.LowerDistance()));GA_TRY(ga_real(double(e)));
 const Extrema_ExtPC& x=e.Extrema();ga_flag(x.IsDone());GA_TRY(ga_int(x.NbExt()));
}
// `curve mode x y z u0 u1 qx qy qz`: GeomAPI_ProjectPointOnCurve; mode 0 (P, C), 1 (P, C,
// u0, u1), 2 Init(C, u0, u1) + Perform(P), 3 Init(P, C) + Perform(Q), 4 default, 5
// Init(C, u0, u1) only.
void ga_gapi_ppc_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);int mode=r.integer();gp_Pnt p=r.pnt();
  double u0=r.real(),u1=r.real();gp_Pnt q=r.pnt();
  std::unique_ptr<GeomAPI_ProjectPointOnCurve> e;
  if(mode==0)e.reset(new GeomAPI_ProjectPointOnCurve(p,c));
  else if(mode==1)e.reset(new GeomAPI_ProjectPointOnCurve(p,c,u0,u1));
  else{e.reset(new GeomAPI_ProjectPointOnCurve());
   if(mode==2){e->Init(c,u0,u1);e->Perform(p);}if(mode==3){e->Init(p,c);e->Perform(q);}if(mode==5)e->Init(c,u0,u1);}
  ga_gapi_ppc_print(*e));
 ga_end();
}
// Every accessor of a GeomAPI_ProjectPointOnSurf, indices 0..NbPoints+1.
void ga_gapi_pps_print(const GeomAPI_ProjectPointOnSurf& e){
 ga_flag(e.IsDone());int n=e.NbPoints();ga_int(n);ga_int(int(e));
 for(int i=0;i<=n+1;i++){GA_TRY(ga_pnt(e.Point(i)));GA_TRY(double u=0,v=0;e.Parameters(i,u,v);ga_real(u);ga_real(v));
  GA_TRY(ga_real(e.Distance(i)));}
 GA_TRY(ga_pnt(e.NearestPoint()));GA_TRY(ga_pnt(gp_Pnt(e)));
 GA_TRY(double u=0,v=0;e.LowerDistanceParameters(u,v);ga_real(u);ga_real(v));
 GA_TRY(ga_real(e.LowerDistance()));GA_TRY(ga_real(double(e)));
 const Extrema_ExtPS& x=e.Extrema();ga_flag(x.IsDone());GA_TRY(ga_int(x.NbExt()));
}
// `surface ctor algo tol u1 u2 v1 v2 flag n (x y z a)*n`: GeomAPI_ProjectPointOnSurf built by
// constructor or Init `ctor` (0-3 the constructors (P1, S, [tol,] [bounds,] algo), 4-5
// Init(S, bounds, [tol,] algo), 6-9 Init(P1, S, ...), 10 default), SetExtremaFlag(flag)
// when flag >= 0, then for each point after the first (every point for 4-5) an optional
// SetExtremaAlgo(a) (a >= 0) and Perform; the accessors after each step.
void ga_gapi_pps_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom_Surface> s=ga_ics_geom_surface(r);int ctor=r.integer();Extrema_ExtAlgo algo=Extrema_ExtAlgo(r.integer());
  double tol=r.real(),u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();int fl=r.integer();int n=r.integer();
  std::vector<gp_Pnt> pts;std::vector<int> algos;for(int i=0;i<n;i++){pts.push_back(r.pnt());algos.push_back(r.integer());}
  gp_Pnt p1=n>0?pts[0]:gp_Pnt();std::unique_ptr<GeomAPI_ProjectPointOnSurf> e;
  if(ctor==0)e.reset(new GeomAPI_ProjectPointOnSurf(p1,s,algo));
  else if(ctor==1)e.reset(new GeomAPI_ProjectPointOnSurf(p1,s,tol,algo));
  else if(ctor==2)e.reset(new GeomAPI_ProjectPointOnSurf(p1,s,u1,u2,v1,v2,algo));
  else if(ctor==3)e.reset(new GeomAPI_ProjectPointOnSurf(p1,s,u1,u2,v1,v2,tol,algo));
  else{e.reset(new GeomAPI_ProjectPointOnSurf());
   if(ctor==4)e->Init(s,u1,u2,v1,v2,algo);if(ctor==5)e->Init(s,u1,u2,v1,v2,tol,algo);
   if(ctor==6)e->Init(p1,s,algo);if(ctor==7)e->Init(p1,s,tol,algo);
   if(ctor==8)e->Init(p1,s,u1,u2,v1,v2,algo);if(ctor==9)e->Init(p1,s,u1,u2,v1,v2,tol,algo);}
  if(fl>=0)e->SetExtremaFlag(Extrema_ExtFlag(fl));
  ga_gapi_pps_print(*e);
  int first=(ctor==4||ctor==5)?0:1;if(ctor==10)first=n;
  for(int i=first;i<n;i++){if(algos[i]>=0)e->SetExtremaAlgo(Extrema_ExtAlgo(algos[i]));e->Perform(pts[i]);ga_gapi_pps_print(*e);});
 ga_end();
}
// The members of GeomAPI_ExtremaCurveCurve, then the Total members.
void ga_gapi_ecc_print(GeomAPI_ExtremaCurveCurve& e){
 int n=e.NbExtrema();ga_int(n);GA_TRY(ga_int(int(e)));
 for(int i=0;i<=n+1;i++){GA_TRY(gp_Pnt p,q;e.Points(i,p,q);ga_pnt(p);ga_pnt(q));
  GA_TRY(double u=0,v=0;e.Parameters(i,u,v);ga_real(u);ga_real(v));GA_TRY(ga_real(e.Distance(i)));}
 GA_TRY(ga_flag(e.IsParallel()));GA_TRY(gp_Pnt p,q;e.NearestPoints(p,q);ga_pnt(p);ga_pnt(q));
 GA_TRY(double u=0,v=0;e.LowerDistanceParameters(u,v);ga_real(u);ga_real(v));
 GA_TRY(ga_real(e.LowerDistance()));GA_TRY(ga_real(double(e)));
 const Extrema_ExtCC& x=e.Extrema();ga_flag(x.IsDone());GA_TRY(ga_flag(x.IsParallel()));GA_TRY(ga_int(x.NbExt()));
}
void ga_gapi_ecc_total(GeomAPI_ExtremaCurveCurve& e,int order){
 if(order==1)GA_TRY(ga_real(e.TotalLowerDistance()));
 GA_TRY(gp_Pnt p,q;bool ok=e.TotalNearestPoints(p,q);ga_flag(ok);if(ok){ga_pnt(p);ga_pnt(q);});
 GA_TRY(double u=0,v=0;bool ok=e.TotalLowerDistanceParameters(u,v);ga_flag(ok);if(ok){ga_real(u);ga_real(v);});
 GA_TRY(ga_real(e.TotalLowerDistance()));
}
// `count (c1 c2 ranged u1 u2 v1 v2)*count order`: GeomAPI_ExtremaCurveCurve built on the
// first pair (constructor), then Init on each other pair of the same object (the Extrema
// member keeps state across Inits); count 0 is the default object. The Total members are
// called in the native order, or with TotalLowerDistance first when order is 1.
void ga_gapi_ecc_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int count=r.integer();std::unique_ptr<GeomAPI_ExtremaCurveCurve> e;
  for(int k=0;k<count;k++){occ::handle<Geom_Curve> c1=ga_ics_geom_curve(r),c2=ga_ics_geom_curve(r);bool ranged=r.flag();
   double a=r.real(),b=r.real(),c=r.real(),d=r.real();
   if(k==0&&ranged)e.reset(new GeomAPI_ExtremaCurveCurve(c1,c2,a,b,c,d));else if(k==0)e.reset(new GeomAPI_ExtremaCurveCurve(c1,c2));
   else if(ranged)e->Init(c1,c2,a,b,c,d);else e->Init(c1,c2);}
  if(count==0)e.reset(new GeomAPI_ExtremaCurveCurve());int order=r.integer();
  ga_gapi_ecc_print(*e);ga_gapi_ecc_total(*e,order));
 ga_end();
}
// `curve surface mode w1 w2 u1 u2 v1 v2`: GeomAPI_ExtremaCurveSurface; mode 0 (C, S), 1
// (C, S, ranges), 2 default, 3 default + Init(C, S), 4 default + Init(C, S, ranges).
void ga_gapi_ecs_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);occ::handle<Geom_Surface> s=ga_ics_geom_surface(r);int mode=r.integer();
  double w1=r.real(),w2=r.real(),u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
  std::unique_ptr<GeomAPI_ExtremaCurveSurface> e;
  if(mode==0)e.reset(new GeomAPI_ExtremaCurveSurface(c,s));
  else if(mode==1)e.reset(new GeomAPI_ExtremaCurveSurface(c,s,w1,w2,u1,u2,v1,v2));
  else{e.reset(new GeomAPI_ExtremaCurveSurface());if(mode==3)e->Init(c,s);if(mode==4)e->Init(c,s,w1,w2,u1,u2,v1,v2);}
  int n=e->NbExtrema();ga_int(n);ga_int(int(*e));
  for(int i=0;i<=n+1;i++){GA_TRY(gp_Pnt p,q;e->Points(i,p,q);ga_pnt(p);ga_pnt(q));
   GA_TRY(double w=0,u=0,v=0;e->Parameters(i,w,u,v);ga_real(w);ga_real(u);ga_real(v));GA_TRY(ga_real(e->Distance(i)));}
  GA_TRY(ga_flag(e->IsParallel()));GA_TRY(gp_Pnt p,q;e->NearestPoints(p,q);ga_pnt(p);ga_pnt(q));
  GA_TRY(double w=0,u=0,v=0;e->LowerDistanceParameters(w,u,v);ga_real(w);ga_real(u);ga_real(v));
  GA_TRY(ga_real(e->LowerDistance()));GA_TRY(ga_real(double(*e)));
  const Extrema_ExtCS& x=e->Extrema();ga_flag(x.IsDone());GA_TRY(ga_flag(x.IsParallel()));GA_TRY(ga_int(x.NbExt())));
 ga_end();
}
// `s1 s2 mode u1 u2 v1 v2 u3 u4 v3 v4`: GeomAPI_ExtremaSurfaceSurface; mode 0 (S1, S2), 1
// (S1, S2, bounds), 2 default, 3 default + Init(S1, S2), 4 default + Init(S1, S2, bounds).
void ga_gapi_ess_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom_Surface> s1=ga_ics_geom_surface(r),s2=ga_ics_geom_surface(r);int mode=r.integer();
  double b[8];for(double& x:b)x=r.real();std::unique_ptr<GeomAPI_ExtremaSurfaceSurface> e;
  if(mode==0)e.reset(new GeomAPI_ExtremaSurfaceSurface(s1,s2));
  else if(mode==1)e.reset(new GeomAPI_ExtremaSurfaceSurface(s1,s2,b[0],b[1],b[2],b[3],b[4],b[5],b[6],b[7]));
  else{e.reset(new GeomAPI_ExtremaSurfaceSurface());if(mode==3)e->Init(s1,s2);if(mode==4)e->Init(s1,s2,b[0],b[1],b[2],b[3],b[4],b[5],b[6],b[7]);}
  int n=e->NbExtrema();ga_int(n);ga_int(int(*e));
  for(int i=0;i<=n+1;i++){GA_TRY(gp_Pnt p,q;e->Points(i,p,q);ga_pnt(p);ga_pnt(q));
   GA_TRY(double a=0,c=0,d=0,f=0;e->Parameters(i,a,c,d,f);ga_real(a);ga_real(c);ga_real(d);ga_real(f));GA_TRY(ga_real(e->Distance(i)));}
  GA_TRY(ga_flag(e->IsParallel()));GA_TRY(gp_Pnt p,q;e->NearestPoints(p,q);ga_pnt(p);ga_pnt(q));
  GA_TRY(double a=0,c=0,d=0,f=0;e->LowerDistanceParameters(a,c,d,f);ga_real(a);ga_real(c);ga_real(d);ga_real(f));
  GA_TRY(ga_real(e->LowerDistance()));GA_TRY(ga_real(double(*e)));
  const Extrema_ExtSS& x=e->Extrema();ga_flag(x.IsDone());GA_TRY(ga_flag(x.IsParallel()));GA_TRY(ga_int(x.NbExt())));
 ga_end();
}
