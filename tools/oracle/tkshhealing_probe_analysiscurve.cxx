// ShapeAnalysis_Curve projection overloads at pinned 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <ShapeAnalysis_Curve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <ShapeAnalysis.hxx>
struct ShProjectionRequest {
 int operation;occ::handle<Geom_Curve> curve;gp_Pnt point;double tolerance,first,last,previous;bool adjust;
};
ShProjectionRequest sh_read_projection(GaReader& r) {
 int op=r.integer();auto curve=tp_geom_curve(r);gp_Pnt point=r.pnt();
 double tolerance=r.real(),first=r.real(),last=r.real(),previous=r.real();
 return {op,curve,point,tolerance,first,last,previous,r.flag()};
}
double sh_run_adaptor_projection(const ShProjectionRequest& a,gp_Pnt& point,double& parameter) {
 GeomAdaptor_Curve curve(a.curve,a.first,a.last);ShapeAnalysis_Curve tool;
 switch(a.operation){case 2:return tool.Project(curve,a.point,a.tolerance,point,parameter,a.adjust);
 case 3:return tool.ProjectAct(curve,a.point,a.tolerance,point,parameter);
 default:return tool.NextProject(a.previous,curve,a.point,a.tolerance,point,parameter);}
}
double sh_run_projection(const ShProjectionRequest& a,gp_Pnt& point,double& parameter) {
 ShapeAnalysis_Curve tool;switch(a.operation){
 case 0:return tool.Project(a.curve,a.point,a.tolerance,point,parameter,a.adjust);
 case 1:return tool.Project(a.curve,a.point,a.tolerance,point,parameter,a.first,a.last,a.adjust);
 case 4:return tool.NextProject(a.previous,a.curve,a.point,a.tolerance,point,parameter,a.first,a.last,a.adjust);
 default:return sh_run_adaptor_projection(a,point,parameter);}
}
void sh_analysis_curve_projection_case(std::istream& in) {
 GaReader r{in};gp_Pnt point(91,-37,18);double parameter=123;
 GA_TRY(auto request=sh_read_projection(r);ga_real(sh_run_projection(request,point,parameter)));
 ga_pnt(point);ga_real(parameter);ga_end();
}
void sh_analysis_period_case(std::istream& in) {
 GaReader r{in};double value=r.real(),first=r.real(),last=r.real();
 ga_real(ShapeAnalysis::AdjustByPeriod(value,first,last));ga_real(ShapeAnalysis::AdjustToPeriod(value,first,last));ga_end();
}
