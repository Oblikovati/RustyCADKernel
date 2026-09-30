// GeomFill_SweepFunction and GeomFill_Sweep probe operations (tkgeomalgo_probe.cxx); the
// Rust twin is crates/tkgeomalgo/examples/ga_probe/geom_fill_sweepalgo.rs. The law readers
// are those of tkgeomalgo_probe_geomfill_laws.cxx.
#include <GeomFill_CircularBlendFunc.hxx>
#include <GeomFill_Sweep.hxx>
#include <GeomFill_SweepFunction.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
// A location law (ga_gfl_location) on a path, with its stop surface and guided section.
occ::handle<GeomFill_LocationLaw> ga_gfw_location(GaReader& r){
 occ::handle<GeomFill_SectionLaw> sec;double set[4]={0,0,0,0};occ::handle<Adaptor3d_Surface> stop;
 occ::handle<GeomFill_LocationLaw> l=ga_gfl_location(r,sec,set,stop);occ::handle<Adaptor3d_Curve> c=ga_gft_curve(r);
 GA_TRY(ga_flag(l->SetCurve(c)));
 occ::handle<GeomFill_LocationDraft> d=occ::down_cast<GeomFill_LocationDraft>(l);
 if(!d.IsNull()&&!stop.IsNull())GA_TRY(d->SetStopSurf(stop));
 occ::handle<GeomFill_LocationGuide> g=occ::down_cast<GeomFill_LocationGuide>(l);
 if(!g.IsNull()&&!sec.IsNull())GA_TRY(double la=-9;g->Set(sec,set[0]!=0,set[1],set[2],set[3],la);ga_real(la));
 return l;
}
// D0, D1 and D2 of a sweep function (GeomFill_SweepFunction or GeomFill_CircularBlendFunc) at `u` on `np` poles and `n2` 2d poles.
void ga_gfw_eval(const occ::handle<Approx_SweepFunction>& f,double u,int np,int n2){
 NCollection_Array1<gp_Pnt> p(1,np);NCollection_Array1<gp_Vec> dp(1,np),d2p(1,np);NCollection_Array1<double> w(1,np),dw(1,np),d2w(1,np);
 NCollection_Array1<gp_Pnt2d> p2(1,n2);NCollection_Array1<gp_Vec2d> dp2(1,n2),d2p2(1,n2);
 auto reset=[&](){p.Init(gp_Pnt(-1,-1,-1));dp.Init(gp_Vec(-1,-1,-1));d2p.Init(gp_Vec(-1,-1,-1));w.Init(-1);dw.Init(-1);d2w.Init(-1);
  p2.Init(gp_Pnt2d(-1,-1));dp2.Init(gp_Vec2d(-1,-1));d2p2.Init(gp_Vec2d(-1,-1));};
 auto show2d=[&](const NCollection_Array1<gp_Pnt2d>& a){for(int i=a.Lower();i<=a.Upper();++i)ga_pnt2d(a(i));};
 auto showv2d=[&](const NCollection_Array1<gp_Vec2d>& a){for(int i=a.Lower();i<=a.Upper();++i)ga_xy(a(i).XY());};
 reset();GA_TRY(ga_flag(f->D0(u,0,1,p,p2,w)));ga_gfs_pnts(p);show2d(p2);ga_gfs_reals(w);
 reset();GA_TRY(ga_flag(f->D1(u,0,1,p,dp,p2,dp2,w,dw)));ga_gfs_pnts(p);ga_gfs_vecs(dp);show2d(p2);showv2d(dp2);ga_gfs_reals(w);ga_gfs_reals(dw);
 reset();GA_TRY(ga_flag(f->D2(u,0,1,p,dp,d2p,p2,dp2,d2p2,w,dw,d2w)));ga_gfs_pnts(p);ga_gfs_vecs(dp);ga_gfs_vecs(d2p);
 show2d(p2);showv2d(dp2);showv2d(d2p2);ga_gfs_reals(w);ga_gfs_reals(dw);ga_gfs_reals(d2w);
}
// The queries of a sweep function; returns NbPoles.
int ga_gfw_queries(const occ::handle<Approx_SweepFunction>& f){
 int np=0,nk=0,deg=0;GA_TRY(ga_int(f->Nb2dCurves()));GA_TRY(f->SectionShape(np,nk,deg);ga_int(np);ga_int(nk);ga_int(deg));
 GA_TRY(NCollection_Array1<double> k(1,nk>0?nk:1);f->Knots(k);ga_int(k.Lower());ga_gfs_reals(k));
 GA_TRY(NCollection_Array1<int> m(1,nk>0?nk:1);f->Mults(m);for(int i=m.Lower();i<=m.Upper();++i)ga_int(m(i)));
 GA_TRY(ga_flag(f->IsRational()));
 for(int sh=0;sh<=6;++sh){GA_TRY(int n=f->NbIntervals(GeomAbs_Shape(sh));ga_int(n);
  NCollection_Array1<double> t(1,n+1);t.Init(-1);GA_TRY(f->Intervals(t,GeomAbs_Shape(sh)));ga_gfs_reals(t));}
 GA_TRY(NCollection_Array1<double> t(1,np>0?np:1);f->GetTolerance(1e-3,1e-2,0.1,t);ga_gfs_reals(t));
 GA_TRY(double tu=-1,tv=-1;f->Resolution(1,1e-3,tu,tv);ga_real(tu);ga_real(tv));
 GA_TRY(ga_pnt(f->BarycentreOfSurf()));GA_TRY(ga_real(f->MaximalSection()));
 GA_TRY(NCollection_Array1<double> w(1,np>0?np:1);f->GetMinimalWeight(w);ga_gfs_reals(w));
 return np;
}
// `sweepfunc location section f fOnS ratio nparams params... a b`: the queries, the
// evaluators, SetInterval(a, b), SetTolerance and the evaluators again. The evaluators
// run only for a section with poles (the native arrays are not range checked).
void ga_gfw_function_case(std::istream& input){
 GaReader r{input};occ::handle<GeomFill_LocationLaw> l;GA_TRY(l=ga_gfw_location(r));
 if(l.IsNull()){ga_end();return;}
 occ::handle<GeomFill_SectionLaw> s;GA_TRY(s=ga_gfl_section(r));
 if(s.IsNull()){ga_end();return;}
 double f=r.real(),fos=r.real(),ratio=r.real();std::vector<double> params=r.reals();double a=r.real(),b=r.real();
 occ::handle<GeomFill_SweepFunction> fn=new GeomFill_SweepFunction(s,l,f,fos,ratio);
 int np=ga_gfw_queries(fn);int n2=l->Nb2dCurves();
 if(np<=0){ga_end();return;}
 for(double u:params)ga_gfw_eval(fn,u,np,n2);
 GA_TRY(fn->SetInterval(a,b));GA_TRY(fn->SetTolerance(1e-5,1e-6));
 for(double u:params)ga_gfw_eval(fn,u,np,n2);
 ga_end();
}
void ga_gfw_curve2d(const occ::handle<Geom2d_Curve>& c){
 if(c.IsNull()){ga_int(-1);return;}
 double f=c->FirstParameter(),l=c->LastParameter();ga_real(f);ga_real(l);GA_TRY(ga_pnt2d(c->Value(0.5*(f+l))));}
