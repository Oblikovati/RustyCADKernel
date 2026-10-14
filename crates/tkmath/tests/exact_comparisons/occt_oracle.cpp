// SPDX-License-Identifier: GPL-2.0-only
// OCCT 8.0.0 ordinary-range metric comparisons; exact Fraction fixtures cover extreme inputs.
#include <gp_Pnt2d.hxx>
#include <gp_Vec2d.hxx>
#include <array>
#include <cstring>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
std::array<gp_Pnt2d,4> read_points(const std::string &row) {
  std::stringstream stream(row); std::array<double,8> coordinates{};
  for (double &coordinate : coordinates) {
    std::string field; std::getline(stream,field,',');
    const std::uint64_t bits=std::stoull(field,nullptr,16);
    std::memcpy(&coordinate,&bits,sizeof(coordinate));
  }
  return {gp_Pnt2d(coordinates[0],coordinates[1]),gp_Pnt2d(coordinates[2],coordinates[3]),
    gp_Pnt2d(coordinates[4],coordinates[5]),gp_Pnt2d(coordinates[6],coordinates[7])};
}
int sign(double value) { return (value>0)-(value<0); }
int main(int argc,char **argv) {
  if(argc!=2) throw std::runtime_error("argument count "+std::to_string(argc)+": expected one rational corpus path");
  std::ifstream source(argv[1]); std::string row;
  if(!source) throw std::runtime_error("path "+std::string(argv[1])+": expected readable corpus");
  std::getline(source,row);
  for(int index=0;index<8;++index) {
    if(!std::getline(source,row)) throw std::runtime_error("row count "+std::to_string(index)+": expected eight ordinary cases");
    const auto points=read_points(row);
    const double distance=points[0].SquareDistance(points[1])-points[0].SquareDistance(points[2]);
    const double projection=gp_Vec2d(points[0],points[1]).Dot(gp_Vec2d(points[3],points[2]));
    std::cout<<index<<','<<sign(distance)<<','<<sign(projection)<<'\n';
  }
}
