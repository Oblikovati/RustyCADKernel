// Included after immutable Contap_Contour.cxx to call its private ComputeTangency.
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include "tkgeomalgo_probe_common.cxx"
#include "tkhlr_probe_contana.cxx"
#include "tkhlr_probe_contap_surface.cxx"
class CtTangencyDomain : public Adaptor3d_TopolTool {
public:
 int Mode=0;
 CtTangencyDomain(const occ::handle<Adaptor3d_Surface>& surface):Adaptor3d_TopolTool(surface){}
 TopAbs_Orientation Orientation(const occ::handle<Adaptor2d_Curve2d>& arc) override {
  switch(Mode%5){case 1:return TopAbs_REVERSED;case 2:return TopAbs_INTERNAL;case 3:return TopAbs_EXTERNAL;
   case 4:return arc->FirstParameter()<0?TopAbs_REVERSED:TopAbs_FORWARD;default:return Adaptor3d_TopolTool::Orientation(arc);}
 }
 TopAbs_Orientation Orientation(const occ::handle<Adaptor3d_HVertex>& vertex) override {
  auto orientation=Adaptor3d_TopolTool::Orientation(vertex);if(Mode<5)return orientation;
  return orientation==TopAbs_FORWARD?TopAbs_REVERSED:orientation==TopAbs_REVERSED?TopAbs_FORWARD:orientation;
 }
};
void ct_tangency_report(const IntSurf_PathPoint& p) {
 ga_pnt(p.Value());ga_flag(p.IsPassingPnt());ga_flag(p.IsTangent());ga_int(p.Multiplicity());
 for(int i=0;i<=p.Multiplicity();++i){double u,v;p.Parameters(i,u,v);ga_real(u);ga_real(v);}
 GA_TRY(ga_xyz(p.Direction3d().XYZ()));GA_TRY(ga_xy(p.Direction2d().XY()));
}
void ct_tangency_case(std::istream& input) {
 GaReader r{input};GA_TRY(auto surface=hl_contap_surface(r);occ::handle<CtTangencyDomain> domain=new CtTangencyDomain(surface);
 int mode=r.integer();gp_Dir direction=r.dir();Contap_ArcFunction arc;arc.Set(surface);Contap_SurfFunction function;function.Set(surface);
 if(mode==0){arc.Set(direction);function.Set(direction);}else if(mode==1){gp_Pnt eye(3,4,5);arc.Set(eye);function.Set(eye);}else{arc.Set(direction,.2);function.Set(direction,.2);}function.Set(1e-7);
 int seed=r.integer(),orientation=r.integer();Contap_TheSearch search;search.Perform(arc,domain,1e-7,1e-7,false);
 ga_flag(search.IsDone());ga_int(search.NbPoints());ga_int(search.NbSegments());
 NCollection_Array1<int> destination(1,std::max(1,search.NbPoints()));
 for(int i=1;i<=destination.Upper();++i)destination(i)=seed==1&&i%2==0?-3:seed==2&&i%2!=0?2:0;
 NCollection_Sequence<IntSurf_PathPoint> output;if(seed==3)output.Append(IntSurf_PathPoint(gp_Pnt(7,8,9),7,8));
 domain->Mode=orientation;GA_TRY(ComputeTangency(search,domain,function,output,destination));ga_int(output.Length());
 for(const auto& p:output)ct_tangency_report(p);for(int i=1;i<=search.NbPoints();++i)ga_int(destination(i)););ga_end();
}
int main() {
 std::string line;while(std::getline(std::cin,line)){std::istringstream input(line);std::string operation;input>>operation;
 if(operation.empty())continue;if(operation!="tangency"){std::cerr<<"Unknown probe operation: "<<operation<<std::endl;return 2;}ct_tangency_case(input);}
}
