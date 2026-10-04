#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <TopoDS_Vertex.hxx>
int main(int argc,char**){std::cout<<std::setprecision(17);
 if(argc>1){for(int kind=0;kind<5;kind++)for(int index=0;index<3;index++){
  auto fixture=pcurve_fixture(kind);auto edge=BRepBuilderAPI_MakeEdge(fixture.curve,fixture.first,fixture.last).Edge();
  double parameter=fixture.first+(fixture.last-fixture.first)*index/2.;gp_Pnt point;
  BOPTools_AlgoTools::PointOnEdge(edge,parameter,point);std::cout<<kind<<' '<<index<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z()<<'\n';
 }return 0;}
 for(int kind=0;kind<9;kind++)for(int initial=0;initial<2;initial++){
  TopoDS_Shape shape;if(initial){TopoDS_Vertex vertex;BRep_Builder().MakeVertex(vertex);shape=vertex;shape.Orientation(TopAbs_REVERSED);}auto original=shape;
  BOPTools_AlgoTools::MakeContainer(TopAbs_ShapeEnum(kind),shape);
  std::cout<<kind<<' '<<initial<<' '<<shape.IsNull()<<' '<<shape.IsSame(original);
  if(!shape.IsNull())std::cout<<' '<<int(shape.ShapeType())<<' '<<int(shape.Orientation());std::cout<<'\n';
 }
}
