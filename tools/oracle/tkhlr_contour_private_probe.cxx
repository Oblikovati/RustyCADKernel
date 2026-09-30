// Included after the unchanged Contap_Contour.cxx so private functions remain accessible.
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include "tkgeomalgo_probe_common.cxx"
#include "tkhlr_probe_contana.cxx"
#include "tkhlr_probe_contap_surface.cxx"
void ct_private_match(Contap_Line& line,const occ::handle<Adaptor3d_Surface>& surface,const gp_Pnt2d& uv){
 gp_Pnt point;gp_Vec tangent(1,2,3),normal;double parameter=17.;
 GA_TRY(ga_flag(FindLine(line,surface,uv,point,parameter,tangent,normal)));
 ga_pnt(point);ga_real(parameter);ga_xyz(tangent.XYZ());ga_xyz(normal.XYZ());
}
void ct_private_math(std::istream& input){
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);double u=r.real(),v=r.real();gp_Pnt2d uv(u,v);gp_Vec tangent(r.xyz());
 GA_TRY(Recadre(surface,u,v));ga_real(u);ga_real(v);
 Contap_SurfFunction function;function.Set(surface);function.Set(r.dir());
 GA_TRY(ga_int(ComputeTransitionOnLine(function,u,v,tangent)));ga_pnt(function.Point());ga_real(function.Root());
 gp_Lin geometry(gp_Pnt(.2,-.3,.4),gp_Dir(0,0,1));GA_TRY(ga_int(ComputeTransitionOngpLine(function,geometry)));ga_pnt(function.Point());
 gp_Circ circle(gp_Ax2(),r.real());GA_TRY(ga_int(ComputeTransitionOngpCircle(function,circle)));ga_pnt(function.Point());
 Contap_Line line;line.SetValue(geometry);ct_private_match(line,surface,uv);line.SetValue(circle);ct_private_match(line,surface,uv));ga_end();
}
#include "tkhlr_contour_private_sections.cxx"
int main(){
 std::string line;while(std::getline(std::cin,line)){std::istringstream input(line);std::string operation;input>>operation;
 if(operation.empty())continue;if(operation=="contour_sections"){ct_private_sections(input);continue;}if(operation!="contour_math"){std::cerr<<"Unknown probe operation: "<<operation<<std::endl;return 2;}ct_private_math(input);}
}
