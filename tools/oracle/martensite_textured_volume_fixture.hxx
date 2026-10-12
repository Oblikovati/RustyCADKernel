// Actual BRep topology preserves the native compound/solid grouping and cumulative location rules.
#include "martensite_textured_face_fixture.hxx"
#include <BRepTools.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <TopoDS_CompSolid.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_Vertex.hxx>
static TopoDS_Shell native_textured_volume_shell(bool mesh) {
  BRep_Builder builder; TopoDS_Shell shell; builder.MakeShell(shell);
  auto face = native_textured_face(1);
  if (!mesh) BRepTools::Clean(face);
  builder.Add(shell,face); shell.Closed(true);
  return shell;
}
static TopoDS_Solid native_textured_volume_solid(bool mesh) {
  BRep_Builder builder; TopoDS_Solid solid; builder.MakeSolid(solid);
  builder.Add(solid,native_textured_volume_shell(mesh));
  return solid;
}
static TopoDS_CompSolid native_textured_comp_solid() {
  BRep_Builder builder; TopoDS_CompSolid solid; builder.MakeCompSolid(solid);
  builder.Add(solid,native_textured_volume_solid(true));
  builder.Add(solid,native_textured_volume_solid(false));
  return solid;
}
static TopoDS_Compound native_textured_volume_compound() {
  BRep_Builder builder; TopoDS_Compound compound; builder.MakeCompound(compound);
  builder.Add(compound,native_textured_comp_solid());
  TopoDS_Vertex vertex; builder.MakeVertex(vertex,gp_Pnt(1,2,3),1e-7); builder.Add(compound,vertex);
  gp_Trsf transform; transform.SetTranslation(gp_Vec(.5,1.5,2.5));
  return TopoDS::Compound(compound.Reversed().Located(TopLoc_Location(transform),false));
}
static TopoDS_Shape native_textured_box_volume(bool mesh, bool closed) {
  auto solid = BRepPrimAPI_MakeBox(2.,3.,4.).Solid();
  if (mesh) BRepMesh_IncrementalMesh(solid,.1,false,.5,false);
  if (closed) solid.Closed(true);
  return solid;
}
static TopoDS_Shape native_textured_volume_shape(int scenario) {
  switch (scenario) {
    case 0: return TopoDS_Shape();
    case 1: return native_textured_face(1);
    case 2: return native_textured_volume_shell(true);
    case 3: return native_textured_volume_solid(true);
    case 4: return native_textured_volume_solid(false);
    case 5: return native_textured_comp_solid();
    case 6: return native_textured_volume_compound();
    case 7: return native_textured_box_volume(true,false);
    case 8: return native_textured_box_volume(false,false);
    default: return native_textured_box_volume(true,true);
  }
}
