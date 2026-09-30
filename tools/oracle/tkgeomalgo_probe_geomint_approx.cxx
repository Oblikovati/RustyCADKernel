// ApproxInt and GeomInt_WLApprox probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom_int_approx.rs. The walking lines are those of the
// intersection of two adaptors (ga_gi_inter, tkgeomalgo_probe_geomint.cxx).
#include <AppParCurves_MultiBSpCurve.hxx>
#include <ApproxInt_KnotTools.hxx>
#include <GeomInt_TheImpPrmSvSurfacesOfWLApprox.hxx>
#include <GeomInt_TheMultiLineOfWLApprox.hxx>
#include <GeomInt_TheMultiLineToolOfWLApprox.hxx>
#include <GeomInt_ThePrmPrmSvSurfacesOfWLApprox.hxx>
#include <GeomInt_WLApprox.hxx>
#include <IntSurf_Quadric.hxx>
#include <NCollection_LocalArray.hxx>
// A multi-curve B-spline: the poles count, degree, knots, multiplicities and curves.
void ga_gi_mbsp(const AppParCurves_MultiBSpCurve& m){
 int np=m.NbPoles();ga_int(np);if(np==0)return;
 GA_TRY(ga_int(m.Degree()));
 GA_TRY(const NCollection_Array1<double>& k=m.Knots();ga_int(k.Length());for(double x:k)ga_real(x));
 GA_TRY(const NCollection_Array1<int>& mu=m.Multiplicities();for(int x:mu)ga_int(x));
 int nc=m.NbCurves();ga_int(nc);
 for(int c=1;c<=nc;++c){int d=m.Dimension(c);ga_int(d);
  if(d==3){NCollection_Array1<gp_Pnt> p(1,np);m.Curve(c,p);for(const gp_Pnt& x:p)ga_pnt(x);}
  else{NCollection_Array1<gp_Pnt2d> p(1,np);m.Curve(c,p);for(const gp_Pnt2d& x:p)ga_pnt2d(x);}}}
// The walking line `k` (1-based) among the lines of the intersection, or null.
occ::handle<IntPatch_WLine> ga_gi_wline(const GaGiInter& g,int k){
 int seen=0;for(int i=1;i<=g.in.NbLines();++i){occ::handle<IntPatch_WLine> w=occ::down_cast<IntPatch_WLine>(g.in.Line(i));
  if(!w.IsNull()&&++seen==k)return w;}return nullptr;}
// The range `kind [a b]` of a line: 0 all its points, 1 `a b`, 2 the defaults 0 0; false
// for a null line (the range is read all the same).
bool ga_gi_range(GaReader& r,const occ::handle<IntPatch_WLine>& w,int& a,int& b){
 int kind=r.integer();a=0;b=0;if(kind==1){a=r.integer();b=r.integer();}
 if(w.IsNull()){ga_int(-1);return false;}if(kind==0){a=1;b=w->NbPnts();}return true;}
// `s1 s2 tol set [tol3d tol2d degmin degmax nbiter nbpntmax tangency partype] xyz u1v1 u2v2
// mode nlines (range)*`: GeomInt_WLApprox of the first `nlines` walking lines of the
// intersection (one object for all), mode 0 with the surfaces, 1 on the line alone.
void ga_gi_wlapprox_case(std::istream& input){
 GaReader r{input};
 GA_TRY(GaGiInter g;ga_gi_inter(r,g);GeomInt_WLApprox a;bool set=r.flag();
  if(set){double t3=r.real(),t2=r.real();int d0=r.integer(),d1=r.integer(),it=r.integer(),np=r.integer();bool tg=r.flag();
   a.SetParameters(t3,t2,d0,d1,it,np,tg,Approx_ParametrizationType(r.integer()));}
  bool x=r.flag(),u1=r.flag(),u2=r.flag();int mode=r.integer(),nl=r.integer();
  for(int k=1;k<=nl;++k){occ::handle<IntPatch_WLine> w=ga_gi_wline(g,k);int lo=0,hi=0;
   if(!ga_gi_range(r,w,lo,hi))continue;ga_int(w->NbPnts());
   if(mode==0){GA_TRY(a.Perform(g.s1,g.s2,w,x,u1,u2,lo,hi));}else{GA_TRY(a.Perform(w,x,u1,u2,lo,hi));}
   ga_flag(a.IsDone());ga_real(a.TolReached3d());ga_real(a.TolReached2d());ga_int(a.NbMultiCurves());GA_TRY(ga_gi_mbsp(a.Value(1)));});
 ga_end();
}
// The quadric of an adaptor of quadric kind (IntSurf_Quadric::SetValue).
IntSurf_Quadric ga_gi_quadric(const occ::handle<Adaptor3d_Surface>& s){
 IntSurf_Quadric q;switch(s->GetType()){case GeomAbs_Plane:q.SetValue(s->Plane());break;case GeomAbs_Cylinder:q.SetValue(s->Cylinder());break;
  case GeomAbs_Sphere:q.SetValue(s->Sphere());break;case GeomAbs_Cone:q.SetValue(s->Cone());break;case GeomAbs_Torus:q.SetValue(s->Torus());break;
  default:break;}return q;}
