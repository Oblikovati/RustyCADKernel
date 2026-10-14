// SPDX-License-Identifier: GPL-2.0-only
// Authored general pair/self/trim domains consume the same mapped construction.
#pragma once
#include "martensite_analytic_edge_fixture.hxx"
#include "martensite_partition_fixture.hxx"
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
static NCollection_List<TopoDS_Shape> ContactCurvedSources()
{
  NCollection_List<TopoDS_Shape> sources;
  sources.Append(CensusEdge({{0,0},{.5,0},{1,1}}));
  sources.Append(CensusEdge({{0,.5},{.5,.5},{1,-.5}}));
  return sources;
}
static NCollection_List<TopoDS_Shape> ContactRationalSources()
{
  NCollection_List<TopoDS_Shape> sources;
  sources.Append(CensusEdge({{0,0},{1,1}}));
  sources.Append(CensusEdge({{0,1},{1,0}},{1,2}));
  return sources;
}
static NCollection_List<TopoDS_Shape> ContactTrimmedSources()
{
  NCollection_List<TopoDS_Shape> sources;
  for(const auto& original:ContactCurvedSources()) {
    double first,last;const auto curve=BRep_Tool::Curve(TopoDS::Edge(original),first,last);
    sources.Append(BRepBuilderAPI_MakeEdge(curve,.25,.75).Edge());
  }
  return sources;
}
static NCollection_List<TopoDS_Shape> ContactDegenerateSources(int mode)
{
  auto sources=ContactCurvedSources();
  if(mode==16) { sources.Append(CensusEdge({{.5,-1},{.5,1}}));return sources; }
  sources.Clear();
  if(mode==14) {
    sources.Append(CensusEdge({{0,0},{.5,0},{1,1}}));
    sources.Append(CensusEdge({{0,0},{1,0}}));
    return sources;
  }
  sources.Append(CensusEdge({{0,0},{1,0}}));
  sources.Append(CensusEdge({{0,1e-8},{1,1e-8}}));
  return sources;
}
static NCollection_List<TopoDS_Shape> ContactTranslatedSources()
{
  auto sources=ContactCurvedSources();gp_Trsf movement;movement.SetTranslation(gp_Vec(2,3,5));
  for(NCollection_List<TopoDS_Shape>::Iterator it(sources);it.More();it.Next())
    it.ChangeValue().Location(TopLoc_Location(movement));
  return sources;
}
static NCollection_List<TopoDS_Shape> ContactTransportSources(int mode)
{
  if(mode<7) return PartitionBindingSources(mode);
  if(mode==17) return ContactTranslatedSources();
  if(mode>=14) return ContactDegenerateSources(mode);
  if(mode==8) return ContactRationalSources();
  if(mode==9) return ContactTrimmedSources();
  if(mode>=12) {
    NCollection_List<TopoDS_Shape> sources;
    sources.Append(mode==12?CensusEdge({{0,0},{4,0},{0,4},{0,0}})
                           :CensusEdge({{3,-9},{-1,13},{-1,-13},{3,9}}));
    return sources;
  }
  auto sources=ContactCurvedSources();
  if(mode==10) sources.First().Reverse();
  if(mode==11) sources.Reverse();
  return sources;
}
