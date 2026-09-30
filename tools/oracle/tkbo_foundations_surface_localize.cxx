// Surface grid aliasing, strict frames, map order, and null-source assignment at pinned OCCT 3d097a0.
#include <IntTools_SurfaceRangeLocalizeData.hxx>
using SurfaceCache=IntTools_SurfaceRangeLocalizeData;
void bo_surface_cache_point(const gp_Pnt& p) {bo_real(p.X());bo_real(p.Y());bo_real(p.Z());}
void bo_surface_cache_state(const SurfaceCache& c) {
 bo_int(c.GetNbSampleU());bo_int(c.GetNbSampleV());bo_real(c.GetMinRangeU());bo_real(c.GetMinRangeV());bo_real(c.GetGridDeflection());
 bo_int(c.GetRangeUGrid());bo_int(c.GetRangeVGrid());bo_int(c.GetNBUPointsInFrame());bo_int(c.GetNBVPointsInFrame());
}
void bo_surface_fill(SurfaceCache& c,const std::vector<double>& v) {
 c.SetRangeVGrid(int(v[1]));c.SetRangeUGrid(int(v[0]));
 for(int u=1;u<=int(v[0]);++u)c.SetUParam(u,double(u)*v[2]);
 for(int w=1;w<=int(v[1]);++w)c.SetVParam(w,double(w)*v[3]);
 for(int u=1;u<=int(v[0]);++u)for(int w=1;w<=int(v[1]);++w)c.SetGridPoint(u,w,gp_Pnt(double(u),double(w),-0.0));
}
void bo_surface_frame(SurfaceCache& c,const std::vector<double>& v) {
 c.SetFrame(v[4],v[5],v[6],v[7]);bo_surface_cache_state(c);
 for(int i=1;i<=std::max(0,c.GetNBUPointsInFrame())+1;++i)bo_real(c.GetUParamInFrame(i));
 for(int i=1;i<=std::max(0,c.GetNBVPointsInFrame())+1;++i)bo_real(c.GetVParamInFrame(i));
 for(int u=1;u<=std::max(0,c.GetNBUPointsInFrame())+1;++u)
  for(int w=1;w<=std::max(0,c.GetNBVPointsInFrame())+1;++w)bo_surface_cache_point(c.GetPointInFrame(u,w));
}
void bo_surface_sharing(SurfaceCache& c) {
 SurfaceCache copy(c);copy.SetUParam(1,77.0);copy.SetVParam(1,78.0);copy.SetGridPoint(1,1,gp_Pnt(79.,80.,81.));
 bo_real(c.GetUParam(1));bo_real(c.GetVParam(1));bo_surface_cache_point(c.GetGridPoint(1,1));
 copy.Assign(SurfaceCache());bo_surface_cache_state(copy);
 bo_real(copy.GetUParam(1));bo_real(copy.GetVParam(1));bo_surface_cache_point(copy.GetGridPoint(1,1));
 copy.ClearGrid();bo_surface_cache_state(copy);bo_surface_cache_state(c);
}
void bo_surface_grid_case(const std::vector<double>& v) {
 SurfaceCache c;bo_surface_cache_state(c);bo_surface_frame(c,v);c=SurfaceCache(3,-2,-0.0,0.125);c.SetGridDeflection(-0.25);bo_surface_fill(c,v);
 bo_surface_frame(c,v);c.SetRangeUGrid(int(v[0]));c.SetRangeVGrid(int(v[1]));bo_surface_cache_state(c);bo_surface_sharing(c);
 c.SetRangeUGrid(int(v[0])+1);bo_surface_cache_point(c.GetGridPoint(1,1));c.SetRangeVGrid(int(v[1])+1);bo_surface_cache_point(c.GetGridPoint(1,1));
 c.ClearGrid();bo_surface_frame(c,v);
}
void bo_surface_key_state(const SurfaceCache& c,const IntTools_SurfaceRangeSample& key) {
 Bnd_Box b;b.SetWhole();bo_int(c.IsRangeOut(key));bo_int(c.FindBox(key,b));bo_int(b.IsWhole());bo_int(b.IsVoid());
}
void bo_surface_listing(const SurfaceCache& c) {
 NCollection_List<IntTools_SurfaceRangeSample> samples;samples.Append(IntTools_SurfaceRangeSample());c.ListRangeOut(samples);c.ListRangeOut(samples);bo_int(samples.Size());
 for(const auto& key:samples) {bo_int(key.GetIndexU());bo_int(key.GetDepthU());bo_int(key.GetIndexV());bo_int(key.GetDepthV());}
}
void bo_surface_cache_case(const std::vector<double>& v) {
 SurfaceCache c;bo_surface_key_state(c,IntTools_SurfaceRangeSample());
 for(int i=0;i<int(v[0]);++i) {IntTools_SurfaceRangeSample key(i%137,i%3,i%7,i%5);c.AddBox(key,Bnd_Box());c.AddOutRange(key);bo_surface_key_state(c,key);}
 bo_surface_listing(c);bo_surface_listing(SurfaceCache(c));SurfaceCache assigned;assigned.Assign(c);bo_surface_listing(assigned);
 c.RemoveRangeOutAll();bo_surface_listing(c);bo_surface_key_state(c,IntTools_SurfaceRangeSample());
}
