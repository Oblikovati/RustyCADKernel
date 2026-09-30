// GeomFill section and location law probe operations (tkgeomalgo_probe.cxx); the Rust
// twin is crates/tkgeomalgo/examples/ga_probe/geom_fill_laws.rs. The laws reuse the
// readers of tkgeomalgo_probe_geomfill_trihedron.cxx and tkgeomalgo_probe_law.cxx.
#include <GeomFill_CurveAndTrihedron.hxx>
#include <GeomFill_EvolvedSection.hxx>
#include <GeomFill_LocationDraft.hxx>
#include <GeomFill_LocationGuide.hxx>
#include <GeomFill_LocFunction.hxx>
#include <GeomFill_NSections.hxx>
#include <GeomFill_UniformSection.hxx>
#include <Law_BSpFunc.hxx>
#include <Law_Constant.hxx>
#include <Law_Linear.hxx>
// A scalar law `kind ...`: 0 constant v f l, 1 linear pdeb vdeb pfin vfin, 2 the
// B-spline law of tkgeomalgo_probe_law.cxx over `first last`.
occ::handle<Law_Function> ga_gfl_law(GaReader& r){
 int kind=r.integer();
 if(kind==0){occ::handle<Law_Constant> c=new Law_Constant();double v=r.real(),f=r.real();c->Set(v,f,r.real());return c;}
 if(kind==1){occ::handle<Law_Linear> l=new Law_Linear();double a=r.real(),b=r.real(),c=r.real();l->Set(a,b,c,r.real());return l;}
 occ::handle<Law_BSpline> b=ga_law_bspline(r);double f=r.real();return new Law_BSpFunc(b,f,r.real());
}
gp_Trsf ga_gfn_trsf(GaReader& r){
 double v[12];for(double& x:v)x=r.real();gp_Trsf t;t.SetValues(v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7],v[8],v[9],v[10],v[11]);return t;}
// `ctor n curves... np params... [extras]`: ctor 0 `NC`, 1 `NC NP`, 2 `NC NP UF UL`, 3 `NC
// NP UF UL VF VL`, 4 `NC Trsfs NP UF UL VF VL Surf` (read as `ntrsf [12 reals]... uf ul vf
// vl surface` after the parameters).
occ::handle<GeomFill_SectionLaw> ga_gfn_nsections(GaReader& r){
 int ctor=r.integer(),n=r.integer();NCollection_Sequence<occ::handle<Geom_Curve>> nc;
 for(int i=0;i<n;++i)nc.Append(ga_ics_geom_curve(r));
 std::vector<double> p=r.reals();NCollection_Sequence<double> np;for(double x:p)np.Append(x);
 if(ctor==0)return new GeomFill_NSections(nc);
 if(ctor==1)return new GeomFill_NSections(nc,np);
 if(ctor==2){double uf=r.real();return new GeomFill_NSections(nc,np,uf,r.real());}
 if(ctor==3){double uf=r.real(),ul=r.real(),vf=r.real();return new GeomFill_NSections(nc,np,uf,ul,vf,r.real());}
 int nt=r.integer();NCollection_Sequence<gp_Trsf> ts;for(int i=0;i<nt;++i)ts.Append(ga_gfn_trsf(r));
 double uf=r.real(),ul=r.real(),vf=r.real(),vl=r.real();
 occ::handle<Geom_BSplineSurface> s=occ::down_cast<Geom_BSplineSurface>(ga_ics_geom_surface(r));
 return new GeomFill_NSections(nc,ts,np,uf,ul,vf,vl,s);
}
// A section law `kind ...`: 0 uniform curve first last, 1 evolved curve law, 2 N sections.
occ::handle<GeomFill_SectionLaw> ga_gfl_section(GaReader& r){
 int kind=r.integer();if(kind==2)return ga_gfn_nsections(r);occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);
 if(kind==0){double f=r.real();return new GeomFill_UniformSection(c,f,r.real());}
 return new GeomFill_EvolvedSection(c,ga_gfl_law(r));
}
void ga_gfl_curve(const occ::handle<Geom_Curve>& c){
 if(c.IsNull()){ga_int(-1);return;}
 double f=c->FirstParameter(),l=c->LastParameter();ga_real(f);ga_real(l);GA_TRY(ga_pnt(c->Value(0.5*(f+l))));}
