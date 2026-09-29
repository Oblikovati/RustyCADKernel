// IntImp probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_imp.rs. The generic templates are driven
// through their TKGeomAlgo instances: IntWalk_TheFunctionOfTheInt2S and
// IntWalk_TheInt2S (IntImp_ZerParFunc, IntImp_Int2S), IntPatch_TheSurfFunction
// (IntImp_ZerImpFunc), IntCurveSurface_TheCSFunctionOfHInter and
// IntCurveSurface_TheExactHInter (IntImp_ZerCSParFunc, IntImp_IntCS).
#include <IntImp_ComputeTangence.hxx>
#include <IntWalk_TheFunctionOfTheInt2S.hxx>
#include <IntWalk_TheInt2S.hxx>
#include <IntPatch_TheSurfFunction.hxx>
#include <IntCurveSurface_TheCSFunctionOfHInter.hxx>
#include <IntCurveSurface_TheExactHInter.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom_Circle.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <math_FunctionSetRoot.hxx>
#include <math_Matrix.hxx>
// `kind loc dir r1 r2 u1 u2 v1 v2`: 0 plane, 1 cylinder, 2 sphere, 3 torus, adapted on the bounds.
occ::handle<Adaptor3d_Surface> ga_ii_surface(GaReader& r){
 int kind=r.integer();gp_Pnt p=r.pnt();gp_Dir d=r.dir();double r1=r.real(),r2=r.real();
 double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();gp_Ax3 ax(p,d);occ::handle<Geom_Surface> s;
 if(kind==0)s=new Geom_Plane(ax);else if(kind==1)s=new Geom_CylindricalSurface(ax,r1);
 else if(kind==2)s=new Geom_SphericalSurface(ax,r1);else s=new Geom_ToroidalSurface(ax,r1,r2);
 return new GeomAdaptor_Surface(s,u1,u2,v1,v2);
}
// `kind loc dir r first last`: 0 line, 1 circle, adapted on the range.
occ::handle<Adaptor3d_Curve> ga_ii_curve(GaReader& r){
 int kind=r.integer();gp_Pnt p=r.pnt();gp_Dir d=r.dir();double rad=r.real(),f=r.real(),l=r.real();occ::handle<Geom_Curve> c;
 if(kind==0)c=new Geom_Line(p,d);else c=new Geom_Circle(gp_Ax2(p,d),rad);
 return new GeomAdaptor_Curve(c,f,l);
}
void ga_ii_vector(const math_Vector& v){for(int i=v.Lower();i<=v.Upper();i++)ga_real(v(i));}
void ga_ii_matrix(const math_Matrix& m){for(int i=m.LowerRow();i<=m.UpperRow();i++)for(int j=m.LowerCol();j<=m.UpperCol();j++)ga_real(m(i,j));}
// `index`: ChoixRef.
void ga_ii_choix_case(std::istream& input){GaReader r{input};int i=r.integer();GA_TRY(ga_int(int(ChoixRef(i))));ga_end();}
// `dpuv(4 xyz) eps(4)`: IntImp_ComputeTangence with zeroed outputs.
void ga_ii_tangence_case(std::istream& input){
 GaReader r{input};gp_Vec dpuv[4];for(gp_Vec& v:dpuv)v=gp_Vec(r.xyz());double eps[4];for(double& e:eps)e=r.real();
 double tg[4]={0,0,0,0};IntImp_ConstIsoparametric iso[4]={IntImp_UIsoparametricOnCaro1,IntImp_UIsoparametricOnCaro1,IntImp_UIsoparametricOnCaro1,IntImp_UIsoparametricOnCaro1};
 ga_flag(IntImp_ComputeTangence(dpuv,eps,tg,iso));for(double t:tg)ga_real(t);for(auto i:iso)ga_int(int(i));
 ga_end();
}
// `s1 s2 iso u1 v1 u2 v2`: IntWalk_TheFunctionOfTheInt2S through ComputeParameters,
// Value, Values, Root, Point, IsTangent and the directions.
void ga_ii_zerpar_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s1=ga_ii_surface(r),s2=ga_ii_surface(r);int iso=r.integer();
 NCollection_Array1<double> param(1,4);for(int i=1;i<=4;i++)param(i)=r.real();
 GA_TRY(IntWalk_TheFunctionOfTheInt2S f(s1,s2);math_Vector uv(1,3,0.0),inf(1,3,0.0),sup(1,3,0.0),tol(1,3,0.0);
  f.ComputeParameters(IntImp_ConstIsoparametric(iso),param,uv,inf,sup,tol);ga_ii_vector(uv);ga_ii_vector(inf);ga_ii_vector(sup);ga_ii_vector(tol);
  math_Vector fv(1,3,0.0);math_Matrix d(1,3,1,3,0.0);ga_flag(f.Value(uv,fv));ga_ii_vector(fv);ga_real(f.Root());
  math_Vector x2(1,3);for(int i=1;i<=3;i++)x2(i)=uv(i)+0.01*i;ga_flag(f.Values(x2,fv,d));ga_ii_vector(fv);ga_ii_matrix(d);
  math_Matrix d2(1,3,1,3,0.0);ga_flag(f.Derivatives(uv,d2));ga_ii_matrix(d2);ga_real(f.Root());ga_pnt(f.Point());
  NCollection_Array1<double> out(1,4);out.Init(0.0);IntImp_ConstIsoparametric best=IntImp_UIsoparametricOnCaro1;
  ga_flag(f.IsTangent(uv,out,best));for(int i=1;i<=4;i++)ga_real(out(i));ga_int(int(best));
  GA_TRY(ga_dir(f.Direction()));GA_TRY(gp_Dir2d a=f.DirectionOnS1();ga_real(a.X());ga_real(a.Y()));
  GA_TRY(gp_Dir2d b=f.DirectionOnS2();ga_real(b.X());ga_real(b.Y())));
 ga_end();
}
void ga_ii_int2s_report(const IntWalk_TheInt2S& s){
 ga_flag(s.IsDone());GA_TRY(ga_flag(s.IsEmpty()));
 GA_TRY(const IntSurf_PntOn2S& p=s.Point();ga_pnt(p.Value());double a,b,c,d;p.Parameters(a,b,c,d);ga_real(a);ga_real(b);ga_real(c);ga_real(d));
 GA_TRY(ga_flag(s.IsTangent()));GA_TRY(ga_dir(s.Direction()));
 GA_TRY(gp_Dir2d a=s.DirectionOnS1();ga_real(a.X());ga_real(a.Y()));GA_TRY(gp_Dir2d b=s.DirectionOnS2();ga_real(b.X());ga_real(b.Y()));
}
// `s1 s2 tol mode iso u1 v1 u2 v2`: IntWalk_TheInt2S at a start point (mode 0), or
// Perform on the isoparametric `iso` (mode 1).
void ga_ii_int2s_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s1=ga_ii_surface(r),s2=ga_ii_surface(r);double tol=r.real();
 int mode=r.integer(),iso=r.integer();NCollection_Array1<double> param(1,4);for(int i=1;i<=4;i++)param(i)=r.real();
 GA_TRY(if(mode==0){IntWalk_TheInt2S s(param,s1,s2,tol);ga_ii_int2s_report(s);}
  else{IntWalk_TheInt2S s(s1,s2,tol);math_FunctionSetRoot rs(s.Function(),15);
   ga_int(int(s.Perform(param,rs,IntImp_ConstIsoparametric(iso))));ga_ii_int2s_report(s);});
 ga_end();
}
// `surface quadric tol u v`: IntPatch_TheSurfFunction evaluated and tested for tangency.
void ga_ii_zerimp_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s=ga_ii_surface(r);IntSurf_Quadric q=ga_intsurf_quadric(r);double tol=r.real();
 math_Vector x(1,2);x(1)=r.real();x(2)=r.real();
 GA_TRY(IntPatch_TheSurfFunction f(s,q);f.Set(tol);ga_real(f.Tolerance());math_Vector fv(1,1,0.0);math_Matrix d(1,1,1,2,0.0);
  ga_flag(f.Value(x,fv));ga_real(fv(1));ga_real(f.Root());ga_pnt(f.Point());
  GA_TRY(ga_flag(f.IsTangent()));
  ga_flag(f.Derivatives(x,d));ga_ii_matrix(d);GA_TRY(ga_flag(f.IsTangent()));
  math_Vector x2(1,2);x2(1)=x(1)+0.02;x2(2)=x(2)-0.01;ga_flag(f.Values(x2,fv,d));ga_real(fv(1));ga_ii_matrix(d);
  GA_TRY(ga_flag(f.IsTangent()));GA_TRY(gp_Vec v=f.Direction3d();ga_xyz(v.XYZ()));
  GA_TRY(gp_Dir2d v=f.Direction2d();ga_real(v.X());ga_real(v.Y()));ga_pnt(f.Point()));
 ga_end();
}
// `surface curve u v w tol margin`: IntCurveSurface_TheExactHInter from a start point.
void ga_ii_intcs_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s=ga_ii_surface(r);occ::handle<Adaptor3d_Curve> c=ga_ii_curve(r);
 double u=r.real(),v=r.real(),w=r.real(),tol=r.real(),margin=r.real();
 GA_TRY(IntCurveSurface_TheCSFunctionOfHInter f(s,c);IntCurveSurface_TheExactHInter e(u,v,w,f,tol,margin);
  ga_flag(e.IsDone());GA_TRY(ga_flag(e.IsEmpty()));GA_TRY(ga_pnt(e.Point()));
  GA_TRY(double a,b;e.ParameterOnSurface(a,b);ga_real(a);ga_real(b));GA_TRY(ga_real(e.ParameterOnCurve()));
  ga_real(e.Function().Root()));
 ga_end();
}
