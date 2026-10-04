#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepOffset_Analyse.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Vertex.hxx>
#include <algorithm>
#include <iomanip>
#include <iostream>

int main() {
  const TopoDS_Shape box = BRepPrimAPI_MakeBox(4.0, 5.0, 6.0).Shape();
  NCollection_IndexedDataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher> ancestors;
  TopExp::MapShapesAndAncestors(box, TopAbs_EDGE, TopAbs_FACE, ancestors);
  TopoDS_Face first, second;
  TopoDS_Edge shared;
  for (int i = 1; i <= ancestors.Extent() && first.IsNull(); ++i) {
    const auto& faces = ancestors.FindFromIndex(i);
    if (faces.Extent() != 2) continue;
    first = TopoDS::Face(faces.First());
    second = TopoDS::Face(faces.Last());
    shared = TopoDS::Edge(ancestors.FindKey(i));
  }
  TopoDS_Shell input;
  BRep_Builder builder;
  builder.MakeShell(input);
  builder.Add(input, first);
  builder.Add(input, second);

  const double offset = -0.5;
  const double tolerance = 1.0e-7;
  const double angle = 4.0 * std::asin(std::min(tolerance / (std::abs(offset * 0.5) + Precision::Confusion()), 1.0));
  BRepOffset_Analyse analysis;
  analysis.SetOffsetValue(offset);
  analysis.Perform(input, angle);
  const auto& intervals = analysis.Type(shared);

  BRepOffset_MakeOffset maker;
  maker.Initialize(input, offset, tolerance, BRepOffset_Skin, false, false, GeomAbs_Arc, false, false);
  maker.MakeOffsetShape();
  int faces = 0, edges = 0, shells = 0, planes = 0;
  TopExp_Explorer explore(maker.Shape(), TopAbs_FACE);
  for (; explore.More(); explore.Next()) {
    ++faces;
    planes += std::string(BRep_Tool::Surface(TopoDS::Face(explore.Current()))->DynamicType()->Name()) == "Geom_Plane";
  }
  for (explore.Init(maker.Shape(), TopAbs_EDGE); explore.More(); explore.Next()) ++edges;
  for (explore.Init(maker.Shape(), TopAbs_SHELL); explore.More(); explore.Next()) ++shells;
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(maker.Shape(), properties);
  const auto& generated = maker.Generated(shared);
  std::cout << std::setprecision(17) << "status " << static_cast<int>(maker.Error()) << " done " << maker.IsDone()
            << " shape " << static_cast<int>(maker.Shape().ShapeType()) << " faces " << faces
            << " planes " << planes << " edges " << edges << " shells " << shells << " area "
            << properties.Mass() << " valid " << BRepCheck_Analyzer(maker.Shape(), true).IsValid()
            << " face_images " << maker.OffsetFacesFromShapes().HasImage(first) << " "
            << maker.OffsetFacesFromShapes().HasImage(second) << " generated_edge " << generated.Extent()
            << " interval_count " << intervals.Extent() << " interval_type "
            << (intervals.IsEmpty() ? -1 : static_cast<int>(intervals.First().Type())) << "\n";
}
