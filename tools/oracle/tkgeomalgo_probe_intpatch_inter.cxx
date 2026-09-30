// IntPatch_Intersection and IntPatch_PrmPrmIntersection probe operations
// (tkgeomalgo_probe.cxx); the Rust twin is crates/tkgeomalgo/examples/ga_probe/int_patch_inter.rs.
// Two surfaces (ga_ics_surface) with their domains (ga_wt_domain) run through one Perform
// overload; the lines and points it gives are printed. Needs
// tkgeomalgo_probe_intpatch_wltool.cxx.
#include <IntPatch_Intersection.hxx>
#include <IntPatch_Polyhedron.hxx>
#include <NCollection_DynamicArray.hxx>
// The results of an intersection.
void ga_in_report(const GaIpFixture& fx,const IntPatch_Intersection& in){
 ga_flag(in.IsDone());GA_TRY(ga_flag(in.IsEmpty()));GA_TRY(ga_flag(in.TangentFaces()));GA_TRY(ga_flag(in.OppositeFaces()));
 GA_TRY(int n=in.NbPnts();ga_int(n);for(int i=1;i<=n;++i)ga_ip_point(fx,in.Point(i)));
 GA_TRY(int n=in.NbLines();ga_int(n);for(int i=1;i<=n;++i)ga_wt_report(fx,in.Line(i));GA_TRY(in.Line(0));GA_TRY(in.Line(n+1)));
 ga_int(in.SequenceOfLine().Length());}
