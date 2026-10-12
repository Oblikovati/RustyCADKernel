// Real StdPrs_Isolines output aliasing and initialized inline-array contracts; OCCT3d097a0.
#include "martensite_isoline_faces.hxx"
#include <StdPrs_Isolines.hxx>
#include <algorithm>
#include <iomanip>
#include <iostream>
#include <limits>
static void print_isoline_sequence(const NCollection_Sequence<double>& values) {
  std::cout<<' '<<values.Length();
  for(double value:values)std::cout<<' '<<value;
}
static void output_alias_case(int scenario) {
  const auto face=isoline_fixture_face(scenario/3);
  NCollection_Sequence<double> u,v;u.Append(91.);v.Append(92.);
  double bounds[4]={71.,72.,73.,74.};
  const bool same=scenario%3==1,scalar=scenario%3==2;
  StdPrs_Isolines::UVIsoParameters(face,2,3,100.,u,same?u:v,
    bounds[0],scalar?bounds[0]:bounds[1],scalar?bounds[0]:bounds[2],scalar?bounds[0]:bounds[3]);
  std::cout<<scenario;for(double value:bounds)std::cout<<' '<<value;
  print_isoline_sequence(u);print_isoline_sequence(v);std::cout<<'\n';
}
static StdPrs_Isolines::SegOnIso initialized_isoline_segment(int index) {
  StdPrs_Isolines::SegOnIso segment;
  segment.Pnts[0].Pnt=gp_Pnt(index,2.,3.);segment.Pnts[1].Pnt=gp_Pnt(index,5.,6.);
  segment.Pnts[0].Param=index%9;segment.Pnts[1].Param=segment.Pnts[0].Param+1.;
  return segment;
}
static void segment_reference_case() {
  auto segment=initialized_isoline_segment(2);
  StdPrs_Isolines::PntOnIso* writable=segment;
  const StdPrs_Isolines::PntOnIso* retained=segment;
  writable[1].Pnt=gp_Pnt(7.,8.,9.);writable[1].Param=4.;
  const auto other=initialized_isoline_segment(4);
  std::cout<<"30 "<<retained[1].Pnt.X()<<' '<<retained[1].Pnt.Y()<<' '<<retained[1].Pnt.Z();
  std::cout<<' '<<retained[1].Param<<' '<<(segment<other)<<'\n';
  segment.Pnts[1].Param=std::numeric_limits<double>::quiet_NaN();
  std::cout<<"31 "<<(segment<other)<<' '<<(other<segment)<<'\n';
}
static void sorted_segments_case() {
  std::vector<StdPrs_Isolines::SegOnIso> segments;
  for(int index=36;index>=0;--index)segments.push_back(initialized_isoline_segment(index));
  std::stable_sort(segments.begin(),segments.end());
  std::cout<<"32";for(const auto& segment:segments)std::cout<<' '<<segment.Pnts[0].Pnt.X();
  std::cout<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);
  for(int scenario=0;scenario<30;++scenario)output_alias_case(scenario);
  segment_reference_case();sorted_segments_case();
}
