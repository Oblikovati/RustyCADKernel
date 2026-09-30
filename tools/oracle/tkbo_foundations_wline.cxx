// IntTools_WLineTool preserves first-surface bounds reuse and final negated flag.
#include <IntTools_WLineTool.hxx>
#include <IntSurf_LineOn2S.hxx>
void bo_wline_case(const std::vector<double>& v) {
 try{auto first=bo_fclass_face(int(v[0]),v[2]),last=bo_fclass_face(int(v[1]),v[2]);
 occ::handle<IntSurf_LineOn2S> points=new IntSurf_LineOn2S;IntSurf_PntOn2S p;p.SetValue(v[3],v[4],v[5],v[6]);points->Add(p);p.SetValue(v[5],v[6],v[3],v[4]);points->Add(p);
 occ::handle<IntPatch_WLine> line=new IntPatch_WLine(points,false);
 bo_int(IntTools_WLineTool::NotUseSurfacesForApprox(first,last,line,1,2));bo_int(IntTools_WLineTool::NotUseSurfacesForApprox(first,last,line,2,1));
 bo_int(IntTools_WLineTool::NotUseSurfacesForApprox(last,first,line,1,2));bo_int(IntTools_WLineTool::NotUseSurfacesForApprox(first,last,line,1,3));
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
