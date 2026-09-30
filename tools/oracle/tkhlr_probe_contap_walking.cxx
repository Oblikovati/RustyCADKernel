#include <Contap_TheIWalking.hxx>
IntSurf_PathPoint ct_walk_path_point(GaReader& r,Contap_SurfFunction& f){
 double u=r.real(),v=r.real();bool pass=r.flag(),tangent=r.flag();int nmult=r.integer();
 math_Vector x(1,2),fv(1,1);math_Matrix d(1,1,1,2);x(1)=u;x(2)=v;f.Values(x,fv,d);
 IntSurf_PathPoint p(f.Point(),u,v);
 if(!tangent)p.SetDirections(f.Direction3d(),f.Direction2d());
 p.SetPassing(pass);
 for(int i=0;i<nmult;i++){double a=r.real();p.AddUV(a,r.real());}
 return p;
}
// An interior point at (u, v): the function point and tangents (null ones when tangent).
IntSurf_InteriorPoint ct_walk_interior_point(GaReader& r,Contap_SurfFunction& f){
 double u=r.real(),v=r.real();math_Vector x(1,2),fv(1,1);math_Matrix d(1,1,1,2);x(1)=u;x(2)=v;f.Values(x,fv,d);
 if(f.IsTangent())return IntSurf_InteriorPoint(f.Point(),u,v,gp_Vec(0,0,0),gp_Vec2d(1,0));
 gp_Dir2d d2=f.Direction2d();return IntSurf_InteriorPoint(f.Point(),u,v,f.Direction3d(),gp_Vec2d(d2.X(),d2.Y()));
}
void ct_walk_report(const Contap_TheIWalking& w){
 ga_flag(w.IsDone());
 GA_TRY(int n=w.NbLines();ga_int(n);for(int i=1;i<=n;i++)ct_iwline_report(w.Value(i)));
 GA_TRY(int n=w.NbSinglePnts();ga_int(n);
  for(int i=1;i<=n;i++){const IntSurf_PathPoint& p=w.SinglePnt(i);ga_pnt(p.Value());double u,v;p.Value2d(u,v);ga_real(u);ga_real(v);});
 GA_TRY(w.Value(0));GA_TRY(w.Value(w.NbLines()+1));GA_TRY(w.SinglePnt(0));
}
void ct_walking_case(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);Contap_SurfFunction function;function.Set(surface);function.Set(r.dir());function.Set(1e-7);
 double epsilon=r.real(),deflection=r.real(),step=r.real();bool fill=r.flag(),reversed=r.flag();int mode=r.integer();
 NCollection_Sequence<IntSurf_PathPoint> paths;int np=r.integer();for(int i=0;i<np;++i)paths.Append(ct_walk_path_point(r,function));
 NCollection_Sequence<IntSurf_InteriorPoint> loops;int nl=r.integer();for(int i=0;i<nl;++i)loops.Append(ct_walk_interior_point(r,function));
 Contap_TheIWalking walk(epsilon,deflection,step,fill);walk.SetTolerance(epsilon,deflection,step);
 if(mode!=2){GA_TRY(walk.Perform(paths,loops,function,surface,reversed));}
 if(mode==1){GA_TRY(walk.Perform(paths,function,surface,reversed));}
 ct_walk_report(walk));ga_end();
}
