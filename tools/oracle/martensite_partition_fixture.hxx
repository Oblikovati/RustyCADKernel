// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; authored inputs, not analytic certificates.
#pragma once
#include <BOPAlgo_Builder.hxx>
#include <BOPAlgo_PaveFiller.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <Geom_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Trsf.hxx>
class PartitionBuilder final : public BOPAlgo_Builder {
  void CheckData() override
  {
    // Preserve filler diagnostics while admitting the extension's nonempty open edges.
    myReport->Merge(myPaveFiller->GetReport());
  }
};
static NCollection_List<TopoDS_Shape> PartitionQuadraticSources(int mode)
{
  NCollection_Array1<gp_Pnt> poles(1,3);
  poles(1)=gp_Pnt(0,0,0);poles(2)=gp_Pnt(.5,mode==0?1:0,0);
  poles(3)=gp_Pnt(1,mode==0?0:1,0);
  occ::handle<Geom_BezierCurve> curve=new Geom_BezierCurve(poles);
  NCollection_List<TopoDS_Shape> sources;
  sources.Append(BRepBuilderAPI_MakeEdge(curve).Edge());
  sources.Append(BRepBuilderAPI_MakeEdge(gp_Pnt(-1,.25,0),gp_Pnt(2,.25,0)).Edge());
  return sources;
}
static NCollection_List<TopoDS_Shape> PartitionRectangleSources()
{
  BRepBuilderAPI_MakePolygon polygon;
  for(const auto& point:{gp_Pnt(0,0,0),gp_Pnt(2,0,0),gp_Pnt(2,2,0),gp_Pnt(0,2,0)}) polygon.Add(point);
  polygon.Close();NCollection_List<TopoDS_Shape> sources;
  for(TopoDS_Iterator child(polygon.Wire());child.More();child.Next()) sources.Append(child.Value());
  return sources;
}
static void PartitionPlacedSources(NCollection_List<TopoDS_Shape>& sources)
{
  gp_Trsf shift;shift.SetTranslation(gp_Vec(2,3,0));
  const TopLoc_Location location(shift);
  for(auto& source:sources) source=source.Located(location);
}
static NCollection_List<TopoDS_Shape> PartitionBindingSources(int mode)
{
  if(mode<2) return PartitionQuadraticSources(mode);
  auto sources=PartitionRectangleSources();
  if(mode==3) for(auto& source:sources) source.Reverse();
  if(mode==4) PartitionPlacedSources(sources);
  if(mode==5) {auto separate=PartitionRectangleSources();sources.Append(separate);}
  if(mode==6) sources.Append(sources.First().Reversed());
  return sources;
}
