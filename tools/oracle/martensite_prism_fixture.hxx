// SPDX-License-Identifier: GPL-2.0-only
// Shared pinned finite rectangular source; extraction preserves the original fixture body.
#pragma once
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <TopoDS_Face.hxx>
static TopoDS_Face Rectangle()
{
  BRepBuilderAPI_MakePolygon polygon;
  polygon.Add(gp_Pnt(0,0,0)); polygon.Add(gp_Pnt(4,0,0));
  polygon.Add(gp_Pnt(4,3,0)); polygon.Add(gp_Pnt(0,3,0));
  polygon.Close();
  return BRepBuilderAPI_MakeFace(polygon.Wire()).Face();
}
