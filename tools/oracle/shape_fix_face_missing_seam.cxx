#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <GeomConvert.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <ShapeFix_Face.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pnt.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt2d.hxx>
#include <iostream>

static void addCylinderBoundary(BRep_Builder& builder, TopoDS_Face& face, double z,
                                bool reverse);
static void addTorusOuterBoundary(BRep_Builder& builder, TopoDS_Face& face);
static void addSphereBoundary(BRep_Builder& builder, TopoDS_Face& face);

int main() {
  BRep_Builder builder;
  const Handle(Geom_Plane) plane = new Geom_Plane(gp_Ax3());
  TopoDS_Face planarFace;
  builder.MakeFace(planarFace, plane, 1.e-7);
  ShapeFix_Face planarFixer(planarFace);
  std::cout << "plane_open changed=" << planarFixer.FixMissingSeam() << '\n';

  const Handle(Geom_SphericalSurface) surface =
      new Geom_SphericalSurface(gp_Ax3(), 1.0);
  TopoDS_Face face;
  builder.MakeFace(face, surface, 1.e-7);
  TopoDS_Edge boundary = BRepBuilderAPI_MakeEdge(new Geom_Circle(gp_Ax2(), 1.0));
  builder.Range(boundary, 0.0, 2.0 * M_PI, true);
  builder.UpdateEdge(boundary,
                     new Geom2d_Line(gp_Pnt2d(0.0, 0.0), gp_Dir2d(1.0, 0.0)),
                     face, 1.e-7);
  builder.Range(boundary, face, 0.0, 2.0 * M_PI);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, boundary);
  builder.Add(face, wire);

  ShapeFix_Face fixer(face);
  const bool changed = fixer.FixMissingSeam();
  const TopoDS_Shape result = fixer.Result();
  int faces = 0;
  int degeneratedEdges = 0;
  for (TopExp_Explorer faceIt(result, TopAbs_FACE); faceIt.More(); faceIt.Next()) {
    ++faces;
    for (TopExp_Explorer edgeIt(faceIt.Current(), TopAbs_EDGE); edgeIt.More(); edgeIt.Next()) {
      if (BRep_Tool::Degenerated(TopoDS::Edge(edgeIt.Current()))) ++degeneratedEdges;
    }
  }
  std::cout << "sphere_open_u changed=" << changed << " faces=" << faces
            << " degenerated_edges=" << degeneratedEdges << '\n';

  const Handle(Geom_CylindricalSurface) cylinder =
      new Geom_CylindricalSurface(gp_Ax3(), 1.0);
  TopoDS_Face cylinderFace;
  builder.MakeFace(cylinderFace, cylinder, 1.e-7);
  addCylinderBoundary(builder, cylinderFace, -1.0, false);
  addCylinderBoundary(builder, cylinderFace, 1.0, true);
  ShapeFix_Face cylinderFixer(cylinderFace);
  const bool cylinderChanged = cylinderFixer.FixMissingSeam();
  const TopoDS_Shape cylinderResult = cylinderFixer.Result();
  int cylinderFaces = 0;
  int cylinderEdges = 0;
  int cylinderSeamEdges = 0;
  for (TopExp_Explorer faceIt(cylinderResult, TopAbs_FACE); faceIt.More(); faceIt.Next()) {
    ++cylinderFaces;
    for (TopExp_Explorer edgeIt(faceIt.Current(), TopAbs_EDGE); edgeIt.More(); edgeIt.Next()) {
      ++cylinderEdges;
      if (BRep_Tool::IsClosed(TopoDS::Edge(edgeIt.Current()), TopoDS::Face(faceIt.Current()))) {
        ++cylinderSeamEdges;
      }
    }
  }
  std::cout << "cylinder_two_open_u changed=" << cylinderChanged << " faces=" << cylinderFaces
            << " edges=" << cylinderEdges << " seam_edges=" << cylinderSeamEdges << '\n';

  const Handle(Geom_ToroidalSurface) spindle =
      new Geom_ToroidalSurface(gp_Ax3(), 1.0, 2.0);
  TopoDS_Face spindleFace;
  builder.MakeFace(spindleFace, spindle, 1.e-7);
  addTorusOuterBoundary(builder, spindleFace);
  ShapeFix_Face spindleFixer(spindleFace);
  const bool spindleChanged = spindleFixer.FixMissingSeam();
  const TopoDS_Shape spindleResult = spindleFixer.Result();
  int spindleFaces = 0;
  int spindleEdges = 0;
  int spindleDegeneratedEdges = 0;
  for (TopExp_Explorer faceIt(spindleResult, TopAbs_FACE); faceIt.More(); faceIt.Next()) {
    ++spindleFaces;
    for (TopExp_Explorer edgeIt(faceIt.Current(), TopAbs_EDGE); edgeIt.More(); edgeIt.Next()) {
      ++spindleEdges;
      if (BRep_Tool::Degenerated(TopoDS::Edge(edgeIt.Current()))) ++spindleDegeneratedEdges;
    }
  }
  std::cout << "spindle_torus_open_u changed=" << spindleChanged << " faces=" << spindleFaces
            << " edges=" << spindleEdges << " degenerated_edges=" << spindleDegeneratedEdges
            << '\n';

  const Handle(Geom_BSplineSurface) splineSphere =
      GeomConvert::SurfaceToBSplineSurface(surface);
  TopoDS_Face splineFace;
  builder.MakeFace(splineFace, splineSphere, 1.e-7);
  addSphereBoundary(builder, splineFace);
  ShapeFix_Face splineFixer(splineFace);
  const bool splineChanged = splineFixer.FixMissingSeam();
  const TopoDS_Shape splineResult = splineFixer.Result();
  int splineFaces = 0;
  int splineEdges = 0;
  int splineDegeneratedEdges = 0;
  for (TopExp_Explorer faceIt(splineResult, TopAbs_FACE); faceIt.More(); faceIt.Next()) {
    ++splineFaces;
    for (TopExp_Explorer edgeIt(faceIt.Current(), TopAbs_EDGE); edgeIt.More(); edgeIt.Next()) {
      ++splineEdges;
      if (BRep_Tool::Degenerated(TopoDS::Edge(edgeIt.Current()))) ++splineDegeneratedEdges;
    }
  }
  std::cout << "periodic_bspline_sphere_open_u changed=" << splineChanged << " faces="
            << splineFaces << " edges=" << splineEdges
            << " degenerated_edges=" << splineDegeneratedEdges << '\n';
}

