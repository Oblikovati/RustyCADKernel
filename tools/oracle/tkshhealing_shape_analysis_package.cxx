// Repository-owned oracle against OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <ShapeAnalysis.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Plane.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopExp.hxx>
#include <gp_Pln.hxx>
#include <gp.hxx>
#include <iostream>
#include <iomanip>
static void face_row(const char* name,const TopoDS_Face& face) {
  double uf,ul,vf,vl;
  try {
    ShapeAnalysis::GetFaceUVBounds(face,uf,ul,vf,vl);
    std::cout << name << " 1 " << uf << ' ' << ul << ' ' << vf << ' ' << vl << ' ' << ShapeAnalysis::IsOuterBound(face) << '\n';
  } catch(Standard_Failure const&) { std::cout << name << " 0\n"; }
}
static void endpoint_row(const char* name,const TopoDS_Shape& shape) {
  TopoDS_Vertex first,last;
  ShapeAnalysis::FindBounds(shape,first,last);
  std::cout << name << ' ' << !first.IsNull() << ' ' << !last.IsNull();
  if (!first.IsNull()) {
    gp_Pnt a=BRep_Tool::Pnt(first),b=BRep_Tool::Pnt(last);
    std::cout << ' ' << a.X() << ' ' << a.Y() << ' ' << a.Z() << ' ' << b.X() << ' ' << b.Y() << ' ' << b.Z();
  }
  std::cout << '\n';
}
static TopoDS_Wire polygon(bool closed) {
  BRepBuilderAPI_MakePolygon builder;
  builder.Add(gp_Pnt(0,0,0)); builder.Add(gp_Pnt(2,0,0));
  builder.Add(gp_Pnt(2,3,0)); builder.Add(gp_Pnt(0,3,0));
  if (closed) builder.Close();
  return builder.Wire();
}
int main() {
  std::cout << std::setprecision(17);
  BRep_Builder builder;
  TopoDS_Face natural;
  builder.MakeFace(natural,new Geom_Plane(gp::XOY()),1e-7);
  face_row("natural-plane",natural);
  TopoDS_Face bounded=BRepBuilderAPI_MakeFace(gp_Pln(gp::XOY()),0,2,0,3);
  face_row("rectangle",bounded);
  face_row("reversed-face",TopoDS::Face(bounded.Reversed()));
  TopoDS_Face inner; builder.MakeFace(inner,new Geom_Plane(gp::XOY()),1e-7);
  builder.Add(inner,ShapeAnalysis::OuterWire(bounded).Reversed());
  face_row("inner-only",inner);
  TopoDS_Wire hole=BRepBuilderAPI_MakePolygon(gp_Pnt(.5,.5,0),gp_Pnt(.5,1,0),gp_Pnt(1,1,0),gp_Pnt(1,.5,0),true);
  builder.Add(bounded,hole); face_row("with-hole",bounded);
  TopoDS_Face missing; builder.MakeFace(missing,new Geom_CylindricalSurface(gp::XOY(),1),1e-7);
  TopoDS_Edge empty_edge; builder.MakeEdge(empty_edge);
  TopoDS_Wire missing_wire; builder.MakeWire(missing_wire);
  builder.Add(missing_wire,empty_edge); builder.Add(missing,missing_wire);
  face_row("missing-pcurve",missing);
  for (bool closed : {false,true}) {
    TopoDS_Wire wire=polygon(closed);
    std::cout << (closed?"area-closed":"area-open") << ' ' << ShapeAnalysis::ContourArea(wire) << ' ' << ShapeAnalysis::ContourArea(TopoDS::Wire(wire.Reversed())) << '\n';
    endpoint_row(closed?"ends-closed":"ends-open",wire);
    endpoint_row(closed?"ends-closed-reversed":"ends-open-reversed",wire.Reversed());
  }
  TopoDS_Wire empty_wire; builder.MakeWire(empty_wire);
  std::cout << "area-empty " << ShapeAnalysis::ContourArea(empty_wire) << '\n';
  endpoint_row("ends-empty",empty_wire); endpoint_row("ends-face",natural);
  TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(gp_Pnt(1,2,3),gp_Pnt(4,5,6));
  endpoint_row("ends-edge",edge); endpoint_row("ends-edge-reversed",edge.Reversed());
  endpoint_row("ends-vertex",TopExp::FirstVertex(edge));
}
