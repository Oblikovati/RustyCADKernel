// SPDX-License-Identifier: GPL-2.0-only
// Reproduce the curved-vertex discrepancy without changing the native classifier.
#include "occt_geometry.hpp"
#include <GeomAPI_ProjectPointOnSurf.hxx>
#include <array>
#include <iomanip>
std::vector<std::string> selected_fixture(const std::string& path,const std::string& name) {
  std::ifstream input(path);
  if (!input) throw std::runtime_error(path+"; expected readable corpus");
  std::string row;
  while (std::getline(input,row)) {
    auto fields=split(row,'\t');
    if (fields[0]==name) return fields;
  }
  throw std::runtime_error(name+"; expected a named corpus fixture");
}
std::array<double,2> baseline(const TopoDS_Face& face,const gp_Pnt& seed) {
  GeomAPI_ProjectPointOnSurf projection(seed,BRep_Tool::Surface(face));
  double u,v;
  projection.LowerDistanceParameters(u,v);
  std::cout<<"{\"probe\":\"explicit-xy-vertex\",\"exact_vertex\":"
    <<(exact_vertex(face,seed)?"true":"false")<<",\"surface_distance\":"
    <<projection.LowerDistance()<<",\"u\":"<<u<<",\"v\":"<<v<<"}\n";
  return {u,v};
}
void compare_tolerance(const TopoDS_Face& face,const gp_Pnt& seed,
                       const std::array<double,2>& uv,double tolerance) {
  BRepClass_FaceClassifier spatial(face,seed,tolerance);
  BRepClass_FaceClassifier planar(face,gp_Pnt2d(uv[0],uv[1]),tolerance);
  std::cout<<"{\"probe\":\"native-state\",\"tolerance\":"<<tolerance
    <<",\"spatial_state\":"<<spatial.State()<<",\"planar_state\":"<<planar.State()<<"}\n";
}
int main(int argc,char** argv) {
  if (argc!=3) throw std::runtime_error(std::to_string(argc)+"; expected corpus path and fixture name");
  const auto fields=selected_fixture(argv[1],argv[2]);
  const auto line=decode(fields[2]);
  const auto face=face3d(fields[1]);
  const gp_Pnt seed(line[0],line[1],0);
  std::cout<<std::setprecision(17);
  const auto uv=baseline(face,seed);
  for (double tolerance:{1e-12,1e-10,1e-7,1e-5}) compare_tolerance(face,seed,uv,tolerance);
}
