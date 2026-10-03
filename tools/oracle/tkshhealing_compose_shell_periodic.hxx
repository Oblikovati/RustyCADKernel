#pragma once

#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <NCollection_HArray2.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend_CompositeSurface.hxx>
#include <ShapeFix_ComposeShell.hxx>
#include <TopoDS_Iterator.hxx>
#include <gp_Ax3.hxx>
#include <gp_Cylinder.hxx>
#include <iostream>

static void printMappedEdges(
  const Handle(ShapeBuild_ReShape)& context,
  const TopoDS_Shape& wire);

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
