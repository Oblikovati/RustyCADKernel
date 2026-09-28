// Query OCCT itself: preserve full result order and binary64 output bits.
#include <NCollection_KDTree.hxx>
#include <array>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <sstream>
#include <vector>
struct Point {std::array<double,3> xyz{};double Coord(int i) const {return xyz[i-1];}};
uint64_t bits(double value) {uint64_t result;std::memcpy(&result,&value,8);return result;}
Point readPoint(std::istream& input,int dimension) {Point point;for(int i=0;i<dimension;++i)input>>point.xyz[i];return point;}
template<class Range> void printRange(const Range& range) {std::cout<<' '<<range.Size();for(size_t i=0;i<range.Size();++i)std::cout<<' '<<range[i];}
template<int D> void probe(std::istream& input) {
  size_t count;input>>count;std::vector<Point> points;std::vector<double> radii;
  for(size_t i=0;i<count;++i){points.push_back(readPoint(input,D));double radius;input>>radius;radii.push_back(radius);}
  Point query=readPoint(input,D),minimum=readPoint(input,D),maximum=readPoint(input,D);
  double radius,tolerance;size_t k;input>>radius>>tolerance>>k;
  NCollection_KDTree<Point,D,true> tree;tree.Build(points.data(),radii.data(),count);
  double distance,gap;auto nearest=tree.NearestPoint(query,distance);auto weighted=tree.NearestWeighted(query,gap);
  std::cout<<nearest<<' '<<bits(distance)<<' '<<weighted<<' '<<bits(gap);
  printRange(tree.NearestPoints(query,tolerance,distance));std::cout<<' '<<bits(distance);
  printRange(tree.RangeSearch(query,radius));
  NCollection_DynamicArray<size_t> visited;tree.ForEachInRange(query,radius,[&](size_t index){visited.Append(index);});printRange(visited);
  printRange(tree.BoxSearch(minimum,maximum));printRange(tree.ContainingSearch(query));
  NCollection_Array1<size_t> indices;NCollection_Array1<double> distances;
  auto found=tree.KNearestPoints(query,k,indices,distances);std::cout<<' '<<found;
  for(size_t i=1;i<=found;++i)std::cout<<' '<<indices.Value(i)<<' '<<bits(distances.Value(i));
  std::cout<<'\n';
}
int main(){std::string line;while(std::getline(std::cin,line)){std::istringstream input(line);int dimension;input>>dimension;if(dimension==2)probe<2>(input);else probe<3>(input);}}
