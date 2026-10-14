// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0 actual aggregate copy and stored plane representations; not incidence completeness evidence.
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepLib.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopLoc_Location.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <TopoDS_Iterator.hxx>
#include <gp_Pln.hxx>
#include <array>
#include <iomanip>
#include <iostream>
#include <vector>
#include "martensite_analytic_edge_fixture.hxx"

static void EmitDescriptor(const char* name, int occurrence, int physical, const TopoDS_Edge& edge, const TopoDS_Face& support)
{
  BRepLib::BuildPCurveForEdgeOnPlane(edge,support);
  double first,last;
  const auto pcurve=occ::down_cast<Geom2d_BezierCurve>(BRep_Tool::CurveOnSurface(edge,support,first,last));
  if (pcurve.IsNull()) throw Standard_Failure("stored pcurve=null; expected copied planar Bezier");
  std::cout<<name<<' '<<occurrence<<' '<<physical<<' '<<edge.Orientation()<<' '<<first<<' '<<last<<' '<<pcurve->IsRational()<<' '<<pcurve->NbPoles();
  for (int i=1; i<=pcurve->NbPoles(); ++i) {
    const auto point=pcurve->Pole(i);
    std::cout<<' '<<point.X()<<','<<point.Y()<<','<<pcurve->Weight(i);
  }
  std::cout<<'\n';
}
static void EmitCopiedSources(const char* name, const std::vector<TopoDS_Shape>& sources)
{
  BRep_Builder builder;
  TopoDS_Compound aggregate;
  builder.MakeCompound(aggregate);
  for (const auto& source:sources) builder.Add(aggregate,source);
  BRepBuilderAPI_Copy copy(aggregate,true,true);
  const auto support=BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1))).Face();
  std::vector<TopoDS_Shape> occurrences;
  for (TopoDS_Iterator iterator(copy.Shape(),true,true); iterator.More(); iterator.Next()) {
    const auto shape=iterator.Value();
    int physical=static_cast<int>(occurrences.size());
    for (int i=0; i<physical; ++i) if (shape.IsSame(occurrences[i])) { physical=i; break; }
    EmitDescriptor(name,static_cast<int>(occurrences.size()),physical,TopoDS::Edge(shape),support);
    occurrences.push_back(shape);
  }
}
static void EmitSharedAndLocated()
{
  const auto first=CensusEdge({{0,0},{.5,0},{1,1}});
  gp_Trsf translation;
  translation.SetTranslation(gp_Vec(10,0,0));
  const auto located=first.Located(TopLoc_Location(translation));
  EmitCopiedSources("aliases",{first,first.Reversed(),located});
}
int main()
{
  std::cout<<std::setprecision(17);
  EmitCopiedSources("pair",{CensusEdge({{0,0},{.5,0},{1,1}}),CensusEdge({{0,.5},{.5,.5},{1,-.5}})});
  EmitCopiedSources("crossing",{CensusEdge({{3,-9},{-1,13},{-1,-13},{3,9}})});
  EmitCopiedSources("cusp",{CensusEdge({{3,-3},{-1,3},{-1,-3},{3,3}})});
  EmitCopiedSources("closed",{CensusEdge({{0,0},{4,0},{0,4},{0,0}})});
  EmitCopiedSources("rational",{CensusEdge({{0,0},{.5,1}},{1,2}),CensusEdge({{0,1},{1,0}})});
  EmitSharedAndLocated();
  return 0;
}
