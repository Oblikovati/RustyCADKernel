// ReassignPCurve oracle for OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <BRepLib_MakeEdge.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Plane.hxx>
#include <ShapeBuild_Edge.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>

TopoDS_Face plane(double z)
{
  TopoDS_Face face;
  BRep_Builder().MakeFace(face, new Geom_Plane(gp_Pln(gp_Pnt(0, 0, z), gp_Dir(0, 0, 1))), 1.e-7);
  return face;
}

void setPCurve(const TopoDS_Edge& edge, const TopoDS_Face& face,
               const occ::handle<Geom2d_Curve>& curve)
{
  BRep_Builder builder;
  builder.UpdateEdge(edge, curve, face, 1.e-7);
  builder.Range(edge, face, 0., 1.);
}

void writeCurve(const occ::handle<Geom2d_Curve>& curve, double first, double last)
{
  if (curve.IsNull())
  {
    std::cout << " 0 0 0 0 0 0";
    return;
  }
  gp_Pnt2d p1 = curve->Value(first), p2 = curve->Value(last);
  std::cout << ' ' << first << ' ' << last << ' ' << p1.X() << ' ' << p1.Y()
            << ' ' << p2.X() << ' ' << p2.Y();
}

void record(const char* name, bool changed, const TopoDS_Edge& edge,
            const TopoDS_Face& oldFace, const TopoDS_Face& newFace)
{
  double oldFirst = 0., oldLast = 0., newFirst = 0., newLast = 0.;
  bool oldStored = false, newStored = false;
  auto oldCurve = BRep_Tool::CurveOnSurface(edge, oldFace, oldFirst, oldLast, &oldStored);
  auto newCurve = BRep_Tool::CurveOnSurface(edge, newFace, newFirst, newLast, &newStored);
  std::cout << name << ' ' << changed << ' ' << oldStored << ' ' << newStored
            << ' ' << BRep_Tool::IsClosed(edge, oldFace)
            << ' ' << BRep_Tool::IsClosed(edge, newFace);
  writeCurve(oldCurve, oldFirst, oldLast);
  writeCurve(newCurve, newFirst, newLast);
  std::cout << '\n';
}

void seamCase(ShapeBuild_Edge& builder, const char* name, bool reversed)
{
  TopoDS_Edge edge;
  BRep_Builder().MakeEdge(edge);
  TopoDS_Face oldFace, newFace;
  auto oldSurface = new Geom_CylindricalSurface(gp_Ax3(), 1.);
  auto newSurface = new Geom_CylindricalSurface(gp_Ax3(), 2.);
  BRep_Builder().MakeFace(oldFace, oldSurface, 1.e-7);
  BRep_Builder().MakeFace(newFace, newSurface, 1.e-7);
  BRep_Builder().UpdateEdge(edge,
      new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(0, 1)),
      new Geom2d_Line(gp_Pnt2d(6.283185307179586, 0), gp_Dir2d(0, 1)),
      oldFace, 1.e-7);
  BRep_Builder().Range(edge, oldFace, 0., 1.);
  setPCurve(edge, newFace, new Geom2d_Line(gp_Pnt2d(.3, 0), gp_Dir2d(0, 1)));
  if (reversed) edge.Reverse();
  record(name, builder.ReassignPCurve(edge, oldFace, newFace), edge, oldFace, newFace);
}

int main()
{
  std::cout << std::setprecision(17);
  ShapeBuild_Edge builder;
  auto line = new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(1, 0));
  {
    TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0)).Edge();
    TopoDS_Face oldFace = plane(0), newFace = plane(1);
    setPCurve(edge, oldFace, line);
    record("ordinary", builder.ReassignPCurve(edge, oldFace, newFace), edge, oldFace, newFace);
  }
  {
    TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0)).Edge();
    TopoDS_Face oldFace = plane(0), newFace = plane(1);
    record("plane-generated", builder.ReassignPCurve(edge, oldFace, newFace), edge, oldFace, newFace);
  }
  {
    TopoDS_Edge edge;
    BRep_Builder().MakeEdge(edge);
    TopoDS_Face oldFace, newFace;
    BRep_Builder().MakeFace(oldFace, new Geom_CylindricalSurface(gp_Ax3(), 1.), 1.e-7);
    newFace = plane(0);
    record("missing-source", builder.ReassignPCurve(edge, oldFace, newFace), edge, oldFace, newFace);
  }
  seamCase(builder, "seam-into-seam", false);
  seamCase(builder, "seam-into-seam-reversed", true);
}
