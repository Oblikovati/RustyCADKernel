// SPDX-License-Identifier: GPL-2.0-only
// Independent OCCT 8.0.0 face classification for ADR 0010, kernel issue #10.
#include "occt_geometry.hpp"
void record(const std::vector<std::string>& fields) {
  const auto line=decode(fields.at(2));
  const auto face=face3d(fields.at(1));
  // Supplementary ordinary-input tolerance; exact equations own boundary evidence.
  const gp_Pnt seed(line.at(0),line.at(1),0);
  const BRepClass_FaceClassifier result(face,seed,1e-10);
  const auto state=result.State();
  if (state!=TopAbs_IN && state!=TopAbs_OUT && state!=TopAbs_ON)
    throw std::runtime_error(fields[0]+"; expected decided native point classification");
  std::cout<<fields[0]<<'\t'<<(state==TopAbs_IN ? "inside" : state==TopAbs_OUT ? "outside" : "boundary")<<"\tvalid\t"<<(exact_vertex(face,seed) ? "vertex" : "nonvertex")<<'\n';
}
int main(int argc,char** argv) {
  static_assert(OCC_VERSION_MAJOR==8 && OCC_VERSION_MINOR==0 && OCC_VERSION_MAINTENANCE==0);
  if (argc!=2) throw std::runtime_error(std::to_string(argc)+"; expected corpus path");
  std::ifstream input(argv[1]);
  if (!input) throw std::runtime_error(std::string(argv[1])+"; expected readable corpus");
  std::string row;
  while (std::getline(input,row)) record(split(row,'\t'));
  return 0;
}
