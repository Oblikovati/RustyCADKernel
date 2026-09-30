// GeomFill filling-part probe operations: the skinning classes (tkgeomalgo_probe.cxx).
// The Rust twin is crates/tkgeomalgo/examples/ga_probe/gfill.rs. The curve reader and
// the dumps are shared with tkgeomalgo_probe_gfill_*.cxx.
#include <GeomFill_AppSurf.hxx>
#include <GeomFill_Generator.hxx>
#include <GeomFill_Line.hxx>
#include <GeomFill_Profiler.hxx>
#include <GeomFill_SectionGenerator.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Line.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <NCollection_HArray1.hxx>
// A curve: `0 deg periodic rational nk knots mults np poles [weights]` (B-spline),
// `1 np poles rational [weights]` (Bezier), `2 center normal radius` (circle),
// `3 center normal major minor` (ellipse), `4 u1 u2 <curve>` (trimmed),
// `5 point direction` (line).
occ::handle<Geom_Curve> ga_gf_curve(GaReader& r);
occ::handle<Geom_BSplineCurve> ga_gf_bspline(GaReader& r){
 int deg=r.integer();bool periodic=r.flag(),rational=r.flag();int nk=r.integer();
 NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);
 for(int i=1;i<=nk;i++)k(i)=r.real();for(int i=1;i<=nk;i++)m(i)=r.integer();
 int np=r.integer();NCollection_Array1<gp_Pnt> p(1,np);for(int i=1;i<=np;i++)p(i)=r.pnt();
 if(!rational)return new Geom_BSplineCurve(p,k,m,deg,periodic);
 NCollection_Array1<double> w(1,np);for(int i=1;i<=np;i++)w(i)=r.real();
 return new Geom_BSplineCurve(p,w,k,m,deg,periodic);
}
occ::handle<Geom_BezierCurve> ga_gf_bezier(GaReader& r){
 int np=r.integer();NCollection_Array1<gp_Pnt> p(1,np);for(int i=1;i<=np;i++)p(i)=r.pnt();
 if(!r.flag())return new Geom_BezierCurve(p);
 NCollection_Array1<double> w(1,np);for(int i=1;i<=np;i++)w(i)=r.real();
 return new Geom_BezierCurve(p,w);
}
occ::handle<Geom_Curve> ga_gf_curve(GaReader& r){
 int kind=r.integer();
 if(kind==0)return ga_gf_bspline(r);
 if(kind==1)return ga_gf_bezier(r);
 if(kind==2){gp_Pnt c=r.pnt();gp_Dir n=r.dir();return new Geom_Circle(gp_Ax2(c,n),r.real());}
 if(kind==3){gp_Pnt c=r.pnt();gp_Dir n=r.dir();double a=r.real();return new Geom_Ellipse(gp_Ax2(c,n),a,r.real());}
 if(kind==4){double u1=r.real(),u2=r.real();occ::handle<Geom_Curve> b=ga_gf_curve(r);return new Geom_TrimmedCurve(b,u1,u2);}
 gp_Pnt p=r.pnt();return new Geom_Line(p,r.dir());
}
// The fields of a B-spline curve.
void ga_gf_dump_bspline(const occ::handle<Geom_BSplineCurve>& c){
 if(c.IsNull()){std::cout<<"null ";return;}
 ga_int(c->Degree());ga_flag(c->IsPeriodic());ga_flag(c->IsRational());ga_int(c->NbKnots());
 for(double k:c->Knots())ga_real(k);for(int m:c->Multiplicities())ga_int(m);
 ga_int(c->NbPoles());for(const gp_Pnt& p:c->Poles())ga_pnt(p);for(double w:c->WeightsArray())ga_real(w);
}
// `n curves...`: the sections of a profiler.
template<class P> void ga_gf_add_curves(GaReader& r,P& p){
 int n=r.integer();for(int i=0;i<n;i++){occ::handle<Geom_Curve> c=ga_gf_curve(r);GA_TRY(p.AddCurve(c));}
}
// The consultation of a performed (or not) profiler, with sizes and indices out of range.
void ga_gf_profiler_report(const GeomFill_Profiler& p,int n){
 ga_flag(p.IsPeriodic());GA_TRY(ga_int(p.Degree()));GA_TRY(ga_int(p.NbPoles()));GA_TRY(ga_int(p.NbKnots()));
 int np=0,nk=0;try{np=p.NbPoles();nk=p.NbKnots();}catch(const Standard_Failure&){}
 for(int extra=0;extra<2;extra++){
  GA_TRY(NCollection_Array1<double> k(1,nk+extra);NCollection_Array1<int> m(1,nk);p.KnotsAndMults(k,m);
   for(double x:k)ga_real(x);for(int x:m)ga_int(x));
 }
 for(int i=0;i<=n+1;i++){
  GA_TRY(NCollection_Array1<gp_Pnt> q(1,np);p.Poles(i,q);ga_int(q.Lower());for(const gp_Pnt& x:q)ga_pnt(x));
  GA_TRY(NCollection_Array1<double> w(0,np-1);p.Weights(i,w);ga_int(w.Lower());for(double x:w)ga_real(x));
  GA_TRY(ga_gf_dump_bspline(occ::down_cast<Geom_BSplineCurve>(p.Curve(i))));
 }
 GA_TRY(NCollection_Array1<gp_Pnt> q(1,np+1);p.Poles(1,q));
}
// `ptol perform curves...`: the profiler of the sections, performed when `perform`.
void ga_gfill_profiler_case(std::istream& input){
 GaReader r{input};double ptol=r.real();bool perform=r.flag();GeomFill_Profiler p;
 ga_gf_add_curves(r,p);int n=0;try{for(;;n++)p.Curve(n+1);}catch(const Standard_Failure&){}
 ga_gf_profiler_report(p,n);
 if(perform){GA_TRY(p.Perform(ptol));ga_gf_profiler_report(p,n);}
 ga_end();
}
// The section generator report: shape, knots, mults, sections and parameters.
void ga_gf_secgen_report(GeomFill_SectionGenerator& g,int n,bool params){
 int np=1,nk=1,deg=0,np2=0;
 GA_TRY(g.GetShape(np,nk,deg,np2);ga_int(np);ga_int(nk);ga_int(deg);ga_int(np2));
 GA_TRY(NCollection_Array1<double> k(1,nk);g.Knots(k);ga_int(k.Lower());for(double x:k)ga_real(x));
 GA_TRY(NCollection_Array1<int> m(1,nk);g.Mults(m);for(int x:m)ga_int(x));
 for(int i=0;i<=n+1;i++){
  NCollection_Array1<gp_Pnt> p(1,np);NCollection_Array1<gp_Vec> dp(1,np);NCollection_Array1<gp_Pnt2d> p2(1,1);
  NCollection_Array1<gp_Vec2d> dp2(1,1);NCollection_Array1<double> w(1,np),dw(1,np);
  p2(1)=gp_Pnt2d(7.,8.);
  GA_TRY(ga_flag(g.Section(i,p,dp,p2,dp2,w,dw));ga_int(p.Length());for(const gp_Pnt& x:p)ga_pnt(x);for(double x:w)ga_real(x);ga_pnt2d(p2(1)));
  if(params)GA_TRY(ga_real(g.Parameter(i)));
 }
}
// `ptol lower nparams params... curves...`: a generator, performed, with `SetParam` on
// an array of the given lower bound when nparams > 0.
void ga_gfill_secgen_case(std::istream& input){
 GaReader r{input};double ptol=r.real();int lower=r.integer();std::vector<double> prm=r.reals();
 GeomFill_SectionGenerator g;ga_gf_add_curves(r,g);int n=0;try{for(;;n++)g.Curve(n+1);}catch(const Standard_Failure&){}
 GA_TRY(g.Perform(ptol));
 occ::handle<NCollection_HArray1<double>> h;
 if(!prm.empty()){
  h=new NCollection_HArray1<double>(lower,lower+int(prm.size())-1);for(size_t i=0;i<prm.size();i++)h->SetValue(lower+int(i),prm[i]);
  GA_TRY(g.SetParam(h));for(double x:h->Array1())ga_real(x);
 }
 ga_gf_secgen_report(g,n,!prm.empty());
 ga_end();
}
// The AppSurf report (the AppBlend_Approx queries).
void ga_gf_appsurf_report(const GeomFill_AppSurf& s){
 ga_flag(s.IsDone());
 GA_TRY(int ud,vd,nu,nv,nku,nkv;s.SurfShape(ud,vd,nu,nv,nku,nkv);ga_int(ud);ga_int(vd);ga_int(nu);ga_int(nv);ga_int(nku);ga_int(nkv));
 GA_TRY(const NCollection_Array2<gp_Pnt>& p=s.SurfPoles();const NCollection_Array2<double>& w=s.SurfWeights();
  for(int i=p.LowerRow();i<=p.UpperRow();i++)for(int j=p.LowerCol();j<=p.UpperCol();j++){ga_pnt(p(i,j));ga_real(w(i,j));});
 GA_TRY(for(double x:s.SurfUKnots())ga_real(x);for(int m:s.SurfUMults())ga_int(m));
 GA_TRY(for(double x:s.SurfVKnots())ga_real(x);for(int m:s.SurfVMults())ga_int(m));
 GA_TRY(ga_int(s.UDegree());ga_int(s.VDegree()));
 GA_TRY(double t3,t2;s.TolReached(t3,t2);ga_real(t3);ga_real(t2));
 GA_TRY(ga_int(s.NbCurves2d()));GA_TRY(ga_int(s.Curves2dDegree()));
}
// `ptol nparams params... mode dmin dmax tol3d tol2d nbit knownp nbmaxp nlin curves...`:
// the skinning of the sections through GeomFill_AppSurf on a line of `nlin` points
// (mode 0 Perform, 1 Perform(SpApprox), 2 PerformSmoothing, 3 Perform(NbMaxP)), twice.
void ga_gfill_appsurf_case(std::istream& input){
 GaReader r{input};double ptol=r.real();std::vector<double> prm=r.reals();int mode=r.integer(),dmin=r.integer(),dmax=r.integer();
 double t3=r.real(),t2=r.real();int nbit=r.integer();bool knownp=r.flag();int nbmaxp=r.integer(),nlin=r.integer();
 GeomFill_SectionGenerator g;ga_gf_add_curves(r,g);GA_TRY(g.Perform(ptol));
 if(!prm.empty()){
  occ::handle<NCollection_HArray1<double>> h=new NCollection_HArray1<double>(1,int(prm.size()));
  for(size_t i=0;i<prm.size();i++)h->SetValue(int(i)+1,prm[i]);GA_TRY(g.SetParam(h));
 }
 occ::handle<GeomFill_Line> line=new GeomFill_Line(nlin);ga_int(line->NbPoints());
 GeomFill_AppSurf s(dmin,dmax,t3,t2,nbit,knownp);
 for(int round=0;round<2;round++){
  GA_TRY(if(mode==0)s.Perform(line,g,false);else if(mode==1)s.Perform(line,g,true);else if(mode==2)s.PerformSmoothing(line,g);
   else s.Perform(line,g,nbmaxp));
  ga_gf_appsurf_report(s);
 }
 ga_end();
}
// `n i...`: a line of n points and the points of some indices, and the default line.
void ga_gfill_line_case(std::istream& input){
 GaReader r{input};int n=r.integer();std::vector<double> idx=r.reals();
 occ::handle<GeomFill_Line> line=new GeomFill_Line(n);ga_int(line->NbPoints());
 for(double i:idx)ga_int(line->Point(int(i)));
 occ::handle<GeomFill_Line> empty=new GeomFill_Line();ga_int(empty->NbPoints());
 ga_end();
}
