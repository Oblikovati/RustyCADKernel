#include "tkbo_pcurve_fixture.hxx"
#include <BOPTools_AlgoTools3D.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
int main(){
 for(int kind=0;kind<5;kind++)for(int fori=0;fori<4;fori++){
  auto face=pcurve_fixture(kind).face;face.Orientation(TopAbs_Orientation(fori));int index=0;
  for(TopExp_Explorer ex(face,TopAbs_EDGE);ex.More();ex.Next(),index++)for(int eori=0;eori<4;eori++){
   auto edge=TopoDS::Edge(ex.Current());edge.Orientation(TopAbs_Orientation(eori));TopoDS_Edge result;
   BOPTools_AlgoTools3D::OrientEdgeOnFace(edge,face,result);
   std::cout<<kind<<' '<<fori<<' '<<index<<' '<<eori<<' '<<result.Orientation()<<' '<<result.IsSame(edge)<<'\n';
  }
 }
}
