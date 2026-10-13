// SPDX-License-Identifier: GPL-2.0-only
// OCCT 8.0.0 evaluates independent cubic Bezier spans from Go's station derivatives.
#include <Geom2d_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>
struct Station { double parameter; gp_Pnt2d point; gp_Vec2d derivative; };
std::vector<std::string> split_row(const std::string &line) {
  std::vector<std::string> result; std::stringstream stream(line); std::string field;
  while (std::getline(stream, field, ',')) result.push_back(field);
  return result;
}
std::vector<std::vector<Station>> read_stations(const std::string &path) {
  std::ifstream source(path);
  if (!source) throw std::runtime_error("path " + path + ": expected Go station CSV");
  std::vector<std::vector<Station>> cases(6); std::string line;
  while (std::getline(source, line)) {
    const auto row = split_row(line);
    if (row.at(1) != "station") continue;
    cases.at(std::stoul(row.at(0))).push_back({std::stod(row.at(3)),
      gp_Pnt2d(std::stod(row.at(4)),std::stod(row.at(5))),
      gp_Vec2d(std::stod(row.at(6)),std::stod(row.at(7)))});
  }
  return cases;
}
void emit_span(std::size_t index,std::size_t span,const Station &first,const Station &last) {
  const double width=last.parameter-first.parameter;
  NCollection_Array1<gp_Pnt2d> poles(1,4);
  poles(1)=first.point; poles(2)=first.point.Translated(first.derivative*(width/3));
  poles(3)=last.point.Translated(last.derivative*(-width/3)); poles(4)=last.point;
  Geom2d_BezierCurve curve(poles);
  for(int step=0;step<=8;++step) {
    const double local=double(step)/8;
    gp_Pnt2d point; gp_Vec2d d1,d2;
    curve.D2(local,point,d1,d2); d1/=width; d2/=(width*width);
    std::cout<<index<<','<<span<<','<<first.parameter+local*width<<','<<point.X()<<','<<point.Y()
      <<','<<d1.X()<<','<<d1.Y()<<','<<d2.X()<<','<<d2.Y()<<'\n';
  }
}
int main(int argc,char **argv) {
  if(argc!=2) throw std::runtime_error("argument count "+std::to_string(argc)+": expected one CSV path");
  const auto cases=read_stations(argv[1]); std::cout<<std::setprecision(17);
  for(std::size_t index=0;index<cases.size();++index)
    for(std::size_t span=0;span+1<cases[index].size();++span)
      emit_span(index,span,cases[index][span],cases[index][span+1]);
}
