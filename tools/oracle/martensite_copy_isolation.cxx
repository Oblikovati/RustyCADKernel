// Pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; local oracle only.
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Line.hxx>
#include <Poly_Polygon3D.hxx>
#include <Standard_DimensionMismatch.hxx>
#include <Standard_NullObject.hxx>
#include <TopoDS.hxx>
#include <gp_Lin.hxx>
#include <iostream>

static TopoDS_Edge source_edge()
{
  BRepBuilderAPI_MakeEdge maker(gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0));
  return maker.Edge();
}

static void reuse_after_null()
{
  TopoDS_Edge source = source_edge();
  BRepBuilderAPI_Copy copier(source, true, true);
  TopoDS_Edge old = TopoDS::Edge(copier.Shape());
  try { copier.Perform(TopoDS_Shape(), true, true); }
  catch (const Standard_NullObject&) { std::cout << "null " << copier.IsDone() << '\n'; }
  double first, last;
  auto curve = occ::down_cast<Geom_Line>(BRep_Tool::Curve(source, first, last));
  curve->SetLocation(gp_Pnt(9, 0, 0));
  copier.Perform(source, true, true);
  auto result = occ::down_cast<Geom_Line>(BRep_Tool::Curve(TopoDS::Edge(copier.Shape()), first, last));
  std::cout << "reuse " << copier.IsDone() << ' ' << old.IsSame(copier.Shape()) << ' ' << result->Lin().Location().X() << '\n';
}

static TopoDS_Edge mismatched_mesh()
{
  TopoDS_Edge failing = source_edge();
  NCollection_Array1<gp_Pnt> nodes(1, 2);
  nodes.SetValue(1, gp_Pnt(0, 0, 0));
  nodes.SetValue(2, gp_Pnt(2, 0, 0));
  NCollection_Array1<double> parameters(1, 3);
  parameters.Init(0.0);
  occ::handle<Poly_Polygon3D> polygon = new Poly_Polygon3D(nodes, parameters);
  BRep_Builder builder;
  builder.UpdateEdge(failing, polygon);
  return failing;
}

static void recover_after_mesh_failure()
{
  TopoDS_Edge source = source_edge();
  BRepBuilderAPI_Copy copier(source, true, true);
  TopoDS_Edge old = TopoDS::Edge(copier.Shape());
  try { copier.Perform(mismatched_mesh(), true, true); }
  catch (const Standard_DimensionMismatch&) { std::cout << "mesh-failure " << copier.IsDone() << ' ' << !old.IsNull() << '\n'; }
  copier.Perform(source, true, true);
  std::cout << "recover " << copier.IsDone() << ' ' << old.IsSame(copier.Shape()) << '\n';
}

int main()
{
  reuse_after_null();
  recover_after_mesh_failure();
}
