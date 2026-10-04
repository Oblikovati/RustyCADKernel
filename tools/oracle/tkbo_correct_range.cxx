#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_BezierCurve.hxx>
#include <IntTools_Range.hxx>
Handle(Geom_Curve) range_curve(int kind){if(kind<2)return pcurve_fixture(kind==0?0:2).curve;
 NCollection_Array1<gp_Pnt> poles(1,3);poles(1)=gp_Pnt();poles(2)=kind==2?gp_Pnt():gp_Pnt(.5,2,0);poles(3)=gp_Pnt(1,2,0);return new Geom_BezierCurve(poles);}
int main(){std::cout<<std::setprecision(17);for(int kind=0;kind<4;kind++)for(int r=0;r<4;r++)for(int t=0;t<3;t++)for(int faceMode=0;faceMode<2;faceMode++){
 auto edge=BRepBuilderAPI_MakeEdge(range_curve(kind),0.,1.).Edge();auto other=BRepBuilderAPI_MakeEdge(range_curve(0),0.,1.).Edge();auto face=pcurve_fixture(0).face;
 double tolerance=t==0?1e-7:t==1?.1:1.;BRep_Builder builder;builder.UpdateEdge(edge,tolerance);builder.UpdateEdge(other,2*tolerance);builder.UpdateFace(face,3*tolerance);
 double ranges[4][2]={{0,1},{0,1e-10},{1,0},{.2,.8}};IntTools_Range input(ranges[r][0],ranges[r][1]),output(9,8);
 if(faceMode)BOPTools_AlgoTools::CorrectRange(edge,face,input,output);else BOPTools_AlgoTools::CorrectRange(edge,other,input,output);
 std::cout<<kind<<' '<<r<<' '<<t<<' '<<faceMode<<' '<<output.First()<<' '<<output.Last()<<'\n';
}}
