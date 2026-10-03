#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom2d_Line.hxx>
#include <GeomLib.hxx>
#include <ShapeUpgrade_ShapeConvertToBezier.hxx>
#include <TopExp_Explorer.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <gp_Pln.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Ax3.hxx>
#include <gp_Ax2d.hxx>
#include <iostream>
#include <iomanip>
#include <Precision.hxx>

static void report_modes(const char* name, ShapeUpgrade_ShapeConvertToBezier& tool)
{
  std::cout << name << ' ' << tool.Get2dConversion() << ' ' << tool.Get3dConversion() << ' '
            << tool.GetSurfaceConversion() << ' ' << tool.Get3dLineConversion() << ' '
            << tool.Get3dCircleConversion() << ' ' << tool.Get3dConicConversion() << ' '
            << tool.GetPlaneMode() << ' ' << tool.GetRevolutionMode() << ' '
            << tool.GetExtrusionMode() << ' ' << tool.GetBSplineMode() << '\n';
}

static void report_shape(const char* name, ShapeUpgrade_ShapeConvertToBezier& tool, bool changed)
{
  const TopoDS_Shape result = tool.Result();
  int faces = 0, edges = 0, curves3d = 0, curves2d = 0, bezierSurfaces = 0;
  for (TopExp_Explorer faceIt(result, TopAbs_FACE); faceIt.More(); faceIt.Next())
  {
    ++faces;
    TopoDS_Face face = TopoDS::Face(faceIt.Current());
    if (BRep_Tool::Surface(face)->IsKind(STANDARD_TYPE(Geom_BezierSurface)))
      ++bezierSurfaces;
    for (TopExp_Explorer edgeIt(face, TopAbs_EDGE); edgeIt.More(); edgeIt.Next())
    {
      ++edges;
      TopoDS_Edge edge = TopoDS::Edge(edgeIt.Current());
      double first = 0.0, last = 0.0;
      occ::handle<Geom_Curve> curve = BRep_Tool::Curve(edge, first, last);
      curves3d += !curve.IsNull() && curve->IsKind(STANDARD_TYPE(Geom_BezierCurve));
      occ::handle<Geom2d_Curve> pcurve = BRep_Tool::CurveOnSurface(edge, face, first, last);
      curves2d += !pcurve.IsNull() && pcurve->IsKind(STANDARD_TYPE(Geom2d_BezierCurve));
    }
  }
  std::cout << name << " changed=" << changed << " faces=" << faces << " edges=" << edges
            << " curves3d=" << curves3d << " curves2d=" << curves2d
            << " bezierSurfaces=" << bezierSurfaces << '\n';
}

static void report_mixed_range_seam()
{
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Cylinder(gp_Ax3(), 2.0), 0.0, 6.283185307179586, 0.0, 5.0);
  BRep_Builder builder;
  for (TopExp_Explorer edgeIt(face, TopAbs_EDGE); edgeIt.More(); edgeIt.Next())
  {
    TopoDS_Edge edge = TopoDS::Edge(edgeIt.Current());
    edge.Orientation(TopAbs_FORWARD);
    if (!BRep_Tool::IsClosed(edge, face)) continue;
    double first = 0.0, last = 0.0;
    occ::handle<Geom2d_Curve> current = BRep_Tool::CurveOnSurface(edge, face, first, last);
    const gp_Pnt2d p1 = current->Value(first), p2 = current->Value(last);
    NCollection_Array1<gp_Pnt2d> poles(1, 2);
    poles(1) = p1;
    poles(2) = p2;
    occ::handle<Geom2d_Curve> bezier = new Geom2d_BezierCurve(poles);
    occ::handle<Geom2d_Curve> reverse;
    TopoDS_Edge reversed = TopoDS::Edge(edge.Reversed());
    double reverseFirst = 0.0, reverseLast = 0.0;
    reverse = BRep_Tool::CurveOnSurface(reversed, face, reverseFirst, reverseLast);
    builder.UpdateEdge(edge, bezier, reverse, face, 0.0);
    builder.Range(edge, face, 0.2, 0.8);
    break;
  }
  ShapeUpgrade_ShapeConvertToBezier converter(face);
  converter.SetSurfaceConversion(true);
  const bool changed = converter.Perform();
  const TopoDS_Face result = TopoDS::Face(converter.Result());
  for (TopExp_Explorer edgeIt(result, TopAbs_EDGE); edgeIt.More(); edgeIt.Next())
  {
    TopoDS_Edge edge = TopoDS::Edge(edgeIt.Current());
    edge.Orientation(TopAbs_FORWARD);
    if (!BRep_Tool::IsClosed(edge, result)) continue;
    double first = 0.0, last = 0.0;
    occ::handle<Geom2d_Curve> current = BRep_Tool::CurveOnSurface(edge, result, first, last);
    TopoDS_Edge reversed = TopoDS::Edge(edge.Reversed());
    double reverseFirst = 0.0, reverseLast = 0.0;
    occ::handle<Geom2d_Curve> reverse = BRep_Tool::CurveOnSurface(reversed, result, reverseFirst, reverseLast);
    const gp_Pnt2d p1 = current->Value(first), p2 = current->Value(last);
    const gp_Pnt2d r1 = reverse->Value(reverseFirst), r2 = reverse->Value(reverseLast);
    std::cout << std::fixed << std::setprecision(5)
              << "mixed_range_seam changed=" << changed << " currentBezier=" << current->IsKind(STANDARD_TYPE(Geom2d_BezierCurve))
              << " reverseBezier=" << reverse->IsKind(STANDARD_TYPE(Geom2d_BezierCurve))
              << " range=" << first << ',' << last << " reverseRange=" << reverseFirst << ',' << reverseLast
              << " currentEnds=" << p1.X() << ',' << p1.Y() << ';' << p2.X() << ',' << p2.Y()
              << " reverseEnds=" << r1.X() << ',' << r1.Y() << ';' << r2.X() << ',' << r2.Y() << '\n';
    break;
  }
}

