// Exact numeric oracle for the TKGeomBase packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkgeombase_numeric.py compares the
// output with crates/tkgeombase/examples/numeric_probe.rs bit for bit. Each package adds
// its own tkgeombase_probe_<package>.cxx, included below, and its rows in probe_cases.
#include <map>
#include <memory>
#include <vector>
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include <Standard_Failure.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Vec.hxx>
#include <gp_Vec2d.hxx>
#include "tkgeombase_probe_common.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
};
int main() {
 std::string line;
 while(std::getline(std::cin,line)) {
  std::istringstream input(line);std::string op;input>>op;
  if(op.empty())continue;
  auto found=probe_cases.find(op);
  if(found==probe_cases.end()) {std::cerr<<"Unknown probe operation: "<<op<<std::endl;return 2;}
  found->second(input);
 }
}