// D0, D1 and D2 of a section law at `u` on arrays of `n` poles from `lower`.
void ga_gfl_section_eval(const occ::handle<GeomFill_SectionLaw>& s,double u,int n,int lower){
 int up=lower+n-1;
 {NCollection_Array1<gp_Pnt> p(lower,up);NCollection_Array1<double> w(lower,up);p.Init(gp_Pnt(-1,-1,-1));w.Init(-1);
  GA_TRY(ga_flag(s->D0(u,p,w)));ga_int(p.Lower());ga_gfs_pnts(p);ga_gfs_reals(w);}
 {NCollection_Array1<gp_Pnt> p(lower,up);NCollection_Array1<gp_Vec> dp(lower,up);NCollection_Array1<double> w(lower,up),dw(lower,up);
  p.Init(gp_Pnt(-1,-1,-1));dp.Init(gp_Vec(-1,-1,-1));w.Init(-1);dw.Init(-1);
  GA_TRY(ga_flag(s->D1(u,p,dp,w,dw)));ga_gfs_pnts(p);ga_gfs_vecs(dp);ga_gfs_reals(w);ga_gfs_reals(dw);}
 {NCollection_Array1<gp_Pnt> p(lower,up);NCollection_Array1<gp_Vec> dp(lower,up),d2p(lower,up);NCollection_Array1<double> w(lower,up),dw(lower,up),d2w(lower,up);
  p.Init(gp_Pnt(-1,-1,-1));dp.Init(gp_Vec(-1,-1,-1));d2p.Init(gp_Vec(-1,-1,-1));w.Init(-1);dw.Init(-1);d2w.Init(-1);
  GA_TRY(ga_flag(s->D2(u,p,dp,d2p,w,dw,d2w)));ga_gfs_pnts(p);ga_gfs_vecs(dp);ga_gfs_vecs(d2p);ga_gfs_reals(w);ga_gfs_reals(dw);ga_gfs_reals(d2w);}
}
// The queries of a section law.
void ga_gfl_section_queries(const occ::handle<GeomFill_SectionLaw>& s){
 int np=0,nk=0,deg=0;GA_TRY(s->SectionShape(np,nk,deg);ga_int(np);ga_int(nk);ga_int(deg));
 GA_TRY(NCollection_Array1<double> k(1,1);k.Init(0);s->Knots(k);ga_int(k.Lower());ga_gfs_reals(k));
 GA_TRY(NCollection_Array1<int> m(1,1);m.Init(0);s->Mults(m);for(int i=m.Lower();i<=m.Upper();++i)ga_int(m(i)));
 GA_TRY(ga_flag(s->IsRational()));GA_TRY(ga_flag(s->IsUPeriodic()));GA_TRY(ga_flag(s->IsVPeriodic()));
 for(int sh=0;sh<=6;++sh){GA_TRY(int n=s->NbIntervals(GeomAbs_Shape(sh));ga_int(n);
  NCollection_Array1<double> t(1,n+1);t.Init(-1);GA_TRY(s->Intervals(t,GeomAbs_Shape(sh)));ga_gfs_reals(t));}
 GA_TRY(double f,l;s->GetDomain(f,l);ga_real(f);ga_real(l));GA_TRY(double f,l;s->GetInterval(f,l);ga_real(f);ga_real(l));
 GA_TRY(NCollection_Array1<double> t(1,np>1?np:2);s->GetTolerance(1e-3,1e-2,0.1,t);ga_gfs_reals(t));
 GA_TRY(NCollection_Array1<double> t(1,np>1?np:2);s->GetTolerance(1e-1,1e-2,0.1,t);ga_gfs_reals(t));
 GA_TRY(ga_pnt(s->BarycentreOfSurf()));GA_TRY(ga_real(s->MaximalSection()));
 GA_TRY(NCollection_Array1<double> w(1,1);w.Init(0);s->GetMinimalWeight(w);ga_int(w.Upper());ga_gfs_reals(w));
 double e=-7;GA_TRY(ga_flag(s->IsConstant(e));ga_real(e));GA_TRY(ga_gfl_curve(s->ConstantSection()));
 e=-7;GA_TRY(ga_flag(s->IsConicalLaw(e));ga_real(e));GA_TRY(ga_gfl_curve(s->CirclSection(0.5)));
 GA_TRY(occ::handle<Geom_BSplineSurface> b=s->BSplineSurface();ga_flag(b.IsNull());
  if(!b.IsNull()){ga_int(b->NbUPoles());ga_int(b->NbVPoles());ga_pnt(b->Value(0.3,0.6));});
}
// `section nparams params... lower a b`: the queries, the evaluators on NbPoles poles
// from `lower` and on one pole less, SetInterval(a, b) and the evaluators again.
void ga_gfl_section_case(std::istream& input){
 GaReader r{input};occ::handle<GeomFill_SectionLaw> s;GA_TRY(s=ga_gfl_section(r));
 if(s.IsNull()){ga_end();return;}
 std::vector<double> params=r.reals();int lower=r.integer();double a=r.real(),b=r.real();
 ga_gfl_section_queries(s);int np=0,nk=0,deg=0;GA_TRY(s->SectionShape(np,nk,deg));
 for(double u:params){ga_gfl_section_eval(s,u,np,lower);ga_gfl_section_eval(s,u,np-1,1);}
 GA_TRY(s->SetInterval(a,b));GA_TRY(double f,l;s->GetInterval(f,l);ga_real(f);ga_real(l));
 for(double u:params)ga_gfl_section_eval(s,u,np,1);
 // An N-section law recomputed as a segment of its own surface.
 occ::handle<GeomFill_NSections> ns=occ::down_cast<GeomFill_NSections>(s);
 if(!ns.IsNull()){GA_TRY(ns->SetSurface(ns->BSplineSurface());ns->ComputeSurface());ga_gfl_section_queries(ns);}
 ga_end();
}
void ga_gfl_mat(const gp_Mat& m){for(int i=1;i<=3;++i)for(int j=1;j<=3;++j)ga_real(m.Value(i,j));}
gp_Mat ga_gfl_sentinel(){return gp_Mat(-1,-1,-1,-1,-1,-1,-1,-1,-1);}
// A location law `kind ...`: 0 CurveAndTrihedron law, 1 LocationDraft dir angle
// (flag surface, set after SetCurve), 2 LocationGuide law (flag section rotat sfirst slast precangle).
occ::handle<GeomFill_LocationLaw> ga_gfl_location(GaReader& r,occ::handle<GeomFill_SectionLaw>& sec,double set[4],occ::handle<Adaptor3d_Surface>& stop){
 int kind=r.integer();
 if(kind==0)return new GeomFill_CurveAndTrihedron(ga_gft_law(r));
 if(kind==1){gp_Dir d=r.dir();double a=r.real();occ::handle<GeomFill_LocationDraft> l=new GeomFill_LocationDraft(d,a);
  if(r.flag())stop=ga_ics_surface(r);return l;}
 occ::handle<GeomFill_TrihedronWithGuide> t=occ::down_cast<GeomFill_TrihedronWithGuide>(ga_gft_law(r));
 if(r.flag()){sec=ga_gfl_section(r);for(int i=0;i<4;++i)set[i]=r.real();}
 return new GeomFill_LocationGuide(t);
}
// The placements at `u` with `n` 2d poles.
void ga_gfl_location_eval(const occ::handle<GeomFill_LocationLaw>& l,double u,int n){
 gp_Vec s(-1,-1,-1);
 {gp_Mat m=ga_gfl_sentinel();gp_Vec v=s;GA_TRY(ga_flag(l->D0(u,m,v)));ga_gfl_mat(m);ga_xyz(v.XYZ());}
 {gp_Mat m=ga_gfl_sentinel();gp_Vec v=s;NCollection_Array1<gp_Pnt2d> p(1,n);p.Init(gp_Pnt2d(-1,-1));
  GA_TRY(ga_flag(l->D0(u,m,v,p)));ga_gfl_mat(m);ga_xyz(v.XYZ());for(int i=1;i<=n;++i)ga_pnt2d(p(i));}
 {gp_Mat m=ga_gfl_sentinel(),dm=ga_gfl_sentinel();gp_Vec v=s,dv=s;NCollection_Array1<gp_Pnt2d> p(1,n);NCollection_Array1<gp_Vec2d> dp(1,n);
  p.Init(gp_Pnt2d(-1,-1));dp.Init(gp_Vec2d(-1,-1));GA_TRY(ga_flag(l->D1(u,m,v,dm,dv,p,dp)));
  ga_gfl_mat(m);ga_xyz(v.XYZ());ga_gfl_mat(dm);ga_xyz(dv.XYZ());for(int i=1;i<=n;++i){ga_pnt2d(p(i));ga_xy(dp(i).XY());}}
 {gp_Mat m=ga_gfl_sentinel(),dm=ga_gfl_sentinel(),d2m=ga_gfl_sentinel();gp_Vec v=s,dv=s,d2v=s;
  NCollection_Array1<gp_Pnt2d> p(1,n);NCollection_Array1<gp_Vec2d> dp(1,n),d2p(1,n);p.Init(gp_Pnt2d(-1,-1));dp.Init(gp_Vec2d(-1,-1));d2p.Init(gp_Vec2d(-1,-1));
  GA_TRY(ga_flag(l->D2(u,m,v,dm,dv,d2m,d2v,p,dp,d2p)));ga_gfl_mat(m);ga_xyz(v.XYZ());ga_gfl_mat(dm);ga_xyz(dv.XYZ());ga_gfl_mat(d2m);ga_xyz(d2v.XYZ());
  for(int i=1;i<=n;++i){ga_pnt2d(p(i));ga_xy(dp(i).XY());ga_xy(d2p(i).XY());}}
 ga_int(int(l->ErrorStatus()));
}
// The queries of a location law.
// Resolution(1) of a draft law without stop surface reads a null surface (a crash
// natively), so `res1` skips it.
void ga_gfl_location_queries(const occ::handle<GeomFill_LocationLaw>& l,bool res1){
 ga_int(int(l->ErrorStatus()));ga_int(l->Nb2dCurves());ga_flag(l->HasFirstRestriction());ga_flag(l->HasLastRestriction());ga_int(l->TraceNumber());
 for(int sh=0;sh<=6;++sh){GA_TRY(int n=l->NbIntervals(GeomAbs_Shape(sh));ga_int(n);
  NCollection_Array1<double> t(1,n+1);t.Init(-1);GA_TRY(l->Intervals(t,GeomAbs_Shape(sh)));ga_gfs_reals(t));}
 GA_TRY(double f,g;l->GetDomain(f,g);ga_real(f);ga_real(g));GA_TRY(double f,g;l->GetInterval(f,g);ga_real(f);ga_real(g));
 GA_TRY(ga_real(l->GetMaximalNorm()));GA_TRY(gp_Mat m;gp_Vec v;l->GetAverageLaw(m,v);ga_gfl_mat(m);ga_xyz(v.XYZ()));
 double e=-7;GA_TRY(ga_flag(l->IsTranslation(e));ga_real(e));e=-7;GA_TRY(ga_flag(l->IsRotation(e));ga_real(e));
 GA_TRY(gp_Pnt c(-1,-1,-1);l->Rotation(c);ga_pnt(c));
 for(int i=res1?1:2;i<=2;++i)GA_TRY(double tu=-1,tv=-1;l->Resolution(i,1e-3,tu,tv);ga_real(tu);ga_real(tv));
}
// `law path nparams params... a b trsf [9 reals] tol3d`: SetCurve (and Set for a guided
// law with a section), the queries, the evaluators, LocFunction::DN, Copy,
// SetInterval, SetTrsf, SetTolerance and the members of the draft and guided laws.
void ga_gfl_location_case(std::istream& input){
 GaReader r{input};occ::handle<GeomFill_SectionLaw> sec;double set[4]={0,0,0,0};occ::handle<GeomFill_LocationLaw> l;
 occ::handle<Adaptor3d_Surface> stop;GA_TRY(l=ga_gfl_location(r,sec,set,stop));
 if(l.IsNull()){ga_end();return;}
 occ::handle<Adaptor3d_Curve> c=ga_gft_curve(r);std::vector<double> params=r.reals();double a=r.real(),b=r.real();
 bool trsf=r.flag();gp_Mat tm;if(trsf)for(int i=1;i<=3;++i)for(int j=1;j<=3;++j)tm.SetValue(i,j,r.real());double tol=r.real();
 GA_TRY(ga_flag(l->SetCurve(c)));
 occ::handle<GeomFill_LocationDraft> d=occ::down_cast<GeomFill_LocationDraft>(l);
 if(!d.IsNull()&&!stop.IsNull())GA_TRY(d->SetStopSurf(stop));
 occ::handle<GeomFill_LocationGuide> g=occ::down_cast<GeomFill_LocationGuide>(l);
 if(!g.IsNull()&&!sec.IsNull())GA_TRY(double la=-9;g->Set(sec,set[0]!=0,set[1],set[2],set[3],la);ga_real(la));
 ga_gfl_location_queries(l,d.IsNull()||!stop.IsNull());int n=l->Nb2dCurves()>0?l->Nb2dCurves():1;
 for(double u:params)ga_gfl_location_eval(l,u,n);
 if(!params.empty()){GeomFill_LocFunction f(l);for(int o=0;o<=2;++o)GA_TRY(double res[12];int ier=-1;f.DN(params[0],0,1,o,res[0],ier);ga_int(ier);for(double x:res)ga_real(x));}
 // The copy of a guided law has no path (a null curve natively): only its curve is read.
 GA_TRY(occ::handle<GeomFill_LocationLaw> copy=l->Copy();ga_flag(copy->GetCurve().IsNull());if(g.IsNull()&&!params.empty())ga_gfl_location_eval(copy,params[0],n));
 GA_TRY(l->SetInterval(a,b));GA_TRY(double f,h;l->GetInterval(f,h);ga_real(f);ga_real(h));
 for(double u:params)ga_gfl_location_eval(l,u,n);
 if(trsf){GA_TRY(l->SetTrsf(tm));if(!params.empty())ga_gfl_location_eval(l,params[0],n);}
 GA_TRY(l->SetTolerance(tol,tol));
 if(!d.IsNull()){ga_flag(d->IsIntersec());ga_dir(d->Direction());GA_TRY(d->SetAngle(0.2));if(!params.empty())ga_gfl_location_eval(l,params[0],n);}
 // Section() reads the section law, a null handle (a crash natively) until Set.
 if(!g.IsNull()){if(!sec.IsNull())GA_TRY(ga_gfl_curve(g->Section()));occ::handle<Adaptor3d_Curve> gd=g->Guide();ga_real(gd->FirstParameter());ga_real(gd->LastParameter());
  GA_TRY(occ::handle<NCollection_HArray1<gp_Pnt2d>> pr;ga_int(int(g->ComputeAutomaticLaw(pr)));for(int i=1;i<=pr->Length();++i)ga_pnt2d(pr->Value(i)));
  g->EraseRotation();ga_int(int(g->ErrorStatus()));g->SetOrigine(0.1,0.8);if(!params.empty())ga_gfl_location_eval(l,params[0],n);}
 ga_end();
}
