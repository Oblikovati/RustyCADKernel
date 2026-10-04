#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Builder.hxx>
#include <GProp_GProps.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shell.hxx>
#include <BRepAdaptor_Surface.hxx>

#include <algorithm>
#include <iomanip>
#include <iostream>
#include <vector>

int main() {
  const TopoDS_Shape cylinder = BRepPrimAPI_MakeCylinder(3.0, 6.0).Shape();
  TopoDS_Shell input;
  BRep_Builder builder;
  builder.MakeShell(input);
  TopExp_Explorer faces(cylinder, TopAbs_FACE);
  int sourceFaces = 0;
  for (; faces.More(); faces.Next()) {
    builder.Add(input, faces.Current());
    ++sourceFaces;
  }

  BRepOffset_MakeOffset maker;
  maker.Initialize(input, 0.5, 1.0e-7, BRepOffset_Skin, false, false, GeomAbs_Arc, false, false);
  maker.MakeOffsetShape();
  int resultFaces = 0, resultEdges = 0, resultShells = 0;
  std::vector<int> surfaceTypes;
  std::vector<double> faceAreas;
  for (TopExp_Explorer explore(maker.Shape(), TopAbs_FACE); explore.More(); explore.Next()) {
    ++resultFaces;
    const TopoDS_Face face = TopoDS::Face(explore.Current());
    surfaceTypes.push_back(static_cast<int>(BRepAdaptor_Surface(face, false).GetType()));
    GProp_GProps faceProperties;
    BRepGProp::SurfaceProperties(face, faceProperties);
    faceAreas.push_back(faceProperties.Mass());
  }
  for (TopExp_Explorer explore(maker.Shape(), TopAbs_EDGE); explore.More(); explore.Next()) ++resultEdges;
  for (TopExp_Explorer explore(maker.Shape(), TopAbs_SHELL); explore.More(); explore.Next()) ++resultShells;
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(maker.Shape(), properties);
  std::sort(surfaceTypes.begin(), surfaceTypes.end());
  std::sort(faceAreas.begin(), faceAreas.end());
  NCollection_IndexedDataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher> ancestors;
  TopExp::MapShapesAndAncestors(input, TopAbs_EDGE, TopAbs_FACE, ancestors);
  std::vector<int> generatedCounts;
  int offsetEdgeImages = 0;
  for (int index = 1; index <= ancestors.Extent(); ++index) {
    const auto& parents = ancestors.FindFromIndex(index);
    if (parents.Extent() != 2 || parents.First().IsSame(parents.Last())) continue;
    const TopoDS_Shape& edge = ancestors.FindKey(index);
    offsetEdgeImages += maker.OffsetEdgesFromShapes().HasImage(edge);
    generatedCounts.push_back(maker.Generated(edge).Extent());
  }

  std::cout << std::setprecision(17) << "source_faces " << sourceFaces << " status "
            << static_cast<int>(maker.Error()) << " done " << maker.IsDone() << " shape "
            << static_cast<int>(maker.Shape().ShapeType()) << " faces " << resultFaces << " edges "
            << resultEdges << " shells " << resultShells << " area " << properties.Mass() << " valid "
            << BRepCheck_Analyzer(maker.Shape(), true).IsValid() << " connectors "
            << generatedCounts.size() << " offset_edge_images " << offsetEdgeImages << " generated_counts";
  for (int count : generatedCounts) std::cout << " " << count;
  std::cout << " surfaces";
  for (int type : surfaceTypes) std::cout << " " << type;
  std::cout << " face_areas";
  for (double area : faceAreas) std::cout << " " << area;
  std::cout << "\n";
}
