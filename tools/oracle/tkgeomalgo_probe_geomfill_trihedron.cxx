// GeomFill trihedron law probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom_fill_trihedron.rs. The curve readers are
// those of tkgeomalgo_probe_intcurvesurface.cxx and tkgeomalgo_probe_geom2dint.cxx.
#include <Adaptor3d_CurveOnSurface.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <GeomFill_ConstantBiNormal.hxx>
#include <GeomFill_CorrectedFrenet.hxx>
#include <GeomFill_Darboux.hxx>
#include <GeomFill_DiscreteTrihedron.hxx>
#include <GeomFill_DraftTrihedron.hxx>
#include <GeomFill_Fixed.hxx>
#include <GeomFill_Frenet.hxx>
#include <GeomFill_GuideTrihedronAC.hxx>
#include <GeomFill_GuideTrihedronPlan.hxx>
#include <GeomFill_TrihedronWithGuide.hxx>
// `0 curve` (ga_ics_curve) or `1 curve2d surface` (an Adaptor3d_CurveOnSurface).
occ::handle<Adaptor3d_Curve> ga_gft_curve(GaReader& r){
 if(r.integer()==0)return ga_ics_curve(r);
 occ::handle<Geom2d_Curve> c=ga_curve2d(r);occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);
 return new Adaptor3d_CurveOnSurface(new Geom2dAdaptor_Curve(c),s);
}
// A law `kind ...`: 0 Fixed t n, 1 Frenet, 2 CorrectedFrenet mode (0/1 ForEvaluation,
// 2 default), 3 ConstantBiNormal dir, 4 Darboux, 5 DiscreteTrihedron, 6 DraftTrihedron
// b angle, 7 GuideTrihedronAC guide, 8 GuideTrihedronPlan guide.
occ::handle<GeomFill_TrihedronLaw> ga_gft_law(GaReader& r){
 switch(r.integer()){
  case 0:{gp_Vec t(r.xyz());gp_Vec n(r.xyz());return new GeomFill_Fixed(t,n);}
  case 1:return new GeomFill_Frenet();
  case 2:{int m=r.integer();if(m==2)return new GeomFill_CorrectedFrenet();return new GeomFill_CorrectedFrenet(m!=0);}
  case 3:return new GeomFill_ConstantBiNormal(r.dir());
  case 4:return new GeomFill_Darboux();
  case 5:return new GeomFill_DiscreteTrihedron();
  case 6:{gp_Vec b(r.xyz());double a=r.real();return new GeomFill_DraftTrihedron(b,a);}
  case 7:return new GeomFill_GuideTrihedronAC(ga_gft_curve(r));
  default:return new GeomFill_GuideTrihedronPlan(ga_gft_curve(r));
 }
}
void ga_gft_vec(const gp_Vec& v){ga_xyz(v.XYZ());}
// D0, D1 and D2 at `u`, each on outputs preset to (-1, -1, -1).
void ga_gft_eval(const occ::handle<GeomFill_TrihedronLaw>& law,double u){
 gp_Vec s(-1,-1,-1);
 {gp_Vec t=s,n=s,b=s;GA_TRY(ga_flag(law->D0(u,t,n,b)));ga_gft_vec(t);ga_gft_vec(n);ga_gft_vec(b);}
 {gp_Vec t=s,dt=s,n=s,dn=s,b=s,db=s;GA_TRY(ga_flag(law->D1(u,t,dt,n,dn,b,db)));
  for(const gp_Vec* v:{&t,&dt,&n,&dn,&b,&db})ga_gft_vec(*v);}
 {gp_Vec t=s,dt=s,d2t=s,n=s,dn=s,d2n=s,b=s,db=s,d2b=s;GA_TRY(ga_flag(law->D2(u,t,dt,d2t,n,dn,d2n,b,db,d2b)));
  for(const gp_Vec* v:{&t,&dt,&d2t,&n,&dn,&d2n,&b,&db,&d2b})ga_gft_vec(*v);}
}
// The queries that do not depend on a parameter.
void ga_gft_queries(const occ::handle<GeomFill_TrihedronLaw>& law){
 ga_int(int(law->ErrorStatus()));GA_TRY(ga_flag(law->IsConstant()));GA_TRY(ga_flag(law->IsOnlyBy3dCurve()));
 for(int s=0;s<=6;++s){GA_TRY(int n=law->NbIntervals(GeomAbs_Shape(s));ga_int(n);
  NCollection_Array1<double> t(1,n+1);t.Init(-1);GA_TRY(law->Intervals(t,GeomAbs_Shape(s)));for(int i=1;i<=n+1;++i)ga_real(t(i)));}
 GA_TRY(gp_Vec t,n,b;law->GetAverageLaw(t,n,b);ga_gft_vec(t);ga_gft_vec(n);ga_gft_vec(b));
}
// The members of the guided laws: Guide, CurrentPointOnGuide and Origine.
void ga_gft_guide(const occ::handle<GeomFill_TrihedronLaw>& law,double o1,double o2){
 occ::handle<GeomFill_TrihedronWithGuide> g=occ::down_cast<GeomFill_TrihedronWithGuide>(law);if(g.IsNull())return;
 GA_TRY(occ::handle<Adaptor3d_Curve> c=g->Guide();ga_flag(c.IsNull());if(!c.IsNull()){ga_real(c->FirstParameter());ga_real(c->LastParameter());});
 ga_pnt(g->CurrentPointOnGuide());GA_TRY(g->Origine(o1,o2));
}
// `law curve nparams params... a b o1 o2`: SetCurve, the queries, the evaluators at each
// parameter, Copy, SetInterval(a, b) with GetInterval and the evaluators again, the
// guide members (Origine(o1, o2)) and EvaluateBestMode of a corrected Frenet law.
void ga_gft_law_case(std::istream& input){
 GaReader r{input};occ::handle<GeomFill_TrihedronLaw> law;
 GA_TRY(law=ga_gft_law(r));
 if(law.IsNull()){ga_end();return;}
 occ::handle<Adaptor3d_Curve> c=ga_gft_curve(r);std::vector<double> params=r.reals();
 double a=r.real(),b=r.real(),o1=r.real(),o2=r.real();
 GA_TRY(ga_flag(law->SetCurve(c)));ga_gft_queries(law);
 for(double u:params)ga_gft_eval(law,u);
 GA_TRY(occ::handle<GeomFill_TrihedronLaw> copy=law->Copy();if(!params.empty())ga_gft_eval(copy,params[0]));
 // The plane-guided law hides the base myTrimmed, so the base GetInterval would
 // dereference a null handle: it is skipped for that law.
 GA_TRY(law->SetInterval(a,b));
 if(occ::down_cast<GeomFill_GuideTrihedronPlan>(law).IsNull())GA_TRY(double f,l;law->GetInterval(f,l);ga_real(f);ga_real(l));
 for(double u:params)ga_gft_eval(law,u);
 ga_gft_guide(law,o1,o2);
 occ::handle<GeomFill_CorrectedFrenet> cf=occ::down_cast<GeomFill_CorrectedFrenet>(law);
 if(!cf.IsNull())GA_TRY(ga_int(int(cf->EvaluateBestMode())));
 ga_end();
}
