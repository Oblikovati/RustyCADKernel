#include <TopoDS_Edge.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BOPTools_AlgoTools.hxx>
#include <BOPTools_AlgoTools2D.hxx>
#include <Geom_Line.hxx>
#include <IntTools_Context.hxx>
#include <iostream>
#include <iomanip>
int main(){
 std::cout<<std::setprecision(17);
 for(int kind=0;kind<7;kind++)for(int first=0;first<4;first++)for(int second=0;second<4;second++){
  Handle(Geom_Curve) curve=new Geom_Line(gp_Pnt(0,0,0),gp_Dir(1,0,0));
  Handle(Geom_Curve) other=kind==0?curve:Handle(Geom_Curve)(new Geom_Line(gp_Pnt(0,0,0),kind==2?gp_Dir(-1,0,0):kind==3?gp_Dir(0,1,0):gp_Dir(1,0,0)));
  TopoDS_Edge split=BRepBuilderAPI_MakeEdge(curve,0.,1.);TopoDS_Edge original=BRepBuilderAPI_MakeEdge(other,-2.,2.);
  BRep_Builder builder;if(kind==4)builder.Degenerated(split,true);if(kind==5)builder.Degenerated(original,true);
  if(kind==6){builder.MakeEdge(split);builder.MakeEdge(original);}
  split.Orientation(TopAbs_Orientation(first));original.Orientation(TopAbs_Orientation(second));
  Handle(IntTools_Context) context=new IntTools_Context();int error=99;
  bool reverse=BOPTools_AlgoTools::IsSplitToReverse(split,original,context,&error);
  gp_Vec tangent(9,8,7);bool found=kind==6?false:BOPTools_AlgoTools2D::EdgeTangent(split,.5,tangent);
  std::cout<<kind<<' '<<first<<' '<<second<<' '<<reverse<<' '<<error<<' '<<found<<' '<<tangent.X()<<' '<<tangent.Y()<<' '<<tangent.Z()<<'\n';
 }
}
