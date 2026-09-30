// HelixGeom probe operations; the Rust twin is crates/tkhelix/examples/hx_probe/helix_geom.rs
// and tools/tkhelix_corpus_helixgeom.py spells the requests. A helix is `t1 t2 pitch r
// taper cw`; a continuity is its GeomAbs_Shape code (0 C0, 1 G1, 2 C1, 3 G2, 4 C2, 5 C3,
// 6 CN). Every curve is printed as a B-spline: null flag, then degree, periodic,
// rational, poles and knots with multiplicities.
#include <Adaptor3d_Curve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Curve.hxx>
#include <HelixGeom_BuilderHelix.hxx>
#include <HelixGeom_BuilderHelixCoil.hxx>
#include <HelixGeom_HelixCurve.hxx>
#include <HelixGeom_Tools.hxx>
#include <NCollection_Array1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Vec.hxx>
GeomAbs_Shape hx_shape(int code){return GeomAbs_Shape(code);}
void hx_vec(const gp_Vec& v){ga_xyz(v.XYZ());}
// Null flag, then degree, periodic, rational, poles (weights) and knots with multiplicities.
void hx_bspline(const occ::handle<Geom_BSplineCurve>& c){
 ga_flag(c.IsNull());if(c.IsNull())return;
 ga_int(c->Degree());ga_flag(c->IsPeriodic());ga_flag(c->IsRational());ga_int(c->NbPoles());
 for(int i=1;i<=c->NbPoles();++i){ga_pnt(c->Pole(i));if(c->IsRational())ga_real(c->Weight(i));}
 ga_int(c->NbKnots());for(int i=1;i<=c->NbKnots();++i){ga_real(c->Knot(i));ga_int(c->Multiplicity(i));}}
// A curve of a builder: the B-spline flag (the builders only make B-splines), its range
// and its B-spline data.
void hx_curve(const occ::handle<Geom_Curve>& c){
 occ::handle<Geom_BSplineCurve> b=occ::down_cast<Geom_BSplineCurve>(c);ga_flag(!b.IsNull());
 if(!c.IsNull()){ga_real(c->FirstParameter());ga_real(c->LastParameter());}hx_bspline(b);}
// The evaluations of one parameter u: D0, D1, D2, D3 (not implemented) and DN(u, n).
void hx_evaluate(const HelixGeom_HelixCurve& h,double u,int n){
 GA_TRY(ga_pnt(h.Value(u)))
 GA_TRY(gp_Pnt p;gp_Vec v1;h.D1(u,p,v1);ga_pnt(p);hx_vec(v1))
 GA_TRY(gp_Pnt p;gp_Vec v1,v2;h.D2(u,p,v1,v2);ga_pnt(p);hx_vec(v1);hx_vec(v2))
 GA_TRY(gp_Pnt p;gp_Vec v1,v2,v3;h.D3(u,p,v1,v2,v3);ga_pnt(p))
 GA_TRY(hx_vec(h.DN(u,n)))}
// The adaptor queries: range, continuity, intervals into an array of bounds [lo, up],
// and the members that raise.
void hx_queries(const HelixGeom_HelixCurve& h,int lo,int up){
 ga_real(h.FirstParameter());ga_real(h.LastParameter());ga_int(int(h.Continuity()));
 ga_int(h.NbIntervals(GeomAbs_C2));
 GA_TRY(NCollection_Array1<double> t(lo,up);t.Init(-1.);h.Intervals(t,GeomAbs_C0);for(double x:t)ga_real(x))
 GA_TRY(ga_real(h.Resolution(1.)))GA_TRY(ga_flag(h.IsClosed()))GA_TRY(ga_flag(h.IsPeriodic()))
 GA_TRY(ga_real(h.Period()))}
// `nloads (mode helix)... lo up n nu (u)...`: mode 0 Load(helix), 1 Load() after
// storing nothing (the helix fields are read and ignored); each load prints its exception
// type or nothing; then the queries and the evaluations of every u.
void hx_curve_case(std::istream& input){
 GaReader r{input};HelixGeom_HelixCurve h;int nloads=r.integer();
 for(int k=0;k<nloads;++k){int mode=r.integer();double t1=r.real(),t2=r.real(),p=r.real(),rs=r.real(),ta=r.real();bool cw=r.flag();
  if(mode==0){GA_TRY(h.Load(t1,t2,p,rs,ta,cw))}else{GA_TRY(h.Load())}}
 int lo=r.integer(),up=r.integer(),n=r.integer(),nu=r.integer();hx_queries(h,lo,up);
 for(int k=0;k<nu;++k)hx_evaluate(h,r.real(),n);
 ga_end();}
