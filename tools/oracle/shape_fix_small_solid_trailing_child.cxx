#include <BRep_Builder.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <GProp_GProps.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeFix_FixSmallSolid.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopExp_Explorer.hxx>

static TopoDS_Face face_at_x(const TopoDS_Shape& shape, double x) {
  for (TopExp_Explorer iter(shape, TopAbs_FACE); iter.More(); iter.Next()) {
    GProp_GProps props;
    BRepGProp::SurfaceProperties(iter.Current(), props);
    if (std::abs(props.CentreOfMass().X() - x) < 1e-10) return TopoDS::Face(iter.Current());
  }
  return TopoDS_Face();
}

int main() {
  BRep_Builder builder;
  TopoDS_Solid small;
  builder.MakeSolid(small);
  const TopoDS_Shape large = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), 10, 10, 10).Shape();
  const TopoDS_Shape small_box = BRepPrimAPI_MakeBox(gp_Pnt(10, 0, 0), 1, 10, 10).Shape();
  Handle(ShapeBuild_ReShape) contact_context = new ShapeBuild_ReShape();
  contact_context->Replace(face_at_x(small_box, 10), face_at_x(large, 10).Reversed());
  const TopoDS_Shape shared_small = contact_context->Apply(small_box);
  TopoDS_Shell shell;
  TopExp_Explorer shell_iter(shared_small, TopAbs_SHELL);
  shell = TopoDS::Shell(shell_iter.Current());
  builder.Add(small, shell);
  TopoDS_Vertex trailing;
  builder.MakeVertex(trailing, gp_Pnt(0, 0, 0), 0.1);
  builder.Add(small, trailing);
  TopoDS_Compound input;
  builder.MakeCompound(input);
  builder.Add(input, large);
  builder.Add(input, small);
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape();
  ShapeFix_FixSmallSolid fixer;
  fixer.SetFixMode(2);
  fixer.SetVolumeThreshold(102.0);
  (void)fixer.Merge(input, context);
}
