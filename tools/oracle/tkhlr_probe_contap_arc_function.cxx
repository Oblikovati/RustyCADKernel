// All original arc function members, saved points and caller-owned outputs.
#include <Contap_ArcFunction.hxx>
void ct_arc_function_state(Contap_ArcFunction& f){
 ga_pnt(f.LastComputedPoint());ga_flag(f.Surface().IsNull());
 GA_TRY(ga_int(f.NbSamples()));int index=f.GetStateNumber();ga_int(index);
 for(int i:{0,1,index,index+1})GA_TRY(ga_pnt(f.Valpoint(i)));
 ga_flag(&f.Quadric()==&f.Quadric());
}
void ct_arc_function_evaluate(Contap_ArcFunction& f,double u){
 double value=17.,derivative=19.;GA_TRY(ga_flag(f.Value(u,value)));ga_real(value);ct_arc_function_state(f);
 GA_TRY(ga_flag(f.Derivative(u+.125,derivative)));ga_real(derivative);ct_arc_function_state(f);
 GA_TRY(ga_flag(f.Values(u+.25,value,derivative)));ga_real(value);ga_real(derivative);ct_arc_function_state(f);
}
void ct_arc_function_case(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);auto arc=ct_arc(r);double u=r.real(),angle=r.real();
 gp_Dir direction=r.dir();gp_Pnt eye=r.pnt();Contap_ArcFunction f;
 ga_flag(f.Surface().IsNull());ga_pnt(f.LastComputedPoint());ga_int(f.GetStateNumber());ga_pnt(f.Valpoint(1));
 GA_TRY(f.Set(surface));f.Set(arc);ct_arc_function_state(f);
 f.Set(direction);ct_arc_function_evaluate(f,u);f.Set(eye,angle);ct_arc_function_evaluate(f,u);
 f.Set(eye);ct_arc_function_evaluate(f,u);f.Set(direction,angle);ct_arc_function_evaluate(f,u);
 f.Set(arc);ct_arc_function_state(f);GA_TRY(f.Set(surface));ct_arc_function_state(f));ga_end();
}
