#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepTools.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <BRepOffset_Offset.hxx>
#include <BRep_Tool.hxx>
#include <Geom2dAPI_Interpolate.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <GeomAPI.hxx>
#include <GeomConvert.hxx>
#include <GeomFill_BSplineCurves.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_Surface.hxx>
#include <Geom_BezierCurve.hxx>
#include <GProp_GProps.hxx>
#include <Precision.hxx>
#include <ShapeFix_Shape.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS.hxx>
#include <iostream>
#include <iomanip>

namespace {
TopoDS_Shape occ28131Face() {
  const double height = 8.5;
  const gp_Pnt v0(-17.6, 0.0, 0.0);
  const gp_Pnt v1(0.0, 32.8, 0.0);
  NCollection_Array1<gp_Pnt> poles(1, 4);
  poles(1) = v0;
  poles(4) = v1;
  poles(2) = gp_Pnt(v0.X(), (5.4 / 13.2) * v1.Y(), 0.0);
  poles(3) = gp_Pnt((6.0 / 6.8) * v0.X(), v1.Y(), 0.0);
  occ::handle<Geom_BSplineCurve> outline =
      GeomConvert::CurveToBSplineCurve(new Geom_BezierCurve(poles));

  occ::handle<NCollection_HArray1<gp_Pnt2d>> side1Points =
      new NCollection_HArray1<gp_Pnt2d>(1, 2);
  side1Points->SetValue(1, gp_Pnt2d(-v1.Y(), 0.0));
  side1Points->SetValue(2, gp_Pnt2d(0.0, height + height / 2.0));
  Geom2dAPI_Interpolate side1Interpolation(side1Points, false, 1.0e-6);
  side1Interpolation.Load(gp_Vec2d(0.0, 1.0), gp_Vec2d(1.0, 0.0));
  side1Interpolation.Perform();
  const gp_Pln side1Plane{
      gp_Ax3(gp_Pnt(), gp_Dir(gp_Dir::D::X), gp_Dir(gp_Dir::D::NY))};
  occ::handle<Geom_BSplineCurve> side1 = occ::down_cast<Geom_BSplineCurve>(
      GeomAPI::To3d(side1Interpolation.Curve(), side1Plane));

  occ::handle<NCollection_HArray1<gp_Pnt2d>> side2Points =
      new NCollection_HArray1<gp_Pnt2d>(1, 3);
  side2Points->SetValue(1, gp_Pnt2d(-v0.X(), 0.0));
  side2Points->SetValue(2, gp_Pnt2d(-v0.X() - 2.6, height));
  side2Points->SetValue(3, gp_Pnt2d(0.0, height + height / 2.0));
  Geom2dAPI_Interpolate side2Interpolation(side2Points, false, 1.0e-6);
  side2Interpolation.Perform();
  const gp_Pln side2Plane{
      gp_Ax3(gp_Pnt(), gp_Dir(gp_Dir::D::NY), gp_Dir(gp_Dir::D::NX))};
  occ::handle<Geom_BSplineCurve> side2 = occ::down_cast<Geom_BSplineCurve>(
      GeomAPI::To3d(side2Interpolation.Curve(), side2Plane));

  GeomFill_BSplineCurves fill(outline, side1, side2, GeomFill_CoonsStyle);
  BRepBuilderAPI_MakeFace face(fill.Surface(), 0.0);
  return face.IsDone() ? face.Shape() : TopoDS_Shape();
}

double maxTolerance(const TopoDS_Shape& shape) {
  double result = 0.0;
  for (TopAbs_ShapeEnum kind : {TopAbs_VERTEX, TopAbs_EDGE, TopAbs_FACE}) {
    for (TopExp_Explorer it(shape, kind); it.More(); it.Next()) {
      const double tolerance = kind == TopAbs_VERTEX
          ? BRep_Tool::Tolerance(TopoDS::Vertex(it.Current()))
          : kind == TopAbs_EDGE
              ? BRep_Tool::Tolerance(TopoDS::Edge(it.Current()))
              : BRep_Tool::Tolerance(TopoDS::Face(it.Current()));
      result = std::max(result, tolerance);
    }
  }
  return result;
}

double area(const TopoDS_Shape& shape) {
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(shape, properties);
  return properties.Mass();
}

void dump(const char* label, const TopoDS_Shape& shape) {
  int faces = 0, edges = 0, vertices = 0;
  for (TopExp_Explorer it(shape, TopAbs_FACE); it.More(); it.Next()) ++faces;
  for (TopExp_Explorer it(shape, TopAbs_EDGE); it.More(); it.Next()) ++edges;
  for (TopExp_Explorer it(shape, TopAbs_VERTEX); it.More(); it.Next()) ++vertices;
  std::cout << label << ' ' << int(shape.ShapeType()) << ' ' << faces << ' ' << edges << ' '
            << vertices << ' ' << BRepCheck_Analyzer(shape, true).IsValid() << ' '
            << maxTolerance(shape) << ' ' << area(shape) << '\n';
  for (TopExp_Explorer it(shape, TopAbs_FACE); it.More(); it.Next()) {
    TopLoc_Location location;
    occ::handle<Geom_Surface> surface = BRep_Tool::Surface(TopoDS::Face(it.Current()), location);
    double u1, u2, v1, v2;
    BRepTools::UVBounds(TopoDS::Face(it.Current()), u1, u2, v1, v2);
    int faceEdges = 0, faceVertices = 0;
    for (TopExp_Explorer edge(it.Current(), TopAbs_EDGE); edge.More(); edge.Next()) ++faceEdges;
    for (TopExp_Explorer vertex(it.Current(), TopAbs_VERTEX); vertex.More(); vertex.Next()) ++faceVertices;
    std::cout << "face-surface " << surface->DynamicType()->Name() << ' ' << faceEdges << ' '
              << faceVertices << ' ' << u1 << ' ' << u2 << ' ' << v1 << ' ' << v2 << ' '
              << area(it.Current()) << '\n';
  }
}
} // namespace

int main() {
  std::cout << std::setprecision(17);
  const TopoDS_Shape face = occ28131Face();
  for (TopExp_Explorer edge(face, TopAbs_EDGE); edge.More(); edge.Next()) {
    const TopoDS_Edge sourceEdge = TopoDS::Edge(edge.Current());
    std::cout << "input-edge " << BRep_Tool::Degenerated(sourceEdge) << ' '
              << BRep_Tool::IsClosed(sourceEdge, TopoDS::Face(face)) << '\n';
  }
  BRepOffset_Offset offsetFace(TopoDS::Face(face), 10.0, true, GeomAbs_Arc);
  std::cout << "offset-face-status " << offsetFace.Status() << '\n';
  dump("offset-face", offsetFace.Face());
  BRepOffset_MakeOffset maker;
  maker.Initialize(face, 10.0, Precision::Confusion(), BRepOffset_Skin, false, false, GeomAbs_Arc);
  maker.MakeOffsetShape();
  dump("raw", maker.Shape());
  occ::handle<ShapeFix_Shape> fixer = new ShapeFix_Shape(maker.Shape());
  fixer->Perform();
  dump("fixed", fixer->Shape());
}
