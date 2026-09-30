// IntPatch analytic intersection probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_patch_impimp.rs: IntPatch_ImpImpIntersection,
// IntPatch_ALineToWLine and IntPatch_SpecialPoints. Needs ga_ics_surface
// (tkgeomalgo_probe_intcurvesurface.cxx) and the printers of
// tkgeomalgo_probe_intpatch_report.cxx.
#include <IntPatch_ALineToWLine.hxx>
#include <IntPatch_ImpImpIntersection.hxx>
#include <IntPatch_SpecialPoints.hxx>
#include <math_Vector.hxx>
// The results of an intersection, each consultation member guarded.
void ga_ipl_impimp(const GaIplDomains& dm,const IntPatch_ImpImpIntersection& x){
 ga_int(x.GetStatus());ga_flag(x.IsDone());GA_TRY(ga_flag(x.IsEmpty()));GA_TRY(ga_flag(x.TangentFaces()));
 GA_TRY(ga_flag(x.OppositeFaces()));
 GA_TRY(int n=x.NbPnts();ga_int(n);for(int i=0;i<=n+1;++i)GA_TRY(ga_ipl_point(dm,x.Point(i))));
 GA_TRY(int n=x.NbLines();ga_int(n);for(int i=0;i<=n+1;++i)GA_TRY(ga_ipl_line(dm,x.Line(i))));
}
// `s1 s2 tolarc toltang keep`: the intersection of two quadric patches on the domains of
// their natural restrictions (after a default intersection, not done).
void ga_ip_impimp_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r),s2=ga_ics_surface(r);double tolarc=r.real(),toltang=r.real();bool keep=r.flag();
  occ::handle<Adaptor3d_TopolTool> d1=new Adaptor3d_TopolTool(s1),d2=new Adaptor3d_TopolTool(s2);GaIplDomains dm(d1,d2);
  IntPatch_ImpImpIntersection empty;ga_ipl_impimp(dm,empty);
  IntPatch_ImpImpIntersection x(s1,d1,s2,d2,tolarc,toltang,keep);ga_ipl_impimp(dm,x));
 ga_end();
}
// The protected members of IntPatch_ALineToWLine, made public for the probe.
class GaIplALineToWLine : public IntPatch_ALineToWLine{
public:
 GaIplALineToWLine(const occ::handle<Adaptor3d_Surface>& a,const occ::handle<Adaptor3d_Surface>& b,int n):IntPatch_ALineToWLine(a,b,n){}
 using IntPatch_ALineToWLine::CheckDeflection;using IntPatch_ALineToWLine::GetSectionRadius;
 using IntPatch_ALineToWLine::CorrectEndPoint;using IntPatch_ALineToWLine::StepComputing;
};
// The conversion of one analytic line: the lines of MakeWLine, or of the given range, and
// the protected members at points of the line.
void ga_ipl_convert(const GaIplDomains& dm,GaIplALineToWLine& c,const occ::handle<IntPatch_ALine>& a,bool ranged,double fp,double lp){
 NCollection_Sequence<occ::handle<IntPatch_Line>> lines;
 GA_TRY(if(ranged)c.MakeWLine(a,fp,lp,lines);else c.MakeWLine(a,lines));
 ga_int(lines.Length());for(int i=1;i<=lines.Length();++i)ga_ipl_line(dm,lines(i));
 bool inc;double f=0,l=0;GA_TRY(f=a->FirstParameter(inc);l=a->LastParameter(inc));
 for(int k=0;k<=4;++k)GA_TRY(gp_Pnt p=a->Value(f+(l-f)*k/4.);ga_real(c.GetSectionRadius(p));ga_int(c.CheckDeflection(p.XYZ(),c.Tol3D()));
  ga_int(c.CheckDeflection(p.XYZ()+gp_XYZ(1e-3,0,0),1e-5)));
 GA_TRY(gp_Pnt p;gp_Vec v;a->D1(0.5*(f+l),p,v);IntSurf_PntOn2S q;q.SetValue(p,0.,0.,0.,0.);
  double step=0.1;bool ok=c.StepComputing(a,q,l,0.5*(f+l),v.Magnitude(),1e-3,1.,c.Tol3D(),step);ga_flag(ok);ga_real(step));
 for(int i=1;i<=lines.Length();++i){occ::handle<IntPatch_WLine> w=occ::down_cast<IntPatch_WLine>(lines(i));
  if(w.IsNull()||w->NbPnts()<3)continue;occ::handle<IntSurf_LineOn2S> copy=new IntSurf_LineOn2S();
  for(int k=1;k<=w->NbPnts();++k)copy->Add(w->Point(k));
  GA_TRY(c.CorrectEndPoint(copy,1);c.CorrectEndPoint(copy,copy->NbPoints());ga_iw_pnt2s(copy->Value(1));ga_iw_pnt2s(copy->Value(copy->NbPoints())));}
}
// `s1 s2 tolarc toltang nbpoints tolopen toltrans tol3d ranged fpar lpar`: every analytic
// line of the intersection converted by IntPatch_ALineToWLine with these settings.
void ga_ip_aline2wline_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r),s2=ga_ics_surface(r);double tolarc=r.real(),toltang=r.real();int nb=r.integer();
  double topen=r.real(),ttrans=r.real(),t3d=r.real();bool ranged=r.flag();double fp=r.real(),lp=r.real();
  occ::handle<Adaptor3d_TopolTool> d1=new Adaptor3d_TopolTool(s1),d2=new Adaptor3d_TopolTool(s2);GaIplDomains dm(d1,d2);
  IntPatch_ImpImpIntersection x(s1,d1,s2,d2,tolarc,toltang);GaIplALineToWLine c(s1,s2,nb);
  ga_real(c.TolOpenDomain());ga_real(c.TolTransition());ga_real(c.Tol3D());
  c.SetTolOpenDomain(topen);c.SetTolTransition(ttrans);c.SetTol3D(t3d);ga_real(c.TolOpenDomain());ga_real(c.TolTransition());ga_real(c.Tol3D());
  int n=x.IsDone()?x.NbLines():0;ga_int(n);
  for(int i=1;i<=n;++i){occ::handle<IntPatch_ALine> a=occ::down_cast<IntPatch_ALine>(x.Line(i));ga_flag(a.IsNull());
   if(!a.IsNull())ga_ipl_convert(dm,c,a,ranged,fp,lp);});
 ga_end();
}
// The protected members of IntPatch_SpecialPoints, made public for the probe.
class GaIplSpecialPoints : public IntPatch_SpecialPoints{
public:
 using IntPatch_SpecialPoints::ProcessSphere;using IntPatch_SpecialPoints::ProcessCone;
 using IntPatch_SpecialPoints::GetTangentToIntLineForCone;
};
IntSurf_PntOn2S ga_ipl_pnt2s(GaReader& r){gp_Pnt x=r.pnt();double a=r.real(),b=r.real(),c=r.real();IntSurf_PntOn2S p;p.SetValue(x,a,b,c,r.real());return p;}
math_Vector ga_ipl_vec3(GaReader& r){math_Vector v(1,3);v(1)=r.real();v(2)=r.real();v(3)=r.real();return v;}
// `op q p rev ...`: one member of IntPatch_SpecialPoints on the quadric `q` and the other
// surface `p` (reversed: the quadric is S2): 0 AddCrossUVIsoPoint `ref tol`,
// 1 AddPointOnUorVIso `ref isu iso tol3 init3 inf3 sup3`, 2 AddSingularPole `iso vertex refcheck`
// (the vertex `pnt2s tol`), 3 ContinueAfterSpecialPoint `ref type tol2d new`,
// 4 AdjustPointAndVertex `ref periods4 new withvertex`.
void ga_ip_special_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int op=r.integer();occ::handle<Adaptor3d_Surface> q=ga_ics_surface(r),p=ga_ics_surface(r);bool rev=r.flag();
  typedef IntPatch_SpecialPoints S;IntSurf_PntOn2S out;
  if(op==0){IntSurf_PntOn2S ref=ga_ipl_pnt2s(r);double tol=r.real();GA_TRY(ga_flag(S::AddCrossUVIsoPoint(q,p,ref,tol,out,rev));ga_iw_pnt2s(out));}
  else if(op==1){IntSurf_PntOn2S ref=ga_ipl_pnt2s(r);bool isu=r.flag();double iso=r.real();math_Vector t=ga_ipl_vec3(r),i0=ga_ipl_vec3(r),lo=ga_ipl_vec3(r),hi=ga_ipl_vec3(r);
   GA_TRY(ga_flag(S::AddPointOnUorVIso(q,p,ref,isu,iso,t,i0,lo,hi,out,rev));ga_iw_pnt2s(out));}
  else if(op==2){IntSurf_PntOn2S iso=ga_ipl_pnt2s(r);IntSurf_PntOn2S vp=ga_ipl_pnt2s(r);double vt=r.real();bool chk=r.flag();
   IntPatch_Point v;v.SetValue(vp);v.SetTolerance(vt);GA_TRY(ga_flag(S::AddSingularPole(q,p,iso,v,out,rev,chk));ga_iw_pnt2s(out));ga_iw_pnt2s(v.PntOn2S());}
  else if(op==3){IntSurf_PntOn2S ref=ga_ipl_pnt2s(r);int t=r.integer();double tol2d=r.real();out=ga_ipl_pnt2s(r);
   GA_TRY(ga_flag(S::ContinueAfterSpecialPoint(q,p,ref,IntPatch_SpecPntType(t),tol2d,out,rev));ga_iw_pnt2s(out));}
  else{IntSurf_PntOn2S ref=ga_ipl_pnt2s(r);double per[4];for(double& x:per)x=r.real();out=ga_ipl_pnt2s(r);bool wv=r.flag();
   IntPatch_Point v;v.SetValue(out);S::AdjustPointAndVertex(ref,per,out,wv?&v:nullptr);ga_iw_pnt2s(out);ga_iw_pnt2s(v.PntOn2S());});
 ga_end();
}
// `kind pntiso du dv angle rev u`: kind 0 ProcessSphere with V `angle`, 1 ProcessCone of the
// cone of semi-angle `angle`, 2 GetTangentToIntLineForCone(angle, normal du).
void ga_ip_special_pole_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int kind=r.integer();IntSurf_PntOn2S iso=ga_ipl_pnt2s(r);gp_Vec du(r.xyz()),dv(r.xyz());double ang=r.real();bool rev=r.flag();double u=r.real();
  bool isoc=false;
  if(kind==0){GA_TRY(ga_flag(GaIplSpecialPoints::ProcessSphere(iso,du,dv,rev,ang,u,isoc));ga_real(u);ga_flag(isoc));}
  else if(kind==1){gp_Cone co(gp_Ax3(),ang,1.);GA_TRY(ga_flag(GaIplSpecialPoints::ProcessCone(iso,du,dv,co,rev,u,isoc));ga_real(u);ga_flag(isoc));}
  else{gp_XYZ res[2];int n=GaIplSpecialPoints::GetTangentToIntLineForCone(ang,du.XYZ(),res);ga_int(n);for(int i=0;i<n;++i)ga_xyz(res[i]);});
 ga_end();
}
