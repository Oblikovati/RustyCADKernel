// Include after the unchanged IntTools_EdgeEdge.cxx translation unit.
#define main bo_public_main
#include "tkbo_foundations_probe.cxx"
#undef main
#include "tkbo_edge_private_fixtures.cxx"
void ee_math(const std::vector<double>& v){
 NCollection_Sequence<IntTools_Range> output;output.Append(IntTools_Range(17,19));bo_int(SplitRangeOnSegments(v[0],v[1],v[2],int(v[3]),output));bo_int(output.Length());for(const auto& r:output)bo_range(r);
 Bnd_Box box;box.Add(gp_Pnt(-1,-2,-3));box.Add(gp_Pnt(4,5,6));box.SetGap(v[4]);bo_real(PointBoxDistance(box,gp_Pnt(v[5],v[6],v[7])));
}
void ee_geometry(const std::vector<double>& v){
 auto source=ee_geometry_fixture(int(v[0]),v[5]);TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(source,0,1);BRepAdaptor_Curve curve(edge);auto kind=curve.GetType();bo_int(TypeToInteger(kind));IntTools_Range range(v[1],v[2]);
 double coefficient=ResolutionCoeff(curve,range);bo_real(coefficient);auto local=curve.Curve().Curve();
 try{bo_real(Resolution(local,kind,coefficient,v[3]));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try{bo_real(CurveDeflection(curve,range));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try{bo_int(IsClosed(source,v[1],v[2],v[3],v[4]));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
#include "tkbo_edge_private_distance.cxx"
#include "tkbo_edge_private_parameters.cxx"
#include "tkbo_edge_private_pairs.cxx"
#include "tkbo_edge_private_lines.cxx"
int main(){
 std::string line;while(std::getline(std::cin,line)){std::istringstream input(line);std::string operation;input>>operation;double x;std::vector<double> v;while(input>>x)v.push_back(x);
 try{if(operation=="edge_lines")ee_lines(v);else if(operation=="edge_pairs")ee_pairs(v);else if(operation=="edge_parameters")ee_parameters(v);else if(operation=="edge_distance")ee_distance(v);else if(operation=="edge_math")ee_math(v);else if(operation=="edge_geometry")ee_geometry(v);else{return 2;}}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}std::cout<<'\n';}
}
