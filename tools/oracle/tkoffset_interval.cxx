#include <BRepOffset_Interval.hxx>
#include <iostream>
#include <limits>
#include <cmath>
int main(){
 for(auto type:{ChFiDS_Concave,ChFiDS_Convex,ChFiDS_Tangential,ChFiDS_FreeBound,ChFiDS_Other,ChFiDS_Mixed}){
  BRepOffset_Interval interval(6.,-2.,type);std::cout<<interval.First()<<' '<<interval.Last()<<' '<<interval.Type()<<'\n';
  interval.First(-7.);interval.Last(9.);interval.Type(ChFiDS_Other);std::cout<<interval.First()<<' '<<interval.Last()<<' '<<interval.Type()<<'\n';
 }
 BRepOffset_Interval unset;unset.First(-0.);unset.Last(std::numeric_limits<double>::infinity());unset.Type(ChFiDS_Mixed);
 std::cout<<std::signbit(unset.First())<<' '<<std::isinf(unset.Last())<<' '<<unset.Type()<<'\n';
 unset.First(std::numeric_limits<double>::quiet_NaN());std::cout<<std::isnan(unset.First())<<'\n';
}