// `s1 dom1 s2 dom2 tolarc toltang settol [4 tolerances] kind ...`: kind 0 Perform with the
// flags `geomint keeprline postwl`, 1 Perform with `n (u1 v1 u2 v2)*` start points and the
// flags, 2 Perform from `u1 v1 u2 v2`, 3 Perform of S1 alone, 4 the constructor of two
// surfaces, 5 the constructor of S1 alone.
void ga_ip_inter_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);occ::handle<Adaptor3d_TopolTool> d1=ga_wt_domain(r,fx,s1);
  occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);occ::handle<Adaptor3d_TopolTool> d2=ga_wt_domain(r,fx,s2);
  double ta=r.real(),tt=r.real();bool set=r.flag();double t[4]={0,0,0,0};if(set)for(double& x:t)x=r.real();int kind=r.integer();
  if(kind==4){IntPatch_Intersection in(s1,d1,s2,d2,ta,tt);ga_in_report(fx,in);ga_end();return;}
  if(kind==5){IntPatch_Intersection in(s1,d1,ta,tt);ga_in_report(fx,in);ga_end();return;}
  IntPatch_Intersection in;if(set)in.SetTolerances(t[0],t[1],t[2],t[3]);
  if(kind==0){bool g=r.flag(),k=r.flag(),p=r.flag();GA_TRY(in.Perform(s1,d1,s2,d2,ta,tt,g,k,p));}
  else if(kind==1){NCollection_List<IntSurf_PntOn2S> pts;int n=r.integer();
   for(int i=0;i<n;++i){double a=r.real(),b=r.real(),c=r.real(),d=r.real();IntSurf_PntOn2S q;q.SetValue(a,b,c,d);pts.Append(q);}
   bool g=r.flag(),k=r.flag(),p=r.flag();GA_TRY(in.Perform(s1,d1,s2,d2,ta,tt,pts,g,k,p));ga_int(pts.Size());}
  else if(kind==2){double a=r.real(),b=r.real(),c=r.real(),d=r.real();GA_TRY(in.Perform(s1,d1,s2,d2,a,b,c,d,ta,tt));}
  else{GA_TRY(in.Perform(s1,d1,ta,tt));}
  ga_in_report(fx,in));
 ga_end();
}
// `s1 dom1 s2 dom2 tol`: the static members CheckSingularPoints, DefineUVMaxStep and
// PrepareSurfaces.
void ga_ip_isingular_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);occ::handle<Adaptor3d_TopolTool> d1=ga_wt_domain(r,fx,s1);
  occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);occ::handle<Adaptor3d_TopolTool> d2=ga_wt_domain(r,fx,s2);double tol=r.real();
  double dist=0.;GA_TRY(ga_flag(IntPatch_Intersection::CheckSingularPoints(s1,d1,s2,dist));ga_real(dist));
  GA_TRY(ga_flag(IntPatch_Intersection::CheckSingularPoints(s2,d2,s1,dist));ga_real(dist));
  GA_TRY(ga_flag(IntPatch_Intersection::CheckSingularPoints(s1,d1,s1,dist));ga_real(dist));
  GA_TRY(ga_real(IntPatch_Intersection::DefineUVMaxStep(s1,d1,s2,d2)));
  GA_TRY(ga_real(IntPatch_Intersection::DefineUVMaxStep(s1,d1,s1,d1)));
  NCollection_DynamicArray<occ::handle<Adaptor3d_Surface>> v1,v2;
  GA_TRY(IntPatch_Intersection::PrepareSurfaces(s1,d1,s2,d2,tol,v1,v2));
  for(auto* v:{&v1,&v2}){ga_int(int(v->Size()));for(size_t i=0;i<v->Size();++i){const occ::handle<Adaptor3d_Surface>& s=(*v)(int(i));
   ga_real(s->FirstUParameter());ga_real(s->LastUParameter());ga_real(s->FirstVParameter());ga_real(s->LastVParameter());}});
 ga_end();
}
// `s1 dom1 s2 dom2 tt eps defl incr kind ...`: IntPatch_PrmPrmIntersection::Perform; kind 0
// the main overload `clearflag`, 1 `n (u1 v1 u2 v2)*` start points, 2 from `u1 v1 u2 v2`,
// 3 S1 alone, 4 the polyhedra of both surfaces `nu1 nv1 nu2 nv2` (0 for the automatic
// ones), 5 the polyhedron of S1 alone `nu nv`; then NewLine `line low high nb` when asked.
void ga_ip_prmprm_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);occ::handle<Adaptor3d_TopolTool> d1=ga_wt_domain(r,fx,s1);
  occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);occ::handle<Adaptor3d_TopolTool> d2=ga_wt_domain(r,fx,s2);
  double tt=r.real(),eps=r.real(),defl=r.real(),incr=r.real();int kind=r.integer();IntPatch_PrmPrmIntersection pp;
  if(kind==0){bool c=r.flag();GA_TRY(pp.Perform(s1,d1,s2,d2,tt,eps,defl,incr,c));}
  else if(kind==1){NCollection_List<IntSurf_PntOn2S> pts;int n=r.integer();
   for(int i=0;i<n;++i){double a=r.real(),b=r.real(),c=r.real(),d=r.real();IntSurf_PntOn2S q;q.SetValue(a,b,c,d);pts.Append(q);}
   GA_TRY(pp.Perform(s1,d1,s2,d2,tt,eps,defl,incr,pts));ga_int(pts.Size());}
  else if(kind==2){double a=r.real(),b=r.real(),c=r.real(),d=r.real();GA_TRY(pp.Perform(s1,d1,s2,d2,a,b,c,d,tt,eps,defl,incr));}
  else if(kind==3){GA_TRY(pp.Perform(s1,d1,tt,eps,defl,incr));}
  else if(kind==4){int a=r.integer(),b=r.integer(),c=r.integer(),d=r.integer();
   GA_TRY(if(a>0&&c>0){IntPatch_Polyhedron p1(s1,a,b),p2(s2,c,d);pp.Perform(s1,p1,d1,s2,p2,d2,tt,eps,defl,incr);}
    else if(a>0){IntPatch_Polyhedron p1(s1,a,b);pp.Perform(s1,p1,d1,s2,d2,tt,eps,defl,incr);}
    else if(c>0){IntPatch_Polyhedron p2(s2,c,d);pp.Perform(s1,d1,s2,p2,d2,tt,eps,defl,incr);}
    else{IntPatch_Polyhedron p1(s1),p2(s2);pp.Perform(s1,p1,d1,s2,p2,d2,tt,eps,defl,incr);});}
  else{int a=r.integer(),b=r.integer();GA_TRY(IntPatch_Polyhedron p1(s1,a,b);pp.Perform(s1,p1,d1,tt,eps,defl,incr));}
  ga_flag(pp.IsDone());GA_TRY(ga_flag(pp.IsEmpty()));
  GA_TRY(int n=pp.NbLines();ga_int(n);for(int i=1;i<=n;++i)ga_wt_report(fx,pp.Line(i));GA_TRY(pp.Line(0));GA_TRY(pp.Line(n+1)));
  int nl=r.integer();for(int k=0;k<nl;++k){int l=r.integer(),lo=r.integer(),hi=r.integer(),nb=r.integer();
   GA_TRY(ga_wt_report(fx,pp.NewLine(s1,s2,l,lo,hi,nb)));});
 ga_end();
}
