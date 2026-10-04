#include <BRepBuilderAPI_MakeShape.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <BRepOffset_Analyse.hxx>
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
#include <TopExp.hxx>
#include <gp.hxx>
#include <algorithm>
#include <iomanip>
#include <iostream>
#include <vector>

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
  TopoDS_Edge freeEdge;
  for (TopExp_Explorer it(first, TopAbs_EDGE); it.More() && freeEdge.IsNull(); it.Next()) {
    const TopoDS_Edge candidate = TopoDS::Edge(it.Current());
    if (!candidate.IsSame(shared)) freeEdge = candidate;
  }

  const double offset = 0.5;
  const double tolerance = 1.0e-7;
  const double angle = 4.0 * std::asin(std::min(tolerance / (std::abs(offset * 0.5) + Precision::Confusion()), 1.0));
  BRepOffset_Analyse analysis;
  analysis.SetOffsetValue(offset);
  analysis.Perform(input, angle);
  const auto& intervals = analysis.Type(shared);

  BRepOffset_MakeOffset maker;
  maker.Initialize(input, offset, tolerance, BRepOffset_Skin, false, false, GeomAbs_Arc, false, false);
  maker.MakeOffsetShape();
  int faces = 0, edges = 0, cylinders = 0, planes = 0, shells = 0;
  std::vector<double> faceAreas;
  TopExp_Explorer explore(maker.Shape(), TopAbs_FACE);
  for (; explore.More(); explore.Next()) {
    ++faces;
    const auto type = BRep_Tool::Surface(TopoDS::Face(explore.Current()))->DynamicType()->Name();
    cylinders += std::string(type) == "Geom_CylindricalSurface";
    planes += std::string(type) == "Geom_Plane";
    GProp_GProps face_properties;
    BRepGProp::SurfaceProperties(explore.Current(), face_properties);
    faceAreas.push_back(face_properties.Mass());
  }
  for (explore.Init(maker.Shape(), TopAbs_EDGE); explore.More(); explore.Next()) ++edges;
  for (explore.Init(maker.Shape(), TopAbs_SHELL); explore.More(); explore.Next()) ++shells;
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(maker.Shape(), properties);
  const bool has_edge_image_before = maker.OffsetEdgesFromShapes().HasImage(shared);
  const auto& generated_edge = maker.Generated(shared);
  const int generated_edge_count = generated_edge.Extent();
  const int generated_edge_type =
      generated_edge.IsEmpty() ? -1 : static_cast<int>(generated_edge.First().ShapeType());
  const int free_edge_generated = maker.Generated(freeEdge).Extent();
  const bool free_edge_image = maker.OffsetEdgesFromShapes().HasImage(freeEdge);
  TopoDS_Vertex shared_start, shared_end;
  TopExp::Vertices(shared, shared_start, shared_end);
  const int shared_start_generated = maker.Generated(shared_start).Extent();
  const int shared_end_generated = maker.Generated(shared_end).Extent();
  std::cout << std::setprecision(17) << "status " << static_cast<int>(maker.Error()) << " done " << maker.IsDone()
            << " shape " << static_cast<int>(maker.Shape().ShapeType()) << " faces " << faces
            << " planes " << planes << " cylinders " << cylinders << " edges " << edges
            << " shells " << shells << " area " << properties.Mass() << " valid "
            << BRepCheck_Analyzer(maker.Shape(), true).IsValid() << " input_closed "
            << input.Closed() << " face_images " << maker.OffsetFacesFromShapes().HasImage(first)
            << " " << maker.OffsetFacesFromShapes().HasImage(second) << " generated_edge "
            << generated_edge_count << " edge_image " << has_edge_image_before << "/"
            << maker.OffsetEdgesFromShapes().HasImage(shared) << " edge_type "
            << static_cast<int>(shared.ShapeType()) << " generated_type " << generated_edge_type
            << " free_edge_image " << free_edge_image << " free_edge_generated " << free_edge_generated
            << " shared_vertices_generated " << shared_start_generated << " " << shared_end_generated
            << " interval_count " << intervals.Extent()
            << " interval_type " << (intervals.IsEmpty() ? -1 : static_cast<int>(intervals.First().Type())) << "\n";
  std::sort(faceAreas.begin(), faceAreas.end());
  std::cout << "face_areas";
  for (double area : faceAreas) std::cout << " " << area;
  std::cout << "\n";
}
