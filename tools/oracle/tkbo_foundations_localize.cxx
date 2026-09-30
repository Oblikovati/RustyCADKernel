// IntTools_CurveRangeLocalizeData differential cache operations at OCCT 3d097a0.
#include <IntTools_CurveRangeLocalizeData.hxx>
IntTools_CurveRangeSample bo_local_sample(int index,int depth) {
 IntTools_CurveRangeSample result(index);result.SetDepth(depth);return result;
}
void bo_local_bounds(const Bnd_Box& bounds) {
 bo_int(bounds.IsVoid());bo_int(bounds.IsWhole());bo_real(bounds.GetGap());
 if(!bounds.IsVoid() && !bounds.IsWhole()) {
  gp_Pnt p=bounds.CornerMin(),q=bounds.CornerMax();
  for(const gp_Pnt& point:{p,q}) {bo_real(point.X());bo_real(point.Y());bo_real(point.Z());}
 }
}
void bo_local_state(const IntTools_CurveRangeLocalizeData& cache,const IntTools_CurveRangeSample& key) {
 Bnd_Box bounds;bounds.SetWhole();bo_int(cache.IsRangeOut(key));bo_int(cache.FindBox(key,bounds));bo_local_bounds(bounds);
}
void bo_local_apply(IntTools_CurveRangeLocalizeData& cache,int step,int depth) {
 auto key=bo_local_sample(step%137-30,depth+step%3);Bnd_Box bounds;
 bounds.Set(gp_Pnt(double(step),-0.0,-double(step)));bounds.SetGap(0.25);
 cache.AddBox(key,bounds);bo_local_state(cache,key);cache.AddOutRange(key);bo_local_state(cache,key);
 if(step%2==0) {cache.AddBox(key,Bnd_Box());cache.AddBox(key,bounds);}
 bo_local_state(cache,key);
}
void bo_local_case(const std::vector<double>& v) {
 IntTools_CurveRangeLocalizeData cache(int(v[0]),v[1]);bo_int(cache.GetNbSample());bo_real(cache.GetMinRange());
 bo_local_state(cache,IntTools_CurveRangeSample());
 for(int step=0;step<int(v[2]);++step)bo_local_apply(cache,step,int(v[3]));
 NCollection_List<IntTools_CurveRangeSample> output;output.Append(bo_local_sample(999,-1));
 cache.ListRangeOut(output);cache.ListRangeOut(output);bo_int(output.Size());
 for(const auto& key:output) {bo_int(key.GetRangeIndex());bo_int(key.GetDepth());}
}