int main()
{
  ShapeUpgrade_ShapeConvertToBezier modes;
  report_modes("defaults", modes);
  modes.Set2dConversion(true);
  modes.Set3dConversion(true);
  modes.SetSurfaceConversion(true);
  modes.Set3dLineConversion(false);
  modes.Set3dCircleConversion(false);
  modes.Set3dConicConversion(false);
  modes.SetPlaneMode(false);
  modes.SetRevolutionMode(false);
  modes.SetExtrusionMode(false);
  modes.SetBSplineMode(false);
  report_modes("configured", modes);

  TopoDS_Face plane = BRepBuilderAPI_MakeFace(gp_Pln(), 0.0, 1.0, 0.0, 1.0);
  ShapeUpgrade_ShapeConvertToBezier unchanged(plane);
  report_shape("unchanged", unchanged, unchanged.Perform());

  ShapeUpgrade_ShapeConvertToBezier surfaceOnly(plane);
  surfaceOnly.SetSurfaceConversion(true);
  report_shape("surface_only", surfaceOnly, surfaceOnly.Perform());

  ShapeUpgrade_ShapeConvertToBezier curves3d(plane);
  curves3d.Set3dConversion(true);
  report_shape("curves3d", curves3d, curves3d.Perform());

  ShapeUpgrade_ShapeConvertToBezier curves2d(plane);
  curves2d.Set2dConversion(true);
  report_shape("curves2d", curves2d, curves2d.Perform());

  ShapeUpgrade_ShapeConvertToBezier all(plane);
  all.SetSurfaceConversion(true);
  all.Set2dConversion(true);
  all.Set3dConversion(true);
  report_shape("all", all, all.Perform());

  TopoDS_Face cylinder =
    BRepBuilderAPI_MakeFace(gp_Cylinder(gp_Ax3(), 2.0), 0.0, 6.283185307179586, 0.0, 1.0);
  ShapeUpgrade_ShapeConvertToBezier cylinderCurves(cylinder);
  cylinderCurves.Set3dConversion(true);
  report_shape("cylinder_curves", cylinderCurves, cylinderCurves.Perform());
  ShapeUpgrade_ShapeConvertToBezier cylinderPCurves(cylinder);
  cylinderPCurves.Set2dConversion(true);
  report_shape("cylinder_pcurves", cylinderPCurves, cylinderPCurves.Perform());
  ShapeUpgrade_ShapeConvertToBezier lineOnly(cylinder);
  lineOnly.Set3dConversion(true);
  lineOnly.Set3dCircleConversion(false);
  report_shape("cylinder_line_only", lineOnly, lineOnly.Perform());

  occ::handle<Geom2d_Curve> reverseSource = new Geom2d_Line(gp_Ax2d());
  occ::handle<Geom2d_Curve> normalizedReverse;
  GeomLib::SameRange(Precision::PConfusion(), reverseSource, 2.0, 5.0, 0.0, 1.0, normalizedReverse);
  const gp_Pnt2d reverseStart = normalizedReverse->Value(0.0);
  const gp_Pnt2d reverseEnd = normalizedReverse->Value(1.0);
  std::cout << "reverse_line_unit_range " << reverseStart.X() << ' ' << reverseStart.Y() << ' '
            << reverseEnd.X() << ' ' << reverseEnd.Y() << '\n';
  report_mixed_range_seam();
}
