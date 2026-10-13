// SPDX-License-Identifier: GPL-2.0-only
// Independent OCCT 8.0.0 diagnostic for RustyCADKernel#10.
#include <BOPAlgo_Builder.hxx>
#include <BOPAlgo_BuilderFace.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepLib.hxx>
#include <BRep_TEdge.hxx>
#include <BRepGProp.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <Geom_BezierCurve.hxx>
#include <GProp_GProps.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <NCollection_IndexedMap.hxx>
#include <gp_Pln.hxx>
#include <iostream>
#include <iomanip>
#include <stdexcept>
using ShapeMap = NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher>;
using ShapeList = NCollection_List<TopoDS_Shape>;
class NonemptyProbe : public BOPAlgo_Builder {
  void CheckData() override { CheckFiller(); }
};
int count(const TopoDS_Shape& shape, TopAbs_ShapeEnum kind) {
  ShapeMap found;
  TopExp::MapShapes(shape, kind, found);
  return found.Extent();
}
void metrics(const char* label, const TopoDS_Shape& shape) {
  GProp_GProps props;
  BRepGProp::SurfaceProperties(shape, props, false, false);
  const auto center = props.CentreOfMass();
  std::cout << label << ',' << props.Mass() << ',' << center.X() << ','
            << center.Y() << ',' << center.Z() << ',' << count(shape, TopAbs_WIRE)
            << ',' << count(shape, TopAbs_EDGE) << ','
            << int(BRepAdaptor_Surface(TopoDS::Face(shape)).GetType()) << ','
            << BRepCheck_Analyzer(shape, true, false, true).IsValid() << '\n';
}
TopoDS_Edge source() {
  NCollection_Array1<gp_Pnt> poles(1,4);
  poles(1)=gp_Pnt(0,0,0); poles(2)=gp_Pnt(4,0,0);
  poles(3)=gp_Pnt(0,4,0); poles(4)=gp_Pnt(0,0,0);
  return BRepBuilderAPI_MakeEdge(new Geom_BezierCurve(poles)).Edge();
}
ShapeList regions(const TopoDS_Shape& split) {
  const auto plane=BRepBuilderAPI_MakeFace(gp_Pln()).Face();
  ShapeMap edges;
  TopExp::MapShapes(split, TopAbs_EDGE, edges);
  if(edges.Extent()!=1) throw std::runtime_error("split: expected one edge");
  const auto segment=TopoDS::Edge(edges(1));
  BRepLib::BuildPCurveForEdgeOnPlane(segment,plane);
  ShapeList oriented;
  oriented.Append(segment.Oriented(TopAbs_FORWARD));
  oriented.Append(segment.Oriented(TopAbs_REVERSED));
  BOPAlgo_BuilderFace builder;
  builder.SetFace(plane); builder.SetShapes(oriented); builder.Perform();
  if(builder.HasErrors()) throw std::runtime_error("regions: expected complete builder");
  return builder.Areas();
}
void prism_roles(const TopoDS_Shape& region, const TopoDS_Shape& solid,
                 BRepPrimAPI_MakePrism& maker) {
  ShapeMap edges,faces,roles;
  TopExp::MapShapes(region,TopAbs_EDGE,edges);
  const auto& generated=maker.Generated(edges(1));
  if(generated.Size()!=1) throw std::runtime_error("generated: expected one side face");
  roles.Add(generated.First()); roles.Add(maker.FirstShape()); roles.Add(maker.LastShape());
  TopExp::MapShapes(solid,TopAbs_FACE,faces);
  if(roles.Extent()!=faces.Extent()) throw std::runtime_error("history: expected complete face roles");
  for(int index=1; index<=faces.Extent(); ++index) {
    if(!roles.Contains(faces(index))) throw std::runtime_error("history: expected named face role");
  }
  for(int index=1; index<=roles.Extent(); ++index) metrics("face",roles(index));
}
void prism(const TopoDS_Shape& region) {
  BRepPrimAPI_MakePrism maker(region, gp_Vec(0,0,2), true, true);
  const auto solid=maker.Shape();
  GProp_GProps volume;
  BRepGProp::VolumeProperties(solid, volume, true, false, false);
  std::cout << "solid," << volume.Mass() << ',' << count(solid, TopAbs_FACE)
            << ',' << BRepCheck_Analyzer(solid,true,false,true).IsValid() << '\n';
  prism_roles(region,solid,maker);
}
void copied_input() {
  const auto original=source();
  BRepBuilderAPI_Copy copier(original,true,false);
  const auto copied=copier.Shape();
  const auto areas=regions(copied);
  const auto sourceState=occ::down_cast<BRep_TEdge>(original.TShape());
  const auto copyState=occ::down_cast<BRep_TEdge>(copied.TShape());
  std::cout << "copied-input-curves," << sourceState->Curves().Size() << ','
            << copyState->Curves().Size() << '\n';
  double first,last;
  const auto sourceCurve=occ::down_cast<Geom_BezierCurve>(BRep_Tool::Curve(original,first,last));
  const auto copyCurve=occ::down_cast<Geom_BezierCurve>(BRep_Tool::Curve(TopoDS::Edge(copied),first,last));
  sourceCurve->SetPole(2,gp_Pnt(8,0,0));
  std::cout << "copied-source-pole," << copyCurve->Pole(2).X() << ','
            << sourceCurve->Pole(2).X() << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  copied_input();
  NonemptyProbe builder;
  const auto original = source();
  ShapeList arguments; arguments.Append(original);
  builder.SetArguments(arguments); builder.SetNonDestructive(true);
  builder.SetToFillHistory(true); builder.Perform();
  if(builder.HasErrors()) throw std::runtime_error("fuse: expected complete builder");
  const auto edgeState = occ::down_cast<BRep_TEdge>(original.TShape());
  const auto before = edgeState->Curves().Size();
  const auto areas=regions(builder.Shape());
  std::cout << "input-curves," << before << ',' << edgeState->Curves().Size() << '\n';
  std::cout << "regions," << areas.Size() << '\n';
  for(const auto& region: areas) {
    metrics("region", region);
    GProp_GProps props; BRepGProp::SurfaceProperties(region, props, false, false);
    // Fixture-only selection: this cubic has positive analytic signed area 12/5.
    if(props.Mass()>0) prism(region);
  }
}
