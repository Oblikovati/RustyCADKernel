// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; actual DS endpoints, not contact certificates.
#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <TopoDS.hxx>
#include <iostream>
#include <iomanip>
#include "martensite_partition_fixture.hxx"
static void EmitVertices(BOPDS_DS& ds)
{
  for(int index=0;index<ds.NbShapes();++index) {
    const auto& shape=ds.Shape(index);
    if(shape.ShapeType()!=TopAbs_VERTEX) continue;
    const auto point=BRep_Tool::Pnt(TopoDS::Vertex(shape));
    std::cout<<"vertex "<<index<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z()<<'\n';
  }
}
static void EmitSpans(BOPDS_DS& ds)
{
  for(int index=0;index<ds.NbSourceShapes();++index) {
    if(ds.ShapeInfo(index).ShapeType()!=TopAbs_EDGE) continue;
    for(const auto& block:ds.PaveBlocks(index)) {
      int first,last;double a,b;block->Indices(first,last);block->Range(a,b);
      std::cout<<"span "<<index<<' '<<first<<' '<<last<<' '<<a<<' '<<b<<'\n';
    }
  }
}
static void Capture(int mode)
{
  BOPAlgo_PaveFiller filler;filler.SetArguments(PartitionBindingSources(mode));
  filler.SetNonDestructive(true);filler.Perform();
  if(filler.HasErrors()) throw Standard_Failure("endpoint filler HasErrors=true; expected completed observed DS");
  std::cout<<"mode "<<mode<<'\n';
  EmitVertices(*filler.PDS());EmitSpans(*filler.PDS());
}
int main()
{
  std::cout<<std::setprecision(17);
  Capture(0);Capture(1);
  return 0;
}
