// IntPatch quadric-parametric intersection probe operations (tkgeomalgo_probe.cxx); the Rust
// twin is crates/tkgeomalgo/examples/ga_probe/int_patch_impprm.rs:
// IntPatch_ImpPrmIntersection and IntPatch_RstInt. Needs ga_ics_surface
// (tkgeomalgo_probe_intcurvesurface.cxx), the printers of
// tkgeomalgo_probe_intpatch_report.cxx and ga_ipl_pnt2s
// (tkgeomalgo_probe_intpatch_impimp.cxx).
#include <IntPatch_ImpPrmIntersection.hxx>
#include <IntPatch_RstInt.hxx>
// The results of an intersection, each consultation member guarded.
void ga_ipl_impprm(const GaIplDomains& dm,const IntPatch_ImpPrmIntersection& x){
 ga_flag(x.IsDone());GA_TRY(ga_flag(x.IsEmpty()));
 GA_TRY(int n=x.NbPnts();ga_int(n);for(int i=0;i<=n+1;++i)GA_TRY(ga_ipl_point(dm,x.Point(i))));
 GA_TRY(int n=x.NbLines();ga_int(n);for(int i=0;i<=n+1;++i)GA_TRY(ga_ipl_line(dm,x.Line(i))));
}
// `s1 s2 tolarc toltang fleche pas mode u v`: the intersection of a quadric and a
// parametric patch on the domains of their natural restrictions (after a default one, not
// done): mode 0 the constructor, 1 Perform, 2 Perform from the start point `u v`, 3 Perform
// twice on the same object.
void ga_ip_impprm_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r),s2=ga_ics_surface(r);
  double tolarc=r.real(),toltang=r.real(),fleche=r.real(),pas=r.real();int mode=r.integer();double u=r.real(),v=r.real();
  occ::handle<Adaptor3d_TopolTool> d1=new Adaptor3d_TopolTool(s1),d2=new Adaptor3d_TopolTool(s2);GaIplDomains dm(d1,d2);
  IntPatch_ImpPrmIntersection empty;ga_ipl_impprm(dm,empty);
  if(mode==0){IntPatch_ImpPrmIntersection x(s1,d1,s2,d2,tolarc,toltang,fleche,pas);ga_ipl_impprm(dm,x);}
  else{IntPatch_ImpPrmIntersection x;if(mode==2)x.SetStartPoint(u,v);
   GA_TRY(x.Perform(s1,d1,s2,d2,tolarc,toltang,fleche,pas));ga_ipl_impprm(dm,x);
   if(mode==3){GA_TRY(x.Perform(s1,d1,s2,d2,tolarc,toltang,fleche,pas));ga_ipl_impprm(dm,x);}});
 ga_end();
}
// `surf other onfirst tol kind arc ends twice n (pnt2s)*n`: IntPatch_RstInt::PutVertexOnLine
// on a line of the given points (the parameters on the first surface first): kind 0 a
// walking line, 1 a restriction line on the arc of rank `arc` (from 1, 0 none) of the domain
// of `other`; `ends` adds the end vertices first; `twice` runs it a second time.
void ga_ip_rstint_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r),o=ga_ics_surface(r);bool onfirst=r.flag();double tol=r.real();
  int kind=r.integer(),arc=r.integer();bool ends=r.flag(),twice=r.flag();int n=r.integer();
  occ::handle<IntSurf_LineOn2S> pts=new IntSurf_LineOn2S();for(int i=0;i<n;++i)pts->Add(ga_ipl_pnt2s(r));
  occ::handle<Adaptor3d_TopolTool> ds=new Adaptor3d_TopolTool(s),dother=new Adaptor3d_TopolTool(o);
  GaIplDomains dm(onfirst?ds:dother,onfirst?dother:ds);
  occ::handle<IntPatch_PointLine> L;
  if(kind==0)L=new IntPatch_WLine(pts,false,IntSurf_In,IntSurf_Out);
  else{occ::handle<IntPatch_RLine> rl=new IntPatch_RLine(false,IntSurf_In,IntSurf_Out);rl->Add(pts);
   int k=1;for(dother->Init();dother->More();dother->Next(),++k)if(k==arc){if(onfirst)rl->SetArcOnS2(dother->Value());else rl->SetArcOnS1(dother->Value());}
   L=rl;}
  if(ends&&n>0){for(int e=0;e<2;++e){int i=e==0?1:n;IntPatch_Point p;p.SetValue(pts->Value(i).Value(),tol,false);
    double a,b,c,d;pts->Value(i).Parameters(a,b,c,d);p.SetParameters(a,b,c,d);p.SetParameter(i);L->AddVertex(p);}
   if(kind==0){occ::down_cast<IntPatch_WLine>(L)->SetFirstPoint(1);occ::down_cast<IntPatch_WLine>(L)->SetLastPoint(2);}
   else{occ::down_cast<IntPatch_RLine>(L)->SetFirstPoint(1);occ::down_cast<IntPatch_RLine>(L)->SetLastPoint(2);}}
  GA_TRY(IntPatch_RstInt::PutVertexOnLine(L,s,ds,o,onfirst,tol));ga_ipl_line(dm,L);
  if(twice){GA_TRY(IntPatch_RstInt::PutVertexOnLine(L,s,ds,o,onfirst,tol));ga_ipl_line(dm,L);});
 ga_end();
}
