// SPDX-License-Identifier: GPL-2.0-only
#pragma once
#include "../fixture_io.hpp"
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepClass_FaceClassifier.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <Geom_BezierCurve.hxx>
#include <Standard_Version.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pln.hxx>
#include <BRep_Tool.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <fstream>
#include <iostream>
#include <stdexcept>
occ::handle<Geom_BezierCurve> curve3d(const std::vector<std::string>& fields) {
  const auto points=split(fields.at(0),';');
  NCollection_Array1<gp_Pnt> poles(1,static_cast<int>(points.size()));
  for (int index=1;index<=poles.Upper();++index) {
    const auto point=decode(points.at(index-1));
    poles.SetValue(index,gp_Pnt(point.at(0),point.at(1),0));
  }
  if (fields.at(1)=="-") return new Geom_BezierCurve(poles);
  const auto values=decode(fields.at(1));
  NCollection_Array1<double> weights(1,static_cast<int>(values.size()));
  for (int index=1;index<=weights.Upper();++index) weights.SetValue(index,values.at(index-1));
  return new Geom_BezierCurve(poles,weights);
}
TopoDS_Edge edge3d(const std::string& encoded) {
  const auto fields=split(encoded,'/');
  const auto interval=decode(fields.at(2));
  BRepBuilderAPI_MakeEdge builder(curve3d(fields),interval.at(0),interval.at(1));
  if (!builder.IsDone()) throw std::runtime_error(encoded+"; expected completed native edge");
  TopoDS_Edge edge=builder.Edge();
  if (fields.at(3)=="1") edge.Reverse();
  return edge;
}
TopoDS_Face face3d(const std::string& geometry) {
  BRepBuilderAPI_MakeWire wire;
  for (const auto& encoded:split(geometry,'|')) wire.Add(edge3d(encoded));
  if (!wire.IsDone()) throw std::runtime_error(geometry+"; expected completed native wire");
  BRepBuilderAPI_MakeFace face(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1)),wire.Wire(),true);
  if (!face.IsDone() || !BRepCheck_Analyzer(face.Face()).IsValid())
    throw std::runtime_error(geometry+"; expected a valid native planar face");
  return face.Face();
}
bool exact_vertex(const TopoDS_Face& face,const gp_Pnt& seed) {
  for (TopExp_Explorer it(face,TopAbs_VERTEX);it.More();it.Next()) {
    const auto point=BRep_Tool::Pnt(TopoDS::Vertex(it.Current()));
    if (point.X()==seed.X() && point.Y()==seed.Y() && point.Z()==seed.Z()) return true;
  }
  return false;
}
