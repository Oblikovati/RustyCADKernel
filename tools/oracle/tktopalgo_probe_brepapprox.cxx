// BRepApprox probe operations (docs/topalgo-design.md section 12): the ApproxInt instances
// over BRepAdaptor_Surface and BRepApprox_ApproxLine. The walking lines are those of
// IntPatch_Intersection on two faces read by tp_face (tktopalgo_probe_face.cxx) with
// their BRepTopAdaptor_TopolTool domains. The Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_approx.rs (with brep_approx_parts.rs). Needs
// ga_iw_pnt2s (tkgeomalgo_probe_intwalk.cxx), ga_ip_captured
// (tkgeomalgo_probe_intpatch.cxx) and ga_intsurf_quadric (tkgeomalgo_probe_intsurf.cxx).
#include <AppParCurves_MultiBSpCurve.hxx>
#include <BRepApprox_Approx.hxx>
#include <BRepApprox_ApproxLine.hxx>
#include <BRepApprox_SurfaceTool.hxx>
#include <BRepApprox_TheImpPrmSvSurfacesOfApprox.hxx>
#include <BRepApprox_TheInt2SOfThePrmPrmSvSurfacesOfApprox.hxx>
#include <BRepApprox_TheMultiLineOfApprox.hxx>
#include <BRepApprox_TheMultiLineToolOfApprox.hxx>
#include <BRepApprox_ThePrmPrmSvSurfacesOfApprox.hxx>
#include <BRepApprox_TheZerImpFuncOfTheImpPrmSvSurfacesOfApprox.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <IntPatch_WLine.hxx>
// A face and its adaptor (restricted or not): `face restr`.
struct TpBraFace{occ::handle<BRepAdaptor_Surface> s;occ::handle<BRepTopAdaptor_TopolTool> d;};
TpBraFace tp_bra_face(GaReader& r){
 TpFace f=tp_face(r);bool restr=r.flag();occ::handle<BRepAdaptor_Surface> s=new BRepAdaptor_Surface(f.face,restr);
 return TpBraFace{s,new BRepTopAdaptor_TopolTool(s)};}
// `face1 restr1 face2 restr2 tolarc toltang geomint`: the walking lines of the
// intersection of the two faces (their count is printed).
struct TpBraWalk{TpBraFace a,b;std::vector<occ::handle<IntPatch_WLine>> lines;};
void tp_bra_walk(GaReader& r,TpBraWalk& w){
 w.a=tp_bra_face(r);w.b=tp_bra_face(r);double ta=r.real(),tt=r.real();bool g=r.flag();
 IntPatch_Intersection in;in.Perform(w.a.s,w.a.d,w.b.s,w.b.d,ta,tt,g);
 for(int i=1;i<=in.NbLines();++i){occ::handle<IntPatch_WLine> l=occ::down_cast<IntPatch_WLine>(in.Line(i));if(!l.IsNull())w.lines.push_back(l);}
 ga_int(int(w.lines.size()));}
// The approximation line of walking line `k` (1-based), null past the last one.
occ::handle<BRepApprox_ApproxLine> tp_bra_line(const TpBraWalk& w,int k){
 if(k<1||k>int(w.lines.size()))return nullptr;return new BRepApprox_ApproxLine(w.lines[k-1]->Curve(),false);}
// The range `kind [a b]`: 0 all the points, 1 `a b`, 2 the defaults 0 0; false (and -1
// printed) for a null line.
bool tp_bra_range(GaReader& r,const occ::handle<BRepApprox_ApproxLine>& l,int& a,int& b){
 int kind=r.integer();a=0;b=0;if(kind==1){a=r.integer();b=r.integer();}
 if(l.IsNull()){ga_int(-1);return false;}if(kind==0){a=1;b=l->NbPnts();}return true;}
// A multi-curve B-spline: poles count, degree, knots, multiplicities and curves.
void tp_bra_mbsp(const AppParCurves_MultiBSpCurve& m){
 int np=m.NbPoles();ga_int(np);if(np==0)return;
 GA_TRY(ga_int(m.Degree()));
 GA_TRY(const NCollection_Array1<double>& k=m.Knots();ga_int(k.Length());for(double x:k)ga_real(x));
 GA_TRY(const NCollection_Array1<int>& mu=m.Multiplicities();for(int x:mu)ga_int(x));
 int nc=m.NbCurves();ga_int(nc);
 for(int c=1;c<=nc;++c){int d=m.Dimension(c);ga_int(d);
  if(d==3){NCollection_Array1<gp_Pnt> p(1,np);m.Curve(c,p);for(const gp_Pnt& x:p)ga_pnt(x);}
  else{NCollection_Array1<gp_Pnt2d> p(1,np);m.Curve(c,p);for(const gp_Pnt2d& x:p)ga_pnt2d(x);}}}
// `set [tol3d tol2d degmin degmax nbiter nbpntmax tangency partype]`: SetParameters.
void tp_bra_settings(GaReader& r,BRepApprox_Approx& a){
 if(!r.flag())return;double t3=r.real(),t2=r.real();int d0=r.integer(),d1=r.integer(),it=r.integer(),np=r.integer();bool tg=r.flag();
 a.SetParameters(t3,t2,d0,d1,it,np,tg,Approx_ParametrizationType(r.integer()));}
