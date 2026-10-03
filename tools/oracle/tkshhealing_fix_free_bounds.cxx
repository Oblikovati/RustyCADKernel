#include <ShapeFix_FreeBounds.hxx>
#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_Plane.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_FrozenShape.hxx>
#include <gp_Pln.hxx>
#include <iostream>
static TopoDS_Face open_face(double start, double end) {
  BRep_Builder builder; TopoDS_Face face; TopoDS_Wire wire;
  builder.MakeFace(face,new Geom_Plane(gp_Pln(gp::XOY())),1e-7);
  builder.MakeWire(wire); builder.Add(wire,BRepBuilderAPI_MakeEdge(gp_Pnt(start,0,0),gp_Pnt(end,0,0)).Edge());
  builder.Add(face,wire); return face;
}
static TopoDS_Compound faces() {
  BRep_Builder builder; TopoDS_Compound compound; builder.MakeCompound(compound);
  builder.Add(compound,open_face(0,1));builder.Add(compound,open_face(1.001,2));return compound;
}
static int count(const TopoDS_Shape& shape, TopAbs_ShapeEnum type) {
  int result=0; for(TopExp_Explorer exp(shape,type);exp.More();exp.Next()) ++result; return result;
}
static void row(const char* label,const ShapeFix_FreeBounds& fixer) {
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> vertices; TopExp::MapShapes(fixer.GetShape(),TopAbs_VERTEX,vertices);
  std::cout << label << ' ' << fixer.GetShape().IsNull() << ' ' << fixer.GetClosedWires().IsNull() << ' '
    << fixer.GetOpenWires().IsNull() << ' ' << count(fixer.GetClosedWires(),TopAbs_WIRE) << ' '
    << count(fixer.GetOpenWires(),TopAbs_WIRE) << ' ' << count(fixer.GetOpenWires(),TopAbs_EDGE) << ' '
    << vertices.Extent() << '\n';
}
int main() {
  row("default",ShapeFix_FreeBounds());
  row("actual",ShapeFix_FreeBounds(faces(),.01,false,false));
  row("forecast-no-connect",ShapeFix_FreeBounds(faces(),1e-7,0.,false,false));
  try { row("forecast-connect",ShapeFix_FreeBounds(faces(),1e-7,.01,false,false)); }
  catch (const TopoDS_FrozenShape&) { std::cout << "forecast-connect TopoDS_FrozenShape" << '\n'; }
  BRep_Builder builder; TopoDS_Compound empty;builder.MakeCompound(empty);
  row("empty-zero",ShapeFix_FreeBounds(empty,0.,false,false));
  row("empty-positive",ShapeFix_FreeBounds(empty,.01,false,false));
}
