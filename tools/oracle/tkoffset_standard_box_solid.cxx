#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <GProp_GProps.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <iomanip>
#include <iostream>

int main() {
  const TopoDS_Shape input = BRepPrimAPI_MakeBox(1.0, 2.0, 3.0).Shape();
  BRepOffset_MakeOffset maker;
  maker.Initialize(input, 0.5, 1.0e-7, BRepOffset_Skin, false, false, GeomAbs_Arc, false, false);
  maker.MakeOffsetShape();
  int faces = 0, edges = 0;
  for (TopExp_Explorer explore(maker.Shape(), TopAbs_FACE); explore.More(); explore.Next()) ++faces;
  for (TopExp_Explorer explore(maker.Shape(), TopAbs_EDGE); explore.More(); explore.Next()) ++edges;
  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> sourceFaces, sourceEdges, sourceVertices;
  TopExp::MapShapes(input, TopAbs_FACE, sourceFaces);
  TopExp::MapShapes(input, TopAbs_EDGE, sourceEdges);
  TopExp::MapShapes(input, TopAbs_VERTEX, sourceVertices);
  int faceImages = 0, edgeImages = 0, edgeGeneratedFaces = 0, vertexGeneratedFaces = 0;
  for (int index = 1; index <= sourceFaces.Extent(); ++index) {
    faceImages += maker.OffsetFacesFromShapes().HasImage(sourceFaces(index));
  }
  for (int index = 1; index <= sourceEdges.Extent(); ++index) {
    const TopoDS_Shape& edge = sourceEdges(index);
    edgeImages += maker.OffsetEdgesFromShapes().HasImage(edge);
    const auto& generated = maker.Generated(edge);
    edgeGeneratedFaces += generated.Extent() == 1 && generated.First().ShapeType() == TopAbs_FACE;
  }
  for (int index = 1; index <= sourceVertices.Extent(); ++index) {
    const auto& generated = maker.Generated(sourceVertices(index));
    vertexGeneratedFaces += generated.Extent() == 1 && generated.First().ShapeType() == TopAbs_FACE;
  }
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(maker.Shape(), properties);
  std::cout << static_cast<int>(maker.Error()) << " " << maker.IsDone() << " "
            << static_cast<int>(maker.Shape().ShapeType()) << " " << faces << " " << edges << " "
            << BRepCheck_Analyzer(maker.Shape(), true).IsValid() << " area " << std::fixed
            << std::setprecision(9) << properties.Mass() << std::defaultfloat
            << " face_images " << faceImages << " edge_images " << edgeImages
            << " generated_edge_faces " << edgeGeneratedFaces << " vertex_generated_faces "
            << vertexGeneratedFaces << "\n";
}