// The results of the last Perform.
void tp_bra_result(const BRepApprox_Approx& a){
 ga_flag(a.IsDone());ga_real(a.TolReached3d());ga_real(a.TolReached2d());ga_int(a.NbMultiCurves());GA_TRY(tp_bra_mbsp(a.Value(1)));}
// `walk settings xyz u1v1 u2v2 mode n (line range)...`: one BRepApprox_Approx for the
// given walking lines, mode 0 on the two faces, 1 on the line alone.
void tp_bra_approx_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpBraWalk w;tp_bra_walk(r,w);BRepApprox_Approx a;tp_bra_settings(r,a);bool x=r.flag(),u1=r.flag(),u2=r.flag();
  int mode=r.integer(),n=r.integer();
  for(int k=0;k<n;++k){occ::handle<BRepApprox_ApproxLine> l=tp_bra_line(w,r.integer());int lo=0,hi=0;
   if(!tp_bra_range(r,l,lo,hi))continue;ga_int(l->NbPnts());
   if(mode==0){GA_TRY(a.Perform(*w.a.s,*w.b.s,l,x,u1,u2,lo,hi));}else{GA_TRY(a.Perform(l,x,u1,u2,lo,hi));}
   tp_bra_result(a);});
 ga_end();}
// `present [degree n (x y z)... nk (knot mult)...]`: a B-spline curve, or null.
occ::handle<Geom_BSplineCurve> tp_bra_bspline(GaReader& r){
 if(!r.flag())return nullptr;int deg=r.integer(),n=r.integer();NCollection_Array1<gp_Pnt> p(1,n);for(gp_Pnt& x:p)x=r.pnt();
 int nk=r.integer();NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);for(int i=1;i<=nk;++i){k(i)=r.real();m(i)=r.integer();}
 return new Geom_BSplineCurve(p,k,m,deg);}
// `present [degree n (u v)... nk (knot mult)...]`: a 2d B-spline curve, or null.
occ::handle<Geom2d_BSplineCurve> tp_bra_bspline2d(GaReader& r){
 if(!r.flag())return nullptr;int deg=r.integer(),n=r.integer();NCollection_Array1<gp_Pnt2d> p(1,n);for(gp_Pnt2d& x:p)x=r.pnt2d();
 int nk=r.integer();NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);for(int i=1;i<=nk;++i){k(i)=r.real();m(i)=r.integer();}
 return new Geom2d_BSplineCurve(p,k,m,deg);}
// `kind (0 n (x y z u1 v1 u2 v2)... tang | 1 curve3d curve2d curve2d) settings xyz u1v1
// u2v2 range`: an approximation line of points or of curve poles, its points 0..n+1,
// then its approximation alone.
void tp_bra_line_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<BRepApprox_ApproxLine> l;
  if(r.integer()==0){int n=r.integer();occ::handle<IntSurf_LineOn2S> s=new IntSurf_LineOn2S();
   for(int i=0;i<n;++i){gp_Pnt p=r.pnt();double a=r.real(),b=r.real(),c=r.real(),d=r.real();IntSurf_PntOn2S q;q.SetValue(p,a,b,c,d);s->Add(q);}
   l=new BRepApprox_ApproxLine(s,r.flag());}
  else{occ::handle<Geom_BSplineCurve> c=tp_bra_bspline(r);occ::handle<Geom2d_BSplineCurve> c1=tp_bra_bspline2d(r);
   l=new BRepApprox_ApproxLine(c,c1,tp_bra_bspline2d(r));}
  int n=l->NbPnts();ga_int(n);for(int i=0;i<=n+1;++i)GA_TRY(ga_iw_pnt2s(l->Point(i)));
  BRepApprox_Approx a;tp_bra_settings(r,a);bool x=r.flag(),u1=r.flag(),u2=r.flag();int lo=0,hi=0;tp_bra_range(r,l,lo,hi);
  GA_TRY(a.Perform(l,x,u1,u2,lo,hi));tp_bra_result(a));
 ga_end();}
// The quadric of an adaptor of quadric kind (IntSurf_Quadric::SetValue).
IntSurf_Quadric tp_bra_quadric(const BRepAdaptor_Surface& s){
 IntSurf_Quadric q;switch(s.GetType()){case GeomAbs_Plane:q.SetValue(s.Plane());break;case GeomAbs_Cylinder:q.SetValue(s.Cylinder());break;
  case GeomAbs_Sphere:q.SetValue(s.Sphere());break;case GeomAbs_Cone:q.SetValue(s.Cone());break;default:break;}return q;}
// The SvSurfaces of kind 1 PrmPrm, 2 ImpPrm with the quadric of the first face first, 3
// ImpPrm with the quadric of the second face second; null otherwise. The ImpPrm function
// keeps a pointer to the quadric, so the quadric lives beside the SvSurfaces.
struct TpBraSv{IntSurf_Quadric q;std::unique_ptr<ApproxInt_SvSurfaces> sv;};
void tp_bra_sv(const TpBraWalk& w,int kind,TpBraSv& s){
 if(kind==1)s.sv.reset(new BRepApprox_ThePrmPrmSvSurfacesOfApprox(*w.a.s,*w.b.s));
 if(kind==2){s.q=tp_bra_quadric(*w.a.s);s.sv.reset(new BRepApprox_TheImpPrmSvSurfacesOfApprox(s.q,*w.b.s));}
 if(kind==3){s.q=tp_bra_quadric(*w.b.s);s.sv.reset(new BRepApprox_TheImpPrmSvSurfacesOfApprox(*w.a.s,s.q));}}