// The SvSurfaces of kind 1 PrmPrm, 2 ImpPrm with the quadric of S1 first, 3 ImpPrm with the
// quadric of S2 second; null otherwise. The ImpPrm function keeps a pointer to the quadric
// (and to the surface handle), so the quadric lives beside the SvSurfaces.
struct GaGiSv{IntSurf_Quadric q;std::unique_ptr<ApproxInt_SvSurfaces> sv;};
void ga_gi_sv(const GaGiInter& g,int kind,GaGiSv& s){
 if(kind==1)s.sv.reset(new GeomInt_ThePrmPrmSvSurfacesOfWLApprox(g.s1,g.s2));
 if(kind==2){s.q=ga_gi_quadric(g.s1);s.sv.reset(new GeomInt_TheImpPrmSvSurfacesOfWLApprox(s.q,g.s2));}
 if(kind==3){s.q=ga_gi_quadric(g.s2);s.sv.reset(new GeomInt_TheImpPrmSvSurfacesOfWLApprox(g.s1,s.q));}}
// The multipoint `i` of a multi-line (3d then 2d points, the tool's Value).
void ga_gi_ml_value(const GeomInt_TheMultiLineOfWLApprox& m,int i){
 int n3=m.NbP3d(),n2=m.NbP2d();NCollection_Array1<gp_Pnt> p(1,std::max(1,n3));NCollection_Array1<gp_Pnt2d> q(1,std::max(1,n2));
 GA_TRY(if(n3!=0&&n2!=0)GeomInt_TheMultiLineToolOfWLApprox::Value(m,i,p,q);else if(n2!=0)GeomInt_TheMultiLineToolOfWLApprox::Value(m,i,q);
  else if(n3!=0)GeomInt_TheMultiLineToolOfWLApprox::Value(m,i,p);for(const gp_Pnt& x:p)ga_pnt(x);for(const gp_Pnt2d& x:q)ga_pnt2d(x));}
// The tangents of the multipoint `i` (the tool's three Tangency members).
void ga_gi_ml_tangency(const GeomInt_TheMultiLineOfWLApprox& m,int i){
 int n3=std::max(1,m.NbP3d()),n2=std::max(1,m.NbP2d());
 NCollection_Array1<gp_Vec> v(1,n3);NCollection_Array1<gp_Vec2d> w(1,n2);
 GA_TRY(ga_flag(GeomInt_TheMultiLineToolOfWLApprox::Tangency(m,i,v));for(const gp_Vec& x:v)ga_xyz(x.XYZ()));
 GA_TRY(ga_flag(GeomInt_TheMultiLineToolOfWLApprox::Tangency(m,i,w));for(const gp_Vec2d& x:w)ga_xy(x.XY()));
 GA_TRY(ga_flag(GeomInt_TheMultiLineToolOfWLApprox::Tangency(m,i,v,w));for(const gp_Vec& x:v)ga_xyz(x.XYZ());for(const gp_Vec2d& x:w)ga_xy(x.XY()));
 GA_TRY(ga_flag(GeomInt_TheMultiLineToolOfWLApprox::Curvature(m,i,v)));}
// A multi-line: its range, sizes and status, and its multipoints (with one index outside at
// each end).
void ga_gi_ml_report(const GeomInt_TheMultiLineOfWLApprox& m){
 int f=GeomInt_TheMultiLineToolOfWLApprox::FirstPoint(m),l=GeomInt_TheMultiLineToolOfWLApprox::LastPoint(m);
 ga_int(f);ga_int(l);ga_int(GeomInt_TheMultiLineToolOfWLApprox::NbP3d(m));ga_int(GeomInt_TheMultiLineToolOfWLApprox::NbP2d(m));
 ga_int(GeomInt_TheMultiLineToolOfWLApprox::WhatStatus(m,f,l));for(int i=f-1;i<=l+1;++i)ga_gi_ml_value(m,i);}
