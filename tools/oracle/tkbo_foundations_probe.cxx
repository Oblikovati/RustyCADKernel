// Immutable OCCT oracle for TKBO IntTools foundations at 3d097a0.
#include <IntTools_Range.hxx>
#include <IntTools_Root.hxx>
#include <IntTools_BaseRangeSample.hxx>
#include <IntTools_CurveRangeSample.hxx>
#include <IntTools_SurfaceRangeSample.hxx>
#include <iostream>
#include <sstream>
#include <vector>
#include <string>
#include <cstdint>
#include <cstring>
void bo_real(double value) {uint64_t bits;std::memcpy(&bits,&value,8);std::cout<<bits<<' ';}
void bo_int(int value) {std::cout<<value<<' ';}
void bo_range(const IntTools_Range& range) {bo_real(range.First());bo_real(range.Last());}
void bo_range_case(const std::vector<double>& v) {
 bo_range(IntTools_Range());IntTools_Range range(v[0],v[1]);bo_range(range);
 range.SetFirst(v[2]);range.SetLast(v[3]);double first,last;range.Range(first,last);bo_real(first);bo_real(last);
}
void bo_root_state(const IntTools_Root& root) {
 bo_real(root.Root());bo_int(root.Type());bo_real(root.LayerHeight());
 bo_int(root.StateBefore());bo_int(root.StateAfter());bo_int(root.IsValid());
 double t1,t2,f1,f2;root.Interval(t1,t2,f1,f2);for(double value:{t1,t2,f1,f2})bo_real(value);
}
void bo_root_case(const std::vector<double>& v) {
 bo_root_state(IntTools_Root());IntTools_Root root(v[0],int(v[1]));bo_root_state(root);
 root.SetRoot(v[2]);root.SetType(int(v[3]));root.SetLayerHeight(v[4]);
 root.SetStateBefore(TopAbs_State(int(v[5])));root.SetStateAfter(TopAbs_State(int(v[6])));
 root.SetInterval(v[7],v[8],v[9],v[10]);bo_root_state(root);
}
void bo_curve_state(const IntTools_CurveRangeSample& sample,const std::vector<double>& v) {
 bo_int(sample.GetDepth());bo_int(sample.GetRangeIndex());bo_int(sample.GetRangeIndexDeeper(int(v[4])));
 bo_range(sample.GetRange(v[2],v[3],int(v[4])));std::cout<<std::hash<IntTools_CurveRangeSample>{}(sample)<<' ';
}
void bo_curve_case(const std::vector<double>& v) {
 IntTools_BaseRangeSample base;bo_int(base.GetDepth());base=IntTools_BaseRangeSample(int(v[0]));bo_int(base.GetDepth());
 base.SetDepth(int(v[1]));bo_int(base.GetDepth());IntTools_CurveRangeSample sample;bo_curve_state(sample,v);
 sample=IntTools_CurveRangeSample(int(v[5]));sample.SetDepth(int(v[0]));bo_curve_state(sample,v);bo_int(sample.IsEqual(sample));
 sample.SetRangeIndex(int(v[6]));sample.SetDepth(int(v[1]));bo_curve_state(sample,v);bo_int(sample==IntTools_CurveRangeSample());
}
void bo_surface_state(const IntTools_SurfaceRangeSample& sample) {
 IntTools_CurveRangeSample u,v;sample.GetRanges(u,v);bo_int(u==sample.GetSampleRangeU());bo_int(v==sample.GetSampleRangeV());
 int ui,vi,ud,vd;sample.GetIndexes(ui,vi);bo_int(ui);bo_int(vi);sample.GetDepths(ud,vd);bo_int(ud);bo_int(vd);
 bo_int(sample.GetIndexU());bo_int(sample.GetIndexV());bo_int(sample.GetDepthU());bo_int(sample.GetDepthV());
 bo_int(sample.GetRangeIndexUDeeper(3));bo_int(sample.GetRangeIndexVDeeper(2));
 bo_range(sample.GetRangeU(-3.,7.,3));bo_range(sample.GetRangeV(9.,-1.,2));std::cout<<std::hash<IntTools_SurfaceRangeSample>{}(sample)<<' ';
}
void bo_surface_case(const std::vector<double>& v) {
 bo_surface_state(IntTools_SurfaceRangeSample());IntTools_SurfaceRangeSample sample{int(v[0]),int(v[1]),int(v[2]),int(v[3])};
 bo_surface_state(sample);IntTools_SurfaceRangeSample copy(sample);bo_int(copy.IsEqual(sample));
 sample.SetIndexes(int(v[4]),int(v[5]));sample.SetDepthU(int(v[6]));sample.SetDepthV(int(v[7]));bo_surface_state(sample);
 sample.SetIndexU(int(v[8]));sample.SetIndexV(int(v[9]));IntTools_CurveRangeSample u,w;sample.GetRanges(u,w);
 IntTools_SurfaceRangeSample other(w,u);bo_surface_state(other);other.Assign(sample);bo_int(other==sample);
 other.SetRanges(w,u);bo_surface_state(other);other.SetSampleRangeU(u);other.SetSampleRangeV(w);bo_surface_state(other);
}
#include "tkbo_foundations_marked.cxx"
#include "tkbo_foundations_bopds.cxx"
#include "tkbo_foundations_intersections.cxx"
#include "tkbo_foundations_localize.cxx"
#include "tkbo_foundations_surface_localize.cxx"
#include "tkbo_foundations_topol.cxx"
#include "tkbo_foundations_fclass.cxx"
#include "tkbo_foundations_context.cxx"
#include "tkbo_foundations_context_compute.cxx"
#include "tkbo_foundations_tools.cxx"
#include "tkbo_foundations_tools_curves.cxx"
#include "tkbo_foundations_tools_topology.cxx"
#include "tkbo_foundations_wline.cxx"
#include "tkbo_foundations_wline_decompose.cxx"
#include "tkbo_foundations_face_state.cxx"
int main() {
 std::string line;
 while(std::getline(std::cin,line)) {
  std::istringstream input(line);std::string operation;input>>operation;std::vector<double> values;double value;
  while(input>>value)values.push_back(value);
#ifdef TKBO_FACEFACE_PRIVATE_PROBE
  if(operation=="face_clip"){bo_face_clip_case(values);std::cout<<'\n';continue;}
#endif
  if(operation=="face_state")bo_face_state_case(values);else if(operation=="wline_decompose")bo_wline_decompose_case(values);else if(operation=="wline")bo_wline_case(values);else if(operation=="tools_topology")bo_tools_topology_case(values);else if(operation=="tools_surface")bo_tools_surface_case(values);else if(operation=="tools_curves")bo_tools_curves_case(values);else if(operation=="tools")bo_tools_case(values);else if(operation=="context_compute")bo_context_compute_case(values);else if(operation=="context")bo_context_case(values);else if(operation=="fclass")bo_fclass_case(values);else if(operation=="topol")bo_topol_case(values);else if(operation=="surface_grid")bo_surface_grid_case(values);else if(operation=="surface_cache")bo_surface_cache_case(values);else if(operation=="localize")bo_local_case(values);else if(operation=="points")bo_points_case(values);else if(operation=="common")bo_common_case(values);else if(operation=="intersection_curve")bo_intersection_curve_case(values);else if(operation=="bop_range")bo_index_case(values);else if(operation=="bop_pave")bo_pave_case(values);else if(operation=="bop_pair")bo_pair_case(values);else if(operation=="bop_point")bo_point_case(values);else if(operation=="marked")bo_marked_case(values);else if(operation=="range")bo_range_case(values);else if(operation=="root")bo_root_case(values);
  else if(operation=="curve")bo_curve_case(values);else if(operation=="surface")bo_surface_case(values);
  else {std::cerr<<"Unknown probe operation: "<<operation<<'\n';return 2;}
  std::cout<<'\n';
 }
}