static void addCylinderBoundary(BRep_Builder& builder, TopoDS_Face& face, double z,
                                bool reverse) {
  const Handle(Geom_Circle) circle =
      new Geom_Circle(gp_Ax2(gp_Pnt(0.0, 0.0, z), gp_Dir(0.0, 0.0, 1.0)), 1.0);
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(circle);
  builder.Range(edge, 0.0, 2.0 * M_PI, true);
  const double sign = reverse ? -1.0 : 1.0;
  builder.UpdateEdge(edge,
                     new Geom2d_Line(gp_Pnt2d(0.0, z), gp_Dir2d(sign, 0.0)),
                     face, 1.e-7);
  builder.Range(edge, face, 0.0, 2.0 * M_PI);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, edge);
  builder.Add(face, wire);
}

static void addTorusOuterBoundary(BRep_Builder& builder, TopoDS_Face& face) {
  const Handle(Geom_Circle) circle =
      new Geom_Circle(gp_Ax2(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), 3.0);
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(circle);
  builder.Range(edge, 0.0, 2.0 * M_PI, true);
  builder.UpdateEdge(edge,
                     new Geom2d_Line(gp_Pnt2d(0.0, 0.0), gp_Dir2d(1.0, 0.0)),
                     face, 1.e-7);
  builder.Range(edge, face, 0.0, 2.0 * M_PI);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, edge);
  builder.Add(face, wire);
}

static void addSphereBoundary(BRep_Builder& builder, TopoDS_Face& face) {
  const Handle(Geom_Circle) circle =
      new Geom_Circle(gp_Ax2(), 1.0);
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(circle);
  builder.Range(edge, 0.0, 2.0 * M_PI, true);
  builder.UpdateEdge(edge,
                     new Geom2d_Line(gp_Pnt2d(0.0, 0.0), gp_Dir2d(1.0, 0.0)),
                     face, 1.e-7);
  builder.Range(edge, face, 0.0, 2.0 * M_PI);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, edge);
  builder.Add(face, wire);
}