// `s1 s2 tol svkind line nbp3d nbp2d u1v1 u2v2 xo yo zo u1o v1o u2o v2o p2donfirst range
// ntan (i)* nops (op args)*`: a multi-line on walking line `line` with the SvSurfaces of
// `svkind` (0 none); the tangents at the given indices; then op 1 MakeMLBetween `low high
// nb`, 2 MakeMLOneMorePoint `low high bad`, 3 Dump.
void ga_gi_mline_case(std::istream& input){
 GaReader r{input};
 GA_TRY(GaGiInter g;ga_gi_inter(r,g);int kind=r.integer();occ::handle<IntPatch_WLine> w=ga_gi_wline(g,r.integer());
  GaGiSv gs;ga_gi_sv(g,kind,gs);ApproxInt_SvSurfaces* sv=gs.sv.get();
  int n3=r.integer(),n2=r.integer();bool a1=r.flag(),a2=r.flag();double o[7];for(double& x:o)x=r.real();bool first=r.flag();
  int lo=0,hi=0;if(!ga_gi_range(r,w,lo,hi)){ga_end();return;}
  GeomInt_TheMultiLineOfWLApprox m=sv?GeomInt_TheMultiLineOfWLApprox(w,sv,n3,n2,a1,a2,o[0],o[1],o[2],o[3],o[4],o[5],o[6],first,lo,hi)
   :GeomInt_TheMultiLineOfWLApprox(w,n3,n2,a1,a2,o[0],o[1],o[2],o[3],o[4],o[5],o[6],first,lo,hi);
  ga_gi_ml_report(m);int nt=r.integer();for(int k=0;k<nt;++k)ga_gi_ml_tangency(m,r.integer());
  int nops=r.integer();
  for(int k=0;k<nops;++k){int op=r.integer();
   if(op==1){int a=r.integer(),b=r.integer(),c=r.integer();GA_TRY(ga_gi_ml_report(GeomInt_TheMultiLineToolOfWLApprox::MakeMLBetween(m,a,b,c)));}
   else if(op==2){int a=r.integer(),b=r.integer(),c=r.integer();GeomInt_TheMultiLineOfWLApprox other;
    GA_TRY(bool made=GeomInt_TheMultiLineToolOfWLApprox::MakeMLOneMorePoint(m,a,b,c,other);ga_flag(made);if(made)ga_gi_ml_report(other));}
   else{ga_ip_captured([&]{GeomInt_TheMultiLineToolOfWLApprox::Dump(m);});}}
  if(sv)ga_flag(sv->GetUseSolver()));
 ga_end();
}
// `s1 s2 tol svkind nops (op args)*`: the SvSurfaces of `svkind` (1 to 3); op 0 Compute, 1
// Pnt, 2 SeekPoint, 3 Tangency, 4 TangencyOnSurf1, 5 TangencyOnSurf2 on `u1 v1 u2 v2`,
// 6 SetUseSolver `flag`, 7 FillInitialVectorOfSolution `u1 v1 u2 v2` (ImpPrm only).
void ga_gi_sv_case(std::istream& input){
 GaReader r{input};
 GA_TRY(GaGiInter g;ga_gi_inter(r,g);int kind=r.integer();GaGiSv gs;ga_gi_sv(g,kind,gs);ApproxInt_SvSurfaces* sv=gs.sv.get();
  ga_flag(sv->GetUseSolver());int nops=r.integer();
  for(int k=0;k<nops;++k){int op=r.integer();
   if(op==6){sv->SetUseSolver(r.flag());ga_flag(sv->GetUseSolver());continue;}
   double u1=r.real(),v1=r.real(),u2=r.real(),v2=r.real();
   GA_TRY(if(op==0){gp_Pnt p;gp_Vec t;gp_Vec2d a,b;ga_flag(sv->Compute(u1,v1,u2,v2,p,t,a,b));ga_real(u1);ga_real(v1);ga_real(u2);ga_real(v2);
     ga_pnt(p);ga_xyz(t.XYZ());ga_xy(a.XY());ga_xy(b.XY());}
    else if(op==1){gp_Pnt p;sv->Pnt(u1,v1,u2,v2,p);ga_pnt(p);}
    else if(op==2){IntSurf_PntOn2S p;bool ok=sv->SeekPoint(u1,v1,u2,v2,p);ga_flag(ok);if(ok)ga_iw_pnt2s(p);}
    else if(op==3){gp_Vec t;ga_flag(sv->Tangency(u1,v1,u2,v2,t));ga_xyz(t.XYZ());}
    else if(op==4){gp_Vec2d t;ga_flag(sv->TangencyOnSurf1(u1,v1,u2,v2,t));ga_xy(t.XY());}
    else if(op==5){gp_Vec2d t;ga_flag(sv->TangencyOnSurf2(u1,v1,u2,v2,t));ga_xy(t.XY());}
    else{GeomInt_TheImpPrmSvSurfacesOfWLApprox* ip=dynamic_cast<GeomInt_TheImpPrmSvSurfacesOfWLApprox*>(sv);
     occ::handle<Adaptor3d_Surface> ps=(kind==2)?g.s2:g.s1;math_Vector x(1,2,0.);double tu=0,tv=0;
     bool ok=ip->FillInitialVectorOfSolution(u1,v1,u2,v2,ps->FirstUParameter(),ps->LastUParameter(),ps->FirstVParameter(),ps->LastVParameter(),x,tu,tv);
     ga_flag(ok);if(ok){ga_real(x(1));ga_real(x(2));ga_real(tu);ga_real(tv);}});});
 ga_end();
}
// The coordinates `n dim` values, the parameters `lower` and `n` values (math_Vector).
math_Vector ga_gi_pars(GaReader& r,int n){int lower=r.integer();math_Vector p(lower,lower+n-1);for(int i=p.Lower();i<=p.Upper();++i)p(i)=r.real();return p;}
// `mode ...`: ApproxInt_KnotTools; mode 0 BuildCurvature `dim n coords pars`, mode 1
// BuildKnots `n lower (x y z u1 v1 u2 v2)*n pars xyz u1v1 u2v2 minnb`.
void ga_gi_knots_case(std::istream& input){
 GaReader r{input};int mode=r.integer();
 GA_TRY(if(mode==0){int dim=r.integer(),n=r.integer();NCollection_LocalArray<double> c(size_t(n)*dim);for(int i=0;i<n*dim;++i)c[i]=r.real();
   math_Vector p=ga_gi_pars(r,n);NCollection_Array1<double> cv(p.Lower(),p.Upper());double mx=-1;
   ApproxInt_KnotTools::BuildCurvature(c,dim,p,cv,mx);for(double x:cv)ga_real(x);ga_real(mx);}
  else{int n=r.integer(),lower=r.integer();NCollection_Array1<gp_Pnt> x(lower,lower+n-1);NCollection_Array1<gp_Pnt2d> a(lower,lower+n-1),b(lower,lower+n-1);
   for(int i=x.Lower();i<=x.Upper();++i){x(i)=r.pnt();a(i)=r.pnt2d();b(i)=r.pnt2d();}
   math_Vector p(lower,lower+n-1);for(int i=p.Lower();i<=p.Upper();++i)p(i)=r.real();bool f1=r.flag(),f2=r.flag(),f3=r.flag();int mn=r.integer();
   NCollection_DynamicArray<int> k;k.Append(-7);ApproxInt_KnotTools::BuildKnots(x,a,b,p,f1,f2,f3,mn,k);ga_int(int(k.Size()));for(int v:k)ga_int(v);});
 ga_end();
}
// `s1 s2 tol xyz u1v1 u2v2 nlines (range)*`: ApproxInt_KnotTools::DefineParType on the
// walking lines of the intersection, and GeomInt_WLApprox::Parameters of their chord
// length, centripetal and iso-parametric kinds.
void ga_gi_partype_case(std::istream& input){
 GaReader r{input};
 GA_TRY(GaGiInter g;ga_gi_inter(r,g);bool x=r.flag(),u1=r.flag(),u2=r.flag();int nl=r.integer();
  for(int k=1;k<=nl;++k){occ::handle<IntPatch_WLine> w=ga_gi_wline(g,k);int lo=0,hi=0;if(!ga_gi_range(r,w,lo,hi))continue;
   GA_TRY(ga_int(ApproxInt_KnotTools::DefineParType(w,lo,hi,x,u1,u2)));
   int n3=x?1:0,n2=(u1?1:0)+(u2?1:0);GeomInt_TheMultiLineOfWLApprox m(w,n3,n2,u1,u2,0,0,0,0,0,0,0,u1,lo,hi);
   for(int t=0;t<3;++t)GA_TRY(math_Vector p(lo,hi);GeomInt_WLApprox::Parameters(m,lo,hi,Approx_ParametrizationType(t),p);
    for(int i=p.Lower();i<=p.Upper();++i)ga_real(p(i)));});
 ga_end();
}
