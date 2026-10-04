#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepOffset_Analyse.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <Precision.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shell.hxx>
#include <BRepAdaptor_Surface.hxx>

#include <algorithm>
#include <cmath>
#include <iomanip>
#include <iostream>
#include <vector>

static GeomAbs_SurfaceType surfaceType(const TopoDS_Face& face) {
  return BRepAdaptor_Surface(face, false).GetType();
}

int main() {
  const TopoDS_Shape cylinder = BRepPrimAPI_MakeCylinder(3.0, 6.0).Shape();
  NCollection_IndexedDataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher> ancestors;
  TopExp::MapShapesAndAncestors(cylinder, TopAbs_EDGE, TopAbs_FACE, ancestors);
  TopoDS_Face lateral, cap;
  TopoDS_Edge shared;
  for (int i = 1; i <= ancestors.Extent() && shared.IsNull(); ++i) {
    const auto& faces = ancestors.FindFromIndex(i);
    if (faces.Extent() != 2) continue;
    const TopoDS_Face first = TopoDS::Face(faces.First());
    const TopoDS_Face second = TopoDS::Face(faces.Last());
    const auto firstType = surfaceType(first);
    const auto secondType = surfaceType(second);
    if (firstType == GeomAbs_Cylinder && secondType == GeomAbs_Plane) {
      lateral = first;
      cap = second;
    } else if (firstType == GeomAbs_Plane && secondType == GeomAbs_Cylinder) {
      lateral = second;
      cap = first;
    } else {
      continue;
    }
    shared = TopoDS::Edge(ancestors.FindKey(i));
  }
  TopoDS_Shell input;
  BRep_Builder builder;
  builder.MakeShell(input);
  builder.Add(input, lateral);
  builder.Add(input, cap);

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
  int faces = 0, edges = 0, shells = 0;
  std::vector<double> faceAreas;
  std::vector<int> surfaceTypes;
  TopExp_Explorer explore(maker.Shape(), TopAbs_FACE);
  for (; explore.More(); explore.Next()) {
    ++faces;
    const TopoDS_Face face = TopoDS::Face(explore.Current());
    surfaceTypes.push_back(static_cast<int>(surfaceType(face)));
    GProp_GProps faceProperties;
    BRepGProp::SurfaceProperties(face, faceProperties);
    faceAreas.push_back(faceProperties.Mass());
  }
  for (explore.Init(maker.Shape(), TopAbs_EDGE); explore.More(); explore.Next()) ++edges;
  for (explore.Init(maker.Shape(), TopAbs_SHELL); explore.More(); explore.Next()) ++shells;
  std::sort(faceAreas.begin(), faceAreas.end());
  std::sort(surfaceTypes.begin(), surfaceTypes.end());
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(maker.Shape(), properties);

  const bool edgeImage = maker.OffsetEdgesFromShapes().HasImage(shared);
  const int generatedCount = maker.Generated(shared).Extent();
  std::cout << std::setprecision(17) << "status " << static_cast<int>(maker.Error()) << " done " << maker.IsDone()
            << " shape " << static_cast<int>(maker.Shape().ShapeType()) << " faces " << faces
            << " edges " << edges << " shells " << shells << " area " << properties.Mass()
            << " valid " << BRepCheck_Analyzer(maker.Shape(), true).IsValid()
            << " face_images " << maker.OffsetFacesFromShapes().HasImage(lateral) << " "
            << maker.OffsetFacesFromShapes().HasImage(cap) << " edge_image " << edgeImage
            << " generated_count " << generatedCount << " interval_type "
            << (intervals.IsEmpty() ? -1 : static_cast<int>(intervals.First().Type())) << " surfaces";
  for (int type : surfaceTypes) std::cout << " " << type;
  std::cout << " face_areas";
  for (double area : faceAreas) std::cout << " " << area;
  std::cout << "\n";
}
