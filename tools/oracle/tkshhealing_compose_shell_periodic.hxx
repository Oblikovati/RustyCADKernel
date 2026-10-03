#pragma once

#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Tool.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom2d_Curve.hxx>
#include <NCollection_HArray2.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend_CompositeSurface.hxx>
#include <ShapeFix_ComposeShell.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopExp_Explorer.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopoDS.hxx>
#include <gp_Ax3.hxx>
#include <gp_Cylinder.hxx>
#include <iostream>
#include <cmath>

static void printMappedEdges(
  const Handle(ShapeBuild_ReShape)& context,
  const TopoDS_Shape& wire);
static void printFirstSeamPcurve(const TopoDS_Shape& result);
static void printPcurveBounds(const TopoDS_Shape& result);

static void runSplitEdgesPeriodicCylinder()
{
  const gp_Cylinder cylinder(gp_Ax3(), 2.0);
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(cylinder, 0., 2. * M_PI, 0., 2.).Face();
  Handle(NCollection_HArray2<Handle(Geom_Surface)>) patches =
    new NCollection_HArray2<Handle(Geom_Surface)>(1, 1, 1, 1);
  patches->SetValue(1, 1, new Geom_CylindricalSurface(cylinder));
  Handle(ShapeExtend_CompositeSurface) grid = new ShapeExtend_CompositeSurface(patches);
  ShapeFix_ComposeShell tool;
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  tool.SetContext(context);
  tool.Init(grid, TopLoc_Location(), face, 1.0e-7);
  tool.SplitEdges();
  std::cout << "split_edges_periodic";
  for (TopoDS_Iterator wireIt(face); wireIt.More(); wireIt.Next())
  {
    if (wireIt.Value().ShapeType() != TopAbs_WIRE)
      continue;
    printMappedEdges(context, wireIt.Value());
  }
  std::cout << '\n';
}

static void runPerformPeriodicCylinder(bool closedMode)
{
  const gp_Cylinder cylinder(gp_Ax3(), 2.0);
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(cylinder, 0., 2. * M_PI, 0., 2.).Face();
  Handle(NCollection_HArray2<Handle(Geom_Surface)>) patches =
    new NCollection_HArray2<Handle(Geom_Surface)>(1, 1, 1, 1);
  patches->SetValue(1, 1, new Geom_CylindricalSurface(cylinder));
  ShapeFix_ComposeShell tool;
  tool.SetContext(new ShapeBuild_ReShape);
  tool.Init(new ShapeExtend_CompositeSurface(patches), TopLoc_Location(), face, 1.0e-7);
  tool.ClosedMode() = closedMode;
  const bool done = tool.Perform();
  const TopoDS_Shape result = tool.Result();
  int children = 0;
  for (TopoDS_Iterator it(result); it.More(); it.Next())
    ++children;
  std::cout << (closedMode ? "perform_periodic_cylinder_closed " :
                             "perform_periodic_cylinder_open ")
            << done << ' ' << result.ShapeType() << ' ' << result.Orientation() << ' '
            << children << ' ' << tool.Status(ShapeExtend_DONE1);
  printFirstSeamPcurve(result);
  std::cout << '\n';
}

static void runPerformPeriodicCylinderPatches(bool closedMode)
{
  const gp_Cylinder cylinder(gp_Ax3(), 2.0);
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(cylinder, 0., 2. * M_PI, 0., 2.).Face();
  const Handle(Geom_Surface) basis = new Geom_CylindricalSurface(cylinder);
  Handle(NCollection_HArray2<Handle(Geom_Surface)>) patches =
    new NCollection_HArray2<Handle(Geom_Surface)>(1, 2, 1, 1);
  patches->SetValue(1, 1, new Geom_RectangularTrimmedSurface(basis, 0., M_PI, 0., 2., true, true));
  patches->SetValue(2, 1, new Geom_RectangularTrimmedSurface(basis, M_PI, 2. * M_PI, 0., 2., true, true));
  ShapeFix_ComposeShell tool;
  tool.SetContext(new ShapeBuild_ReShape);
  tool.Init(new ShapeExtend_CompositeSurface(patches), TopLoc_Location(), face, 1.0e-7);
  tool.ClosedMode() = closedMode;
  const bool done = tool.Perform();
  const TopoDS_Shape result = tool.Result();
  int children = 0;
  for (TopoDS_Iterator it(result); it.More(); it.Next())
    ++children;
  std::cout << (closedMode ? "perform_periodic_cylinder_patches_closed " :
                             "perform_periodic_cylinder_patches_open ")
            << done << ' ' << result.ShapeType() << ' '
            << result.Orientation() << ' ' << children << ' '
            << tool.Status(ShapeExtend_DONE1);
  printPcurveBounds(result);
  std::cout << '\n';
}

static void printFirstSeamPcurve(const TopoDS_Shape& result)
{
  const TopoDS_Face face = TopoDS::Face(result);
  for (TopExp_Explorer it(result, TopAbs_EDGE); it.More(); it.Next())
  {
    const TopoDS_Edge edge = TopoDS::Edge(it.Current());
    if (!BRep_Tool::IsClosed(edge, face))
      continue;
    double first1 = 0., last1 = 0., first2 = 0., last2 = 0.;
    const Handle(Geom2d_Curve) curve1 = BRep_Tool::CurveOnSurface(edge, face, first1, last1);
    const TopoDS_Edge reversed = TopoDS::Edge(edge.Reversed());
    const Handle(Geom2d_Curve) curve2 = BRep_Tool::CurveOnSurface(reversed, face, first2, last2);
    if (curve1.IsNull() || curve2.IsNull())
      continue;
    const gp_Pnt2d point1 = curve1->Value(first1);
    const gp_Pnt2d point2 = curve2->Value(first2);
    std::cout << ' ' << std::llround(point1.X() * 1.e6) << ' '
              << std::llround(point1.Y() * 1.e6) << ' '
              << std::llround(point2.X() * 1.e6) << ' '
              << std::llround(point2.Y() * 1.e6);
    return;
  }
  std::cout << " no_seam";
}

static void printMappedEdges(
  const Handle(ShapeBuild_ReShape)& context,
  const TopoDS_Shape& wire)
{
  for (TopoDS_Iterator it(wire); it.More(); it.Next())
  {
    TopoDS_Shape mapped = context->Apply(it.Value());
    std::cout << ' ' << mapped.ShapeType();
    if (mapped.ShapeType() != TopAbs_WIRE)
      continue;
    int count = 0;
    for (TopoDS_Iterator edgeIt(mapped); edgeIt.More(); edgeIt.Next())
      ++count;
    std::cout << ':' << count;
  }
}
