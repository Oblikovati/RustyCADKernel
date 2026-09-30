// GeomFill_Gordon and GeomFill_NetworkSurface probe operations (tkgeomalgo_probe.cxx); the
// Rust twin is crates/tkgeomalgo/examples/ga_probe/gordon.rs. Curves are read with
// ga_ics_geom_curve (tkgeomalgo_probe_intcurvesurface.cxx) and surfaces are printed with
// ga_gapi_bsurface_out (tkgeomalgo_probe_geomapi_surface.cxx).
#include <GeomFill_Gordon.hxx>
#include <GeomFill_NetworkSurface.hxx>
// `n curve*n` (n >= 1) as a 1-based array of curves.
NCollection_Array1<occ::handle<Geom_Curve>> ga_gordon_curves(GaReader& r){
 int n=r.integer();NCollection_Array1<occ::handle<Geom_Curve>> a(1,n);
 for(int i=1;i<=n;++i)a(i)=ga_ics_geom_curve(r);return a;}
// `n curve*n` (n >= 1, every curve a B-spline) as a 1-based array of B-spline curves.
NCollection_Array1<occ::handle<Geom_BSplineCurve>> ga_gordon_bsplines(GaReader& r){
 int n=r.integer();NCollection_Array1<occ::handle<Geom_BSplineCurve>> a(1,n);
 for(int i=1;i<=n;++i)a(i)=occ::down_cast<Geom_BSplineCurve>(ga_ics_geom_curve(r));return a;}
// Status, report, IsDone, IsApproximate, IsParallelMode, GetApproximationMode and Surface.
void ga_gordon_print(const GeomFill_Gordon& g){
 const GeomFill_Gordon::BuildReport& rep=g.Report();
 ga_int(int(g.Status()));ga_int(int(rep.Status));ga_int(int(rep.FailedStage));ga_flag(rep.IsApproximate);
 ga_real(rep.MaxContactGap);ga_real(rep.MaxReparametrizationDeviation);ga_real(rep.MaxProfileDeviation);
 ga_real(rep.MaxGuideDeviation);ga_real(rep.MaxApproximationDeviation);
 ga_flag(g.IsDone());ga_flag(g.IsApproximate());ga_flag(g.IsParallelMode());ga_int(int(g.GetApproximationMode()));
 GA_TRY(ga_gapi_bsurface_out(g.Surface()));
}
// `mode parallel tolerance profiles guides performs`: GeomFill_Gordon; mode 0 ExactOnly, 1
// AllowApproximateFallback (set before Init), `performs` Perform calls, each printed.
void ga_gfill_gordon_case(std::istream& input){
 GaReader r{input};int mode=r.integer();bool parallel=r.flag();double tol=r.real();
 NCollection_Array1<occ::handle<Geom_Curve>> profiles=ga_gordon_curves(r),guides=ga_gordon_curves(r);
 int performs=r.integer();GeomFill_Gordon g;g.SetParallelMode(parallel);
 if(mode==1)g.SetApproximationMode(GeomFill_Gordon::ApproximationMode::AllowApproximateFallback);
 GA_TRY(g.Init(profiles,guides,tol));ga_gordon_print(g);
 for(int k=0;k<performs;++k){GA_TRY(g.Perform());ga_gordon_print(g);}
 ga_end();
}
// `profiles guides np vparams ng uparams rl ru cl cu points weights tolerance uclosed vclosed
// performs`: GeomFill_NetworkSurface; the points and weights are row by row on the given
// bounds, `performs` Perform calls, each printed with its status and surface.
void ga_gfill_network_case(std::istream& input){
 GaReader r{input};NCollection_Array1<occ::handle<Geom_BSplineCurve>> profiles=ga_gordon_bsplines(r),guides=ga_gordon_bsplines(r);
 NCollection_Array1<double> vparams=ga_ics_params(r),uparams=ga_ics_params(r);
 int rl=r.integer(),ru=r.integer(),cl=r.integer(),cu=r.integer();
 NCollection_Array2<gp_Pnt> pts(rl,ru,cl,cu);NCollection_Array2<double> ws(rl,ru,cl,cu);
 for(int i=rl;i<=ru;i++)for(int j=cl;j<=cu;j++)pts(i,j)=r.pnt();
 for(int i=rl;i<=ru;i++)for(int j=cl;j<=cu;j++)ws(i,j)=r.real();
 double tol=r.real();bool uc=r.flag(),vc=r.flag();int performs=r.integer();
 GeomFill_NetworkSurface n;n.Init(profiles,guides,vparams,uparams,pts,ws,tol,uc,vc);
 ga_int(int(n.Status()));ga_flag(n.IsDone());
 for(int k=0;k<performs;++k){GA_TRY(n.Perform());ga_int(int(n.Status()));ga_flag(n.IsDone());GA_TRY(ga_gapi_bsurface_out(n.Surface()));}
 ga_end();
}
