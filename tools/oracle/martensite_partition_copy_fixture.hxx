// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; shared authored copy/area fixture only.
#pragma once
#include "martensite_partition_fixture.hxx"
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepLib.hxx>
#include <BRep_Builder.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS.hxx>
#include <TopExp.hxx>
#include <gp.hxx>
#include <gp_Pln.hxx>
#include <vector>
using PartitionShapeIndex=NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>;
static NCollection_List<TopoDS_Shape> IsolatedInputs(const NCollection_List<TopoDS_Shape>& originals)
{
  TopoDS_Compound aggregate;BRep_Builder().MakeCompound(aggregate);
  for(const auto& original:originals) BRep_Builder().Add(aggregate,original);
  BRepBuilderAPI_Copy copier(aggregate,true,true);NCollection_List<TopoDS_Shape> sources;
  for(const auto& original:originals) {
    const auto& modified=copier.Modified(original);
    if(modified.Size()!=1) throw Standard_Failure("copied source image count != 1; expected one actual source image");
    sources.Append(modified.First().Oriented(original.Orientation()));
  }
  return sources;
}

static TopoDS_Face SupportAndInputsOnFace(const TopoDS_Shape& copied,NCollection_List<TopoDS_Shape>& inputs,
                                        const TopoDS_Face& support)
{
  PartitionShapeIndex edges;TopExp::MapShapes(copied,TopAbs_EDGE,edges);
  for(int index=1;index<=edges.Size();++index) {
    const auto edge=TopoDS::Edge(edges(index));BRepLib::BuildPCurveForEdgeOnPlane(edge,support);
    inputs.Append(edge.Oriented(TopAbs_FORWARD));inputs.Append(edge.Oriented(TopAbs_REVERSED));
  }
  return support;
}

static TopoDS_Face SupportAndInputs(const TopoDS_Shape& copied,NCollection_List<TopoDS_Shape>& inputs)
{
  const auto support=BRepBuilderAPI_MakeFace(gp_Pln(gp::XOY())).Face();
  return SupportAndInputsOnFace(copied,inputs,support);
}

static void CollectDirectedMembers(const TopoDS_Shape& member,std::vector<TopoDS_Shape>& members)
{
  for(const auto& existing:members) if(existing.IsEqual(member)) return;
  members.push_back(member);
  for(TopoDS_Iterator child(member,true,true);child.More();child.Next())
    CollectDirectedMembers(child.Value(),members);
}
