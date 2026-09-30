// IntWalk_IWalking.gxx probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_walk_iw.rs. The template runs through its
// IntPatch instance, IntPatch_TheIWalking (Adaptor3d_HSurfaceTool,
// IntPatch_TheSurfFunction and IntPatch_TheIWLineOfTheIWalking), as the Rust twin runs
// the IntPatch alias of the generic.
#include <Adaptor3d_HSurfaceTool.hxx>
#include <Bnd_Range.hxx>
#include <IntPatch_TheIWLineOfTheIWalking.hxx>
#include <IntPatch_TheIWalking.hxx>
#include <IntPatch_TheSurfFunction.hxx>
#include <IntSurf_InteriorPoint.hxx>
#include <IntSurf_InteriorPointTool.hxx>
#include <IntSurf_PathPoint.hxx>
#include <IntSurf_PathPointTool.hxx>
#include <IntWalk_StatusDeflection.hxx>
#include <IntWalk_VectorOfInteger.hxx>
#include <IntWalk_VectorOfWalkingData.hxx>
#include <NCollection_DataMap.hxx>
#include <NCollection_List.hxx>
#include <NCollection_Sequence.hxx>
#include <StdFail_NotDone.hxx>
#include <math_FunctionSetRoot.hxx>
#include <math_Vector.hxx>
// A path point at (u, v) of the function: its point, its tangents unless `tangent`,
// the passing flag and `nmult` auxiliary parameters.
IntSurf_PathPoint ga_iw_path_point(GaReader& r,IntPatch_TheSurfFunction& f){
 double u=r.real(),v=r.real();bool pass=r.flag(),tangent=r.flag();int nmult=r.integer();
 math_Vector x(1,2),fv(1,1);math_Matrix d(1,1,1,2);x(1)=u;x(2)=v;f.Values(x,fv,d);
 IntSurf_PathPoint p(f.Point(),u,v);
 if(!tangent)p.SetDirections(f.Direction3d(),f.Direction2d());
 p.SetPassing(pass);
 for(int i=0;i<nmult;i++){double a=r.real();p.AddUV(a,r.real());}
 return p;
}
// An interior point at (u, v): the function point and tangents (null ones when tangent).
IntSurf_InteriorPoint ga_iw_interior_point(GaReader& r,IntPatch_TheSurfFunction& f){
 double u=r.real(),v=r.real();math_Vector x(1,2),fv(1,1);math_Matrix d(1,1,1,2);x(1)=u;x(2)=v;f.Values(x,fv,d);
 if(f.IsTangent())return IntSurf_InteriorPoint(f.Point(),u,v,gp_Vec(0,0,0),gp_Vec2d(1,0));
 gp_Dir2d d2=f.Direction2d();return IntSurf_InteriorPoint(f.Point(),u,v,f.Direction3d(),gp_Vec2d(d2.X(),d2.Y()));
}
void ga_iw_line_report(const occ::handle<IntPatch_TheIWLineOfTheIWalking>& l){
 int n=l->NbPoints();ga_int(n);for(int i=1;i<=n;i++)ga_iw_pnt2s(l->Value(i));
 ga_flag(l->IsClosed());ga_flag(l->HasFirstPoint());ga_flag(l->HasLastPoint());
 GA_TRY(ga_int(l->FirstPointIndex()));GA_TRY(ga_pnt(l->FirstPoint().Value()));
 GA_TRY(ga_int(l->LastPointIndex()));GA_TRY(ga_pnt(l->LastPoint().Value()));
 ga_int(l->NbPassingPoint());for(int i=1;i<=l->NbPassingPoint();i++){int a,b;l->PassingPoint(i,a,b);ga_int(a);ga_int(b);}
 int idx=0;const gp_Vec& v=l->TangentVector(idx);ga_xyz(v.XYZ());ga_int(idx);
 ga_flag(l->IsTangentAtBegining());ga_flag(l->IsTangentAtEnd());
}
void ga_iw_iw_report(const IntPatch_TheIWalking& w){
 ga_flag(w.IsDone());
 GA_TRY(int n=w.NbLines();ga_int(n);for(int i=1;i<=n;i++)ga_iw_line_report(w.Value(i)));
 GA_TRY(int n=w.NbSinglePnts();ga_int(n);
  for(int i=1;i<=n;i++){const IntSurf_PathPoint& p=w.SinglePnt(i);ga_pnt(p.Value());double u,v;p.Value2d(u,v);ga_real(u);ga_real(v);});
 GA_TRY(w.Value(0));GA_TRY(w.Value(w.NbLines()+1));GA_TRY(w.SinglePnt(0));
}
// `surface quadric tol eps defl step fill reversed mode npath [u v pass tangent nmult
// (u v)*]* nloop [u v]*`: the walking of the quadric on the surface; mode 0 the full
// Perform, 1 the full Perform then the open-line Perform, 2 the consultation of a
// walking never performed.
void ga_iw_iw_walk_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s=ga_iw_surface(r);IntSurf_Quadric q=ga_intsurf_quadric(r);
 double tol=r.real(),eps=r.real(),defl=r.real(),step=r.real();bool fill=r.flag(),rev=r.flag();int mode=r.integer();
 GA_TRY(IntPatch_TheSurfFunction f(s,q);f.Set(tol);
  NCollection_Sequence<IntSurf_PathPoint> paths;NCollection_Sequence<IntSurf_InteriorPoint> loops;
  int np=r.integer();for(int i=0;i<np;i++)paths.Append(ga_iw_path_point(r,f));
  int nl=r.integer();for(int i=0;i<nl;i++)loops.Append(ga_iw_interior_point(r,f));
  IntPatch_TheIWalking w(eps,defl,step,fill);
  if(mode!=2){GA_TRY(w.Perform(paths,loops,f,s,rev));}
  if(mode==1){GA_TRY(w.Perform(paths,f,s,rev));}
  ga_iw_iw_report(w));
 ga_end();
}