// The type (and the basis type of a trimmed surface), bounds, periodicity and nine points
// of a surface (the B-spline shape too).
void ga_gfw_surface(const occ::handle<Geom_Surface>& s){
 if(s.IsNull()){ga_int(-1);return;}
 std::cout<<s->DynamicType()->Name()<<' ';
 occ::handle<Geom_RectangularTrimmedSurface> rt=occ::down_cast<Geom_RectangularTrimmedSurface>(s);
 if(!rt.IsNull())std::cout<<rt->BasisSurface()->DynamicType()->Name()<<' ';double u1,u2,v1,v2;s->Bounds(u1,u2,v1,v2);ga_real(u1);ga_real(u2);ga_real(v1);ga_real(v2);
 ga_flag(s->IsUPeriodic());ga_flag(s->IsVPeriodic());
 for(double tu:{0.1,0.5,0.9})for(double tv:{0.1,0.5,0.9})GA_TRY(ga_pnt(s->Value(u1+(u2-u1)*tu,v1+(v2-v1)*tv)));
 occ::handle<Geom_BSplineSurface> b=occ::down_cast<Geom_BSplineSurface>(s);
 if(!b.IsNull()){ga_int(b->NbUPoles());ga_int(b->NbVPoles());ga_int(b->UDegree());ga_int(b->VDegree());ga_flag(b->IsURational());ga_flag(b->IsVRational());}
}
// The results of a build; the 2d curves exist only after an approximation (a B-spline
// result): natively they are a null handle otherwise.
void ga_gfw_result(const GeomFill_Sweep& sw){
 ga_flag(sw.IsDone());ga_flag(sw.ExchangeUV());ga_flag(sw.UReversed());ga_flag(sw.VReversed());ga_real(sw.ErrorOnSurface());
 occ::handle<Geom_Surface> s=sw.Surface();ga_gfw_surface(s);GA_TRY(ga_int(sw.NumberOfTrace()));
 if(occ::down_cast<Geom_BSplineSurface>(s).IsNull())return;
 for(bool first:{true,false}){GA_TRY(double eu,ev;sw.ErrorOnRestriction(first,eu,ev);ga_real(eu);ga_real(ev));GA_TRY(ga_gfw_curve2d(sw.Restriction(first)));}
 int nt=sw.NumberOfTrace();
 for(int i=-1;i<=nt+1;++i){GA_TRY(double eu,ev;sw.ErrorOnTrace(i,eu,ev);ga_real(eu);ga_real(ev));GA_TRY(ga_gfw_curve2d(sw.Trace(i)));}
}
// `sweep location section kpart domain [lf ll sf sl] tol3d boundtol tol2d tolang forcec1
// methode cont degmax segmax cont2`: the sweep built twice (continuity `cont`, then `cont2`).
void ga_gfw_sweep_case(std::istream& input){
 GaReader r{input};occ::handle<GeomFill_LocationLaw> l;GA_TRY(l=ga_gfw_location(r));
 if(l.IsNull()){ga_end();return;}
 occ::handle<GeomFill_SectionLaw> s;GA_TRY(s=ga_gfl_section(r));
 if(s.IsNull()){ga_end();return;}
 bool kpart=r.flag(),dom=r.flag();double d[4]={0,0,0,0};if(dom)for(double& x:d)x=r.real();
 double t[4];for(double& x:t)x=r.real();bool fc=r.flag();int m=r.integer(),cont=r.integer(),degmax=r.integer(),segmax=r.integer(),cont2=r.integer();
 std::unique_ptr<GeomFill_Sweep> sw;GA_TRY(sw=std::make_unique<GeomFill_Sweep>(l,kpart));
 if(!sw){ga_end();return;}
 if(dom)sw->SetDomain(d[0],d[1],d[2],d[3]);sw->SetTolerance(t[0],t[1],t[2],t[3]);sw->SetForceApproxC1(fc);
 GA_TRY(sw->Build(s,GeomFill_ApproxStyle(m),GeomAbs_Shape(cont),degmax,segmax));ga_gfw_result(*sw);
 GA_TRY(sw->Build(s,GeomFill_ApproxStyle(m),GeomAbs_Shape(cont2),degmax,segmax));ga_gfw_result(*sw);
 ga_end();
}
// `blendfunc path curve1 curve2 radius polynomial nparams params... a b`: the queries,
// the evaluators, SetInterval(a, b) and the evaluators again.
void ga_gfw_blend_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Curve> p=ga_gft_curve(r),c1=ga_gft_curve(r),c2=ga_gft_curve(r);
 double rad=r.real();bool poly=r.flag();std::vector<double> params=r.reals();double a=r.real(),b=r.real();
 occ::handle<GeomFill_CircularBlendFunc> fn;GA_TRY(fn=new GeomFill_CircularBlendFunc(p,c1,c2,rad,poly));
 if(fn.IsNull()){ga_end();return;}
 int np=ga_gfw_queries(fn);
 for(double u:params)ga_gfw_eval(fn,u,np,0);
 GA_TRY(fn->SetInterval(a,b));GA_TRY(fn->SetTolerance(1e-5,1e-6));
 for(double u:params)ga_gfw_eval(fn,u,np,0);
 ga_end();
}
