#pragma once
#include <IntTools_EdgeEdge.hxx>
#include <memory>
void ee_full_snapshot(const IntTools_EdgeEdge& intersection,const TopoDS_Edge& first,const TopoDS_Edge& second){
 bo_int(intersection.IsDone());bo_real(intersection.FuzzyValue());bo_int(const_cast<IntTools_EdgeEdge&>(intersection).IsCoincidenceCheckedQuickly());bo_int(intersection.CommonParts().Length());
 for(const auto& part:intersection.CommonParts()){
  bo_int(part.Type());bo_int(part.AllNullFlag());bo_int(part.Edge1().IsSame(first));bo_int(part.Edge1().IsSame(second));bo_int(part.Edge2().IsSame(first));bo_int(part.Edge2().IsSame(second));
  bo_range(part.Range1());bo_real(part.VertexParameter1());bo_real(part.VertexParameter2());bo_int(part.Ranges2().Length());for(const auto& range:part.Ranges2())bo_range(range);
 }
}
void ee_full_perform(IntTools_EdgeEdge& intersection,const TopoDS_Edge& first,const TopoDS_Edge& second){try{intersection.Perform();}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}ee_full_snapshot(intersection,first,second);}
void ee_full(const std::vector<double>& v){
 TopoDS_Edge first=BRepBuilderAPI_MakeEdge(ee_geometry_fixture(int(v[0]),0.5),0,1),second=BRepBuilderAPI_MakeEdge(ee_geometry_fixture(int(v[1]),0.5),0,1);
 if(v[8]!=0){gp_Trsf placement;placement.SetTranslation(gp_Vec(v[8],0,v[8]/2));second.Move(TopLoc_Location(placement));}
 std::unique_ptr<IntTools_EdgeEdge> intersection;
 switch(int(v[7])){
  case 0:intersection.reset(new IntTools_EdgeEdge());intersection->SetEdge1(first,v[2],v[3]);intersection->SetEdge2(second,v[2],v[3]);break;
  case 1:intersection.reset(new IntTools_EdgeEdge(first,second));intersection->SetRange1(IntTools_Range(v[2],v[3]));intersection->SetRange2(IntTools_Range(v[2],v[3]));intersection->SetRange2(v[2],v[3]);break;
  default:intersection.reset(new IntTools_EdgeEdge(first,v[2],v[3],second,v[2],v[3]));break;
 }
 intersection->SetFuzzyValue(v[4]);intersection->UseQuickCoincidenceCheck(v[5]!=0);ee_full_snapshot(*intersection,first,second);
 ee_full_perform(*intersection,first,second);ee_full_perform(*intersection,first,second);
 if(v[6]!=0){TopoDS_Edge invalid;BRep_Builder builder;if(v[6]>=2)builder.MakeEdge(invalid);if(v[6]==2)builder.Degenerated(invalid,true);intersection->SetEdge1(invalid);ee_full_perform(*intersection,first,second);intersection->SetEdge1(first);intersection->SetEdge2(second);ee_full_perform(*intersection,first,second);}
}
