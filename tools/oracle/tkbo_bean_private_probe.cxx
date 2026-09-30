// Include after the unchanged IntTools_BeanFaceIntersector translation unit.
#define main bo_public_main
#include "tkbo_foundations_probe.cxx"
#undef main
IntTools_CurveRangeSample bean_curve(int index,int depth){IntTools_CurveRangeSample range(index);range.SetDepth(depth);return range;}
IntTools_SurfaceRangeSample bean_surface(int index,int depth){return IntTools_SurfaceRangeSample(index,depth,index%3,depth-1);}
void bean_snapshot(const IntTools_MarkedRangeSet& ranges){bo_int(ranges.Length());for(int i=1;i<=ranges.Length();++i){bo_int(ranges.Flag(i));bo_range(ranges.Range(i));}}
void bean_empty(const std::vector<double>& v){NCollection_Array1<double> bounds(0,2);bounds(0)=v[0];bounds(1)=v[1];bounds(2)=v[2];IntTools_MarkedRangeSet ranges(bounds,int(v[3]));for(double parameter:{v[4],v[5],v[4]}){bo_int(SetEmptyResultRange(parameter,ranges));bean_snapshot(ranges);}}
void bean_group(const std::vector<double>& v){
 NCollection_List<IntTools_CurveRangeSample> curves,sortedCurves;NCollection_List<IntTools_SurfaceRangeSample> surfaces,sortedSurfaces;
 for(int i=0;i<int(v[0]);++i)curves.Append(bean_curve(i,i%4));for(int i=0;i<int(v[1]);++i)surfaces.Append(bean_surface((i*7)%int(v[2]),i%2));
 sortedCurves.Append(bean_curve(17,19));sortedSurfaces.Append(bean_surface(23,29));MergeSolutions(curves,surfaces,sortedCurves,sortedSurfaces);
 bo_int(sortedCurves.Size());bo_int(sortedSurfaces.Size());NCollection_List<IntTools_CurveRangeSample>::Iterator c(sortedCurves);NCollection_List<IntTools_SurfaceRangeSample>::Iterator s(sortedSurfaces);
 for(;c.More()&&s.More();c.Next(),s.Next()){bo_int(c.Value().GetRangeIndex());bo_int(c.Value().GetDepth());bo_int(s.Value().GetIndexU());bo_int(s.Value().GetDepthU());bo_int(s.Value().GetIndexV());bo_int(s.Value().GetDepthV());}
}
void bean_sampling(const std::vector<double>& v){
 auto curve=bean_curve(0,int(v[0]));IntTools_SurfaceRangeSample surface(0,int(v[1]),0,int(v[2]));IntTools_CurveRangeLocalizeData curves(int(v[3]),v[6]);IntTools_SurfaceRangeLocalizeData surfaces(int(v[4]),int(v[5]),v[7],v[8]);bool a=false,b=false,c=false;CheckSampling(curve,surface,curves,surfaces,v[9],v[10],v[11],a,b,c);bo_int(a);bo_int(b);bo_int(c);
}
int main(){std::string line;while(std::getline(std::cin,line)){std::istringstream input(line);std::string operation;input>>operation;double number;std::vector<double> v;while(input>>number)v.push_back(number);try{if(operation=="bean_empty")bean_empty(v);else if(operation=="bean_group")bean_group(v);else if(operation=="bean_sampling")bean_sampling(v);else return 2;}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}std::cout<<'\n';}}
