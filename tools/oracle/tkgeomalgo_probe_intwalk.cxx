// IntWalk_PWalking probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_walk.rs. The protected members are reached
// through a probe subclass.
#include <IntSurf_LineOn2S.hxx>
#include <IntWalk_PWalking.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <NCollection_Array2.hxx>
// `kind ...`: 0..4 `loc dir r1 r2` a plane, cylinder (r1), sphere (r1), torus (r1, r2)
// or cone (r1 the reference radius, r2 the semi-angle); 5 `nu nv poles` a Bezier
// surface; 6 `du dv nu nv poles` a clamped uniform B-spline surface; then the adaptor
// bounds `u1 u2 v1 v2`.
occ::handle<Adaptor3d_Surface> ga_iw_surface(GaReader& r){
 int kind=r.integer();occ::handle<Geom_Surface> s;
 if(kind<=4){
  gp_Pnt p=r.pnt();gp_Dir d=r.dir();double r1=r.real(),r2=r.real();gp_Ax3 ax(p,d);
  if(kind==0)s=new Geom_Plane(ax);else if(kind==1)s=new Geom_CylindricalSurface(ax,r1);
  else if(kind==2)s=new Geom_SphericalSurface(ax,r1);else if(kind==3)s=new Geom_ToroidalSurface(ax,r1,r2);
  else s=new Geom_ConicalSurface(ax,r2,r1);
 } else {
  int du=0,dv=0;if(kind==6){du=r.integer();dv=r.integer();}
  int nu=r.integer(),nv=r.integer();NCollection_Array2<gp_Pnt> poles(1,nu,1,nv);
  for(int i=1;i<=nu;i++)for(int j=1;j<=nv;j++)poles(i,j)=r.pnt();
  if(kind==5)s=new Geom_BezierSurface(poles);
  else{
   NCollection_Array1<double> uk(1,nu-du+1),vk(1,nv-dv+1);NCollection_Array1<int> um(1,nu-du+1),vm(1,nv-dv+1);
   for(int i=1;i<=nu-du+1;i++){uk(i)=i-1;um(i)=(i==1||i==nu-du+1)?du+1:1;}
   for(int i=1;i<=nv-dv+1;i++){vk(i)=i-1;vm(i)=(i==1||i==nv-dv+1)?dv+1:1;}
   s=new Geom_BSplineSurface(poles,uk,vk,um,vm,du,dv);
  }
 }
 double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
 return new GeomAdaptor_Surface(s,u1,u2,v1,v2);
}
struct GaProbePWalking : public IntWalk_PWalking {
 using IntWalk_PWalking::IntWalk_PWalking;
 using IntWalk_PWalking::ComputePasInit;
 using IntWalk_PWalking::DistanceMinimizeByGradient;
 using IntWalk_PWalking::DistanceMinimizeByExtrema;
 using IntWalk_PWalking::SeekPointOnBoundary;
 using IntWalk_PWalking::HandleSingleSingularPoint;
 using IntWalk_PWalking::ExtendLineInCommonZone;
};
void ga_iw_pnt2s(const IntSurf_PntOn2S& p){ga_pnt(p.Value());double a,b,c,d;p.Parameters(a,b,c,d);ga_real(a);ga_real(b);ga_real(c);ga_real(d);}
// The whole state of a walking: done flag, points, tangencies, closure, boundary
// tangent, steps, and the accessors past the line.
void ga_iw_pw_report(const IntWalk_PWalking& w){
 ga_flag(w.IsDone());
 GA_TRY(int n=w.NbPoints();ga_int(n);for(int i=1;i<=n;i++)ga_iw_pnt2s(w.Value(i)));
 GA_TRY(ga_flag(w.TangentAtFirst()));GA_TRY(ga_flag(w.TangentAtLast()));GA_TRY(ga_flag(w.IsClosed()));
 GA_TRY(int idx=0;const gp_Dir& d=w.TangentAtLine(idx);ga_dir(d);ga_int(idx));
 for(int i=-1;i<=4;i++){GA_TRY(ga_real(const_cast<IntWalk_PWalking&>(w).MaxStep(i)));}
 GA_TRY(ga_iw_pnt2s(w.Value(0)));GA_TRY(ga_iw_pnt2s(w.Value(w.NbPoints()+1)));
 GA_TRY(ga_int(w.Line()->NbPoints()));
}
void ga_iw_params(GaReader& r,NCollection_Array1<double>& p){for(int i=1;i<=4;i++)p(i)=r.real();}
// `s1 s2 toltang eps defl incr mode start(4) [box(8) when mode 1] post minpts`: mode 0
// Perform(start), 1 Perform(start, box), 2 the performing constructor; post bit 1
// PutToBoundary, bit 2 SeekAdditionalPoints(minpts) (on a done line).
void ga_iw_pw_walk_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s1=ga_iw_surface(r),s2=ga_iw_surface(r);
 double tt=r.real(),eps=r.real(),defl=r.real(),incr=r.real();int mode=r.integer();
 NCollection_Array1<double> start(1,4);ga_iw_params(r,start);double box[8]={0,0,0,0,0,0,0,0};
 if(mode==1)for(double& b:box)b=r.real();int post=r.integer(),minpts=r.integer();
 GA_TRY(std::unique_ptr<IntWalk_PWalking> w;
  if(mode==2)w.reset(new IntWalk_PWalking(s1,s2,tt,eps,defl,incr,start(1),start(2),start(3),start(4)));
  else{w.reset(new IntWalk_PWalking(s1,s2,tt,eps,defl,incr));
   if(mode==1)w->Perform(start,box[0],box[1],box[2],box[3],box[4],box[5],box[6],box[7]);else w->Perform(start);}
  if(w->IsDone()&&(post&1)){GA_TRY(ga_flag(w->PutToBoundary(s1,s2)));}
  if(w->IsDone()&&(post&2)){GA_TRY(ga_flag(w->SeekAdditionalPoints(s1,s2,minpts)));}
  ga_iw_pw_report(*w));
 ga_end();
}
// `s1 s2 toltang eps defl incr start(4)`: PerformFirstPoint.
void ga_iw_pw_first_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s1=ga_iw_surface(r),s2=ga_iw_surface(r);
 double tt=r.real(),eps=r.real(),defl=r.real(),incr=r.real();NCollection_Array1<double> start(1,4);ga_iw_params(r,start);
 GA_TRY(IntWalk_PWalking w(s1,s2,tt,eps,defl,incr);IntSurf_PntOn2S p;ga_flag(w.PerformFirstPoint(start,p));ga_iw_pnt2s(p);
  for(int i=0;i<=3;i++)ga_real(w.MaxStep(i)));
 ga_end();
}
// `s1 s2 toltang eps defl incr start(4) iso remove`: after Perform, the public stepping
// members: TestDeflection, TestArret (both flags), RepartirOuDiviser (both arrivals),
// RemoveAPoint(remove) and AddAPoint, then the report.
void ga_iw_pw_edit_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s1=ga_iw_surface(r),s2=ga_iw_surface(r);
 double tt=r.real(),eps=r.real(),defl=r.real(),incr=r.real();NCollection_Array1<double> start(1,4);ga_iw_params(r,start);
 int iso=r.integer(),remove=r.integer();
 GA_TRY(IntWalk_PWalking w(s1,s2,tt,eps,defl,incr);w.Perform(start);ga_flag(w.IsDone());
  IntImp_ConstIsoparametric ci=IntImp_ConstIsoparametric(iso);
  GA_TRY(ga_int(int(w.TestDeflection(ci,IntWalk_OK))));GA_TRY(ga_int(int(w.TestDeflection(ci,IntWalk_PasTropGrand))));
  for(int flagv=0;flagv<=1;flagv++){NCollection_Array1<double> p(1,4);p=start;IntImp_ConstIsoparametric c=ci;
   GA_TRY(ga_flag(w.TestArret(flagv==1,p,c));for(int i=1;i<=4;i++)ga_real(p(i));ga_int(int(c)));}
  for(int arr=0;arr<=1;arr++){bool dr=false,a=arr==1;IntImp_ConstIsoparametric c=ci;
   GA_TRY(w.RepartirOuDiviser(dr,c,a);ga_flag(dr);ga_flag(a);ga_int(int(c)));}
  GA_TRY(w.RemoveAPoint(remove));GA_TRY(if(w.IsDone()&&w.NbPoints()>0)w.AddAPoint(w.Value(1)));
  ga_iw_pw_report(w));
 ga_end();
}
// `op s1 s2 toltang eps defl incr pnt(4) ...`: the protected searches. op 0
// DistanceMinimizeByGradient (`hasstep step(4)`), 1 DistanceMinimizeByExtrema of s2's
// point at (pnt3, pnt4) on s1 from (pnt1, pnt2) (`hasstep step(2)`), 2
// HandleSingleSingularPoint (`tol`), 3 ComputePasInit (`deltas(4)`), 4 Perform(start =
// pnt) then SeekPointOnBoundary (`uv(4) first`), 5 Perform then ExtendLineInCommonZone
// (`iso flag`).
void ga_iw_pw_min_case(std::istream& input){
 GaReader r{input};int op=r.integer();occ::handle<Adaptor3d_Surface> s1=ga_iw_surface(r),s2=ga_iw_surface(r);
 double tt=r.real(),eps=r.real(),defl=r.real(),incr=r.real();NCollection_Array1<double> p(1,4);ga_iw_params(r,p);
 GA_TRY(GaProbePWalking w(s1,s2,tt,eps,defl,incr);
  if(op==0){int has=r.integer();double st[4];for(double& x:st)x=r.real();
   ga_flag(w.DistanceMinimizeByGradient(s1,s2,p,has?st:nullptr));for(int i=1;i<=4;i++)ga_real(p(i));}
  else if(op==1){int has=r.integer();double st[2];for(double& x:st)x=r.real();double u=p(1),v=p(2);
   ga_flag(w.DistanceMinimizeByExtrema(s1,s2->Value(p(3),p(4)),u,v,has?st:nullptr));ga_real(u);ga_real(v);}
  else if(op==2){double tol=r.real();ga_flag(w.HandleSingleSingularPoint(s1,s2,tol,p));for(int i=1;i<=4;i++)ga_real(p(i));}
  else if(op==3){double d[4];for(double& x:d)x=r.real();w.ComputePasInit(d[0],d[1],d[2],d[3]);for(int i=0;i<=3;i++)ga_real(w.MaxStep(i));}
  else if(op==4){double uv[4];for(double& x:uv)x=r.real();int first=r.integer();w.Perform(p);
   GA_TRY(ga_flag(w.SeekPointOnBoundary(s1,s2,uv[0],uv[1],uv[2],uv[3],first!=0)));ga_iw_pw_report(w);}
  else{int iso=r.integer(),fl=r.integer();w.Perform(p);
   GA_TRY(ga_flag(w.ExtendLineInCommonZone(IntImp_ConstIsoparametric(iso),fl!=0)));ga_iw_pw_report(w);});
 ga_end();
}
