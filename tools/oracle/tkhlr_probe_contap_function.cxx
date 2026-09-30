// Contour function state, retained DraftPrs output and cached tangent behavior.
#include <Contap_SurfFunction.hxx>
#include <math_Matrix.hxx>
void ct_function_state(Contap_SurfFunction& f){
 ga_int(f.NbVariables());ga_int(f.NbEquations());ga_int(f.FunctionType());
 ga_real(f.Root());ga_real(f.Tolerance());ga_real(f.Angle());ga_pnt(f.Point());
 ga_pnt(f.Eye());ga_xyz(f.Direction().XYZ());
 ga_flag(f.Surface()==f.PSurface());ga_flag(f.Surface().IsNull());
}
void ct_function_tangent(Contap_SurfFunction& f){
 GA_TRY(ga_flag(f.IsTangent()));GA_TRY(ga_xyz(f.Direction3d().XYZ()));
 GA_TRY(auto d=f.Direction2d();ga_real(d.X());ga_real(d.Y()));
 GA_TRY(ga_flag(f.IsTangent()));
}
void ct_function_evaluate(Contap_SurfFunction& f,double u,double v){
 math_Vector x(1,2),value(1,1);math_Matrix grad(1,1,1,2);x(1)=u;x(2)=v;
 value(1)=17.;grad(1,1)=19.;grad(1,2)=23.;
 GA_TRY(ga_flag(f.Value(x,value)));ga_real(value(1));ct_function_state(f);ct_function_tangent(f);
 x(1)=u+.125;GA_TRY(ga_flag(f.Derivatives(x,grad)));ga_real(grad(1,1));ga_real(grad(1,2));ct_function_state(f);ct_function_tangent(f);
 x(2)=v+.25;GA_TRY(ga_flag(f.Values(x,value,grad)));ga_real(value(1));ga_real(grad(1,1));ga_real(grad(1,2));ct_function_state(f);ct_function_tangent(f);
}
void ct_function_case(std::istream& input){
 GaReader r{input};GA_TRY(auto s=hl_contap_surface(r);double u=r.real(),v=r.real(),tol=r.real(),angle=r.real();
 gp_Dir direction=r.dir();gp_Pnt eye=r.pnt();Contap_SurfFunction f;ct_function_state(f);
 GA_TRY(f.Set(s));f.Set(tol);ct_function_state(f);
 f.Set(direction);ct_function_evaluate(f,u,v);
 f.Set(eye,angle);ct_function_tangent(f);ct_function_evaluate(f,u,v);
 f.Set(eye);ct_function_evaluate(f,u,v);
 f.Set(direction,angle);ct_function_evaluate(f,u,v);
 f.Set(direction);ct_function_tangent(f);GA_TRY(f.Set(s));ct_function_evaluate(f,u,v));ga_end();
}