// The adaptor of an ApprCurve3D request: 0 a helix, 1 a GeomAdaptor_Curve on the circle
// of radius R in the XY plane over [f, l], 2 the Adaptor3d_Curve base class itself.
occ::handle<Adaptor3d_Curve> hx_adaptor(GaReader& r){
 int kind=r.integer();
 if(kind==0){double t1=r.real(),t2=r.real(),p=r.real(),rs=r.real(),ta=r.real();bool cw=r.flag();
  HelixGeom_HelixCurve h;h.Load(t1,t2,p,rs,ta,cw);return new HelixGeom_HelixCurve(h);}
 if(kind==1){double rad=r.real(),f=r.real(),l=r.real();
  return new GeomAdaptor_Curve(new Geom_Circle(gp_Ax2(),rad),f,l);}
 return new Adaptor3d_Curve();}
// `adaptor tol cont maxseg maxdeg`: HelixGeom_Tools::ApprCurve3D; the code, the maximum
// error (initialized to -7) and the B-spline (initially null).
void hx_appr_curve3d_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Adaptor3d_Curve> a=hx_adaptor(r);double tol=r.real();int cont=r.integer(),seg=r.integer(),deg=r.integer();
  occ::handle<Geom_BSplineCurve> b;double err=-7.;
  int code=HelixGeom_Tools::ApprCurve3D(a,tol,hx_shape(cont),seg,deg,b,err);
  ga_int(code);ga_real(err);hx_bspline(b))
 ga_end();}
// `helix tol`: HelixGeom_Tools::ApprHelix, printed as hx_appr_curve3d.
void hx_appr_helix_case(std::istream& input){
 GaReader r{input};double t1=r.real(),t2=r.real(),p=r.real(),rs=r.real(),ta=r.real();bool cw=r.flag();double tol=r.real();
 GA_TRY(occ::handle<Geom_BSplineCurve> b;double err=-7.;
  int code=HelixGeom_Tools::ApprHelix(t1,t2,p,rs,ta,cw,tol,b,err);ga_int(code);ga_real(err);hx_bspline(b))
 ga_end();}
// The state of a builder: statuses, settings, reached tolerance and curves.
void hx_builder_state(const HelixGeom_BuilderHelixGen& b){
 GeomAbs_Shape cont;int deg,seg;b.ApproxParameters(cont,deg,seg);
 double t1,t2,p,rs,ta;bool cw;b.CurveParameters(t1,t2,p,rs,ta,cw);
 ga_int(b.ErrorStatus());ga_int(b.WarningStatus());ga_int(int(cont));ga_int(deg);ga_int(seg);
 ga_real(b.Tolerance());ga_real(t1);ga_real(t2);ga_real(p);ga_real(rs);ga_real(ta);ga_flag(cw);
 ga_real(b.ToleranceReached());ga_int(b.Curves().Length());for(const occ::handle<Geom_Curve>& c:b.Curves())hx_curve(c);}
// One run of a builder: `set tol cont maxdeg maxseg helix`, with set 1 applying the
// settings and the helix before Perform (0 performs again unchanged).
void hx_builder_run(GaReader& r,HelixGeom_BuilderHelixGen& b){
 int set=r.integer();double tol=r.real();int cont=r.integer(),deg=r.integer(),seg=r.integer();
 double t1=r.real(),t2=r.real(),p=r.real(),rs=r.real(),ta=r.real();bool cw=r.flag();
 if(set){b.SetTolerance(tol);b.SetApproxParameters(hx_shape(cont),deg,seg);b.SetCurveParameters(t1,t2,p,rs,ta,cw);}
 GA_TRY(b.Perform())hx_builder_state(b);}
// `nruns (run)...`: HelixGeom_BuilderHelixCoil, the default state first, then each run
// on the same builder.
void hx_coil_case(std::istream& input){
 GaReader r{input};HelixGeom_BuilderHelixCoil b;hx_builder_state(b);int n=r.integer();
 for(int k=0;k<n;++k)hx_builder_run(r,b);
 ga_end();}
// `P N X nruns (run)...`: HelixGeom_BuilderHelix at gp_Ax2(P, N, X), printed as hx_coil
// with the position after the default state.
void hx_helix_case(std::istream& input){
 GaReader r{input};gp_Pnt o=r.pnt();gp_Dir n=r.dir();gp_Dir x=r.dir();HelixGeom_BuilderHelix b;hx_builder_state(b);
 b.SetPosition(gp_Ax2(o,n,x));ga_pnt(b.Position().Location());ga_dir(b.Position().Direction());
 ga_dir(b.Position().XDirection());int k=r.integer();for(int i=0;i<k;++i)hx_builder_run(r,b);
 ga_end();}
