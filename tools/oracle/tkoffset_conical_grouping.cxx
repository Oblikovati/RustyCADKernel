#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Geom_ConicalSurface.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <gp_Ax3.hxx>
#include <gp.hxx>
#include <iomanip>
#include <iostream>

static void DumpShape(const char* label, const TopoDS_Shape& shape) {
  int faces = 0;
  int edges = 0;
  int degenerated = 0;
  for (TopExp_Explorer explorer(shape, TopAbs_FACE); explorer.More(); explorer.Next()) {
    ++faces;
  }
  for (TopExp_Explorer explorer(shape, TopAbs_EDGE); explorer.More(); explorer.Next()) {
    ++edges;
    if (BRep_Tool::Degenerated(TopoDS::Edge(explorer.Current()))) {
      ++degenerated;
    }
  }
  GProp_GProps props;
  BRepGProp::SurfaceProperties(shape, props);
  std::cout << label << " " << faces << " " << edges << " " << degenerated << " "
            << BRepCheck_Analyzer(shape, true).IsValid() << " " << std::setprecision(17)
            << props.Mass() << "\n";
  int index = 0;
  for (TopExp_Explorer explorer(shape, TopAbs_FACE); explorer.More(); explorer.Next()) {
    const TopoDS_Face face = TopoDS::Face(explorer.Current());
    const auto surface = BRep_Tool::Surface(face);
    int faceEdges = 0;
    int faceDegenerated = 0;
    for (TopExp_Explorer edgeExplorer(face, TopAbs_EDGE); edgeExplorer.More(); edgeExplorer.Next()) {
      ++faceEdges;
      if (BRep_Tool::Degenerated(TopoDS::Edge(edgeExplorer.Current()))) {
        ++faceDegenerated;
      }
    }
    std::cout << "face " << ++index << " " << surface->DynamicType()->Name() << " "
              << faceEdges << " " << faceDegenerated << "\n";
    int edgeIndex = 0;
    for (TopExp_Explorer edgeExplorer(face, TopAbs_EDGE); edgeExplorer.More(); edgeExplorer.Next()) {
      const TopoDS_Edge edge = TopoDS::Edge(edgeExplorer.Current());
      TopoDS_Vertex first, last;
      TopExp::Vertices(edge, first, last, true);
      const gp_Pnt firstPoint = BRep_Tool::Pnt(first);
      const gp_Pnt lastPoint = BRep_Tool::Pnt(last);
      std::cout << "edge " << edgeIndex++ << " orientation " << edge.Orientation()
                << " closed " << BRep_Tool::IsClosed(edge, face)
                << " deg " << BRep_Tool::Degenerated(edge)
                << " first " << firstPoint.X() << " " << firstPoint.Y() << " " << firstPoint.Z()
                << " last " << lastPoint.X() << " " << lastPoint.Y() << " " << lastPoint.Z() << "\n";
    }
  }
}

int main() {
  const gp_Ax3 axis(gp::Origin(), gp::DZ(), gp::DX());
  const Handle(Geom_ConicalSurface) cone = new Geom_ConicalSurface(axis, 0.5, 0.0);
  const TopoDS_Face first = BRepBuilderAPI_MakeFace(cone, 0.0, M_PI, 0.0, 10.0, 1.e-7);
  const TopoDS_Face second = BRepBuilderAPI_MakeFace(cone, M_PI, 2.0 * M_PI, 0.0, 10.0, 1.e-7);
  BRepBuilderAPI_Sewing sewing(1.e-6, true, true, true, false);
  sewing.Add(first);
  sewing.Add(second);
  sewing.Perform();
  const TopoDS_Shape source = sewing.SewedShape();
  DumpShape("input", source);
  BRepOffsetAPI_MakeOffsetShape maker;
  maker.PerformByJoin(source, 1.0, 1.e-7, BRepOffset_Skin, false, false, GeomAbs_Arc, false);
  std::cout << "done " << maker.IsDone() << "\n";
  if (maker.IsDone()) {
    DumpShape("result", maker.Shape());
  }
}
