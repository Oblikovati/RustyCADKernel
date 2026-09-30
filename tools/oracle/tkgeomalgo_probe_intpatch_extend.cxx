// IntPatch_WLineTool::ExtendTwoWLines probe operation (tkgeomalgo_probe.cxx); the Rust twin
// is crates/tkgeomalgo/examples/ga_probe/int_patch_extend.rs. Two quadrics, walking lines
// given by their 3d points (the parameters on both quadrics are computed with ElSLib) with
// vertices at their ends, extended as IntPatch_Intersection extends the lines of two
// quadrics. Needs tkgeomalgo_probe_intpatch_wltool.cxx.
#include <Bnd_Box2d.hxx>
#include <ElSLib.hxx>
// The parameters of `p` on a quadric adaptor (0 for another type).
void ga_ex_params(const occ::handle<Adaptor3d_Surface>& s,const gp_Pnt& p,double& u,double& v){
 u=0.;v=0.;
 switch(s->GetType()){
  case GeomAbs_Plane:ElSLib::Parameters(s->Plane(),p,u,v);break;
  case GeomAbs_Cylinder:ElSLib::Parameters(s->Cylinder(),p,u,v);break;
  case GeomAbs_Sphere:ElSLib::Parameters(s->Sphere(),p,u,v);break;
  case GeomAbs_Cone:ElSLib::Parameters(s->Cone(),p,u,v);break;
  case GeomAbs_Torus:ElSLib::Parameters(s->Torus(),p,u,v);break;
  default:break;}}
// A walking line `npoints (x y z)* shift1 shift2`: the parameters of each point on both
// quadrics (the U parameters shifted by the given values), vertices at the first and
// last points.
occ::handle<IntPatch_Line> ga_ex_line(GaReader& r,const occ::handle<Adaptor3d_Surface>& s1,const occ::handle<Adaptor3d_Surface>& s2){
 int n=r.integer();occ::handle<IntSurf_LineOn2S> c=new IntSurf_LineOn2S();std::vector<gp_Pnt> pts;
 for(int i=0;i<n;++i)pts.push_back(r.pnt());double sh1=r.real(),sh2=r.real();
 for(const gp_Pnt& p:pts){double u1,v1,u2,v2;ga_ex_params(s1,p,u1,v1);ga_ex_params(s2,p,u2,v2);
  IntSurf_PntOn2S q;q.SetValue(p,u1+sh1,v1,u2+sh2,v2);c->Add(q);}
 occ::handle<IntPatch_WLine> l=new IntPatch_WLine(c,false,IntSurf_In,IntSurf_Out);l->SetCreatingWayInfo(IntPatch_WLine::IntPatch_WLImpImp);
 for(int k=0;k<2&&n>0;++k){int i=k==0?1:n;IntPatch_Point v;v.SetValue(c->Value(i).Value(),1e-7,false);double a,b,e,d;c->Value(i).Parameters(a,b,e,d);
  v.SetParameters(a,b,e,d);v.SetParameter(i);l->AddVertex(v);}
 return l;}
// `s1 s2 tol ncrit (x y z)* nlines lines`: the lines extended as IntPatch_Intersection
// does after an implicit-implicit intersection (the boxes of the surface bounds enlarged
// by PConfusion, the periods of the surfaces, the apexes and poles as critical points,
// then the given ones).
void ga_ip_extend_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);double tol=r.real();
  NCollection_List<gp_Pnt> crit;
  if(s1->GetType()==GeomAbs_Cone)crit.Append(s1->Cone().Apex());
  else if(s1->GetType()==GeomAbs_Sphere){crit.Append(s1->Value(0.0,M_PI_2));crit.Append(s1->Value(0.0,-M_PI_2));}
  if(s2->GetType()==GeomAbs_Cone)crit.Append(s2->Cone().Apex());
  else if(s2->GetType()==GeomAbs_Sphere){crit.Append(s2->Value(0.0,M_PI_2));crit.Append(s2->Value(0.0,-M_PI_2));}
  int nc=r.integer();for(int i=0;i<nc;++i)crit.Append(r.pnt());
  NCollection_Sequence<occ::handle<IntPatch_Line>> lines;int nl=r.integer();for(int i=0;i<nl;++i)lines.Append(ga_ex_line(r,s1,s2));
  Bnd_Box2d b1,b2;
  const double u1f=s1->FirstUParameter(),u1l=s1->LastUParameter(),v1f=s1->FirstVParameter(),v1l=s1->LastVParameter();
  const double u2f=s2->FirstUParameter(),u2l=s2->LastUParameter(),v2f=s2->FirstVParameter(),v2l=s2->LastVParameter();
  b1.Add(gp_Pnt2d(u1f,v1f));b1.Add(gp_Pnt2d(u1l,v1f));b1.Add(gp_Pnt2d(u1l,v1l));b1.Add(gp_Pnt2d(u1f,v1l));
  b2.Add(gp_Pnt2d(u2f,v2f));b2.Add(gp_Pnt2d(u2l,v2f));b2.Add(gp_Pnt2d(u2l,v2l));b2.Add(gp_Pnt2d(u2f,v2l));
  b1.Enlarge(Precision::PConfusion());b2.Enlarge(Precision::PConfusion());
  const double per[4]={s1->IsUPeriodic()?s1->UPeriod():0.0,s1->IsVPeriodic()?s1->VPeriod():0.0,
                       s2->IsUPeriodic()?s2->UPeriod():0.0,s2->IsVPeriodic()?s2->VPeriod():0.0};
  GA_TRY(IntPatch_WLineTool::ExtendTwoWLines(lines,s1,s2,tol,per,b1,b2,crit));
  ga_int(lines.Length());for(int i=1;i<=lines.Length();++i)ga_wt_report(fx,lines(i)));
 ga_end();
}
