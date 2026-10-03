#include <BRep_Builder.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <GProp_GProps.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeFix_FixSmallSolid.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_Shape.hxx>

#include <iostream>
#include <limits>

static TopoDS_Shape box(double x, double y, double z, double width, double height, double depth) {
  return BRepPrimAPI_MakeBox(gp_Pnt(x, y, z), width, height, depth).Shape();
}

static TopoDS_Compound compound(const TopoDS_Shape& first, const TopoDS_Shape& second) {
  TopoDS_Compound result;
  BRep_Builder builder;
  builder.MakeCompound(result);
  builder.Add(result, first);
  builder.Add(result, second);
  return result;
}

static int count(const TopoDS_Shape& shape, TopAbs_ShapeEnum kind) {
  int result = 0;
  for (TopExp_Explorer iter(shape, kind); iter.More(); iter.Next()) ++result;
  return result;
}

static double volume(const TopoDS_Shape& shape) {
  GProp_GProps props;
  BRepGProp::VolumeProperties(shape, props);
  return props.Mass();
}

static void print(const char* name, const TopoDS_Shape& shape);

static TopoDS_Face face_at_x(const TopoDS_Shape& shape, double x) {
  for (TopExp_Explorer iter(shape, TopAbs_FACE); iter.More(); iter.Next()) {
    GProp_GProps props;
    BRepGProp::SurfaceProperties(iter.Current(), props);
    if (std::abs(props.CentreOfMass().X() - x) < 1e-10) return TopoDS::Face(iter.Current());
  }
  return TopoDS_Face();
}

static TopoDS_Shape first_shell(const TopoDS_Shape& shape) {
  for (TopoDS_Iterator iter(shape); iter.More(); iter.Next()) {
    if (iter.Value().ShapeType() == TopAbs_SHELL) return iter.Value();
  }
  return TopoDS_Shape();
}

static void merge_case() {
  const TopoDS_Shape large = box(0, 0, 0, 10, 10, 10);
  const TopoDS_Shape small = box(10, 0, 0, 1, 10, 10);
  const TopoDS_Face large_contact = face_at_x(large, 10.0);
  const TopoDS_Face small_contact = face_at_x(small, 10.0);
  Handle(ShapeBuild_ReShape) contact_context = new ShapeBuild_ReShape();
  contact_context->Replace(small_contact, large_contact.Reversed());
  const TopoDS_Shape shared_small = contact_context->Apply(small);
  ShapeFix_FixSmallSolid fixer;
  fixer.SetFixMode(2);
  fixer.SetVolumeThreshold(101.0);
  Handle(ShapeBuild_ReShape) merge_context = new ShapeBuild_ReShape();
  print("merge", fixer.Merge(compound(large, shared_small), merge_context));

  const TopoDS_Shape middle = box(10, 0, 0, 1, 10, 10);
  const TopoDS_Shape end = box(11, 0, 0, 1, 10, 10);
  Handle(ShapeBuild_ReShape) middle_context = new ShapeBuild_ReShape();
  middle_context->Replace(face_at_x(middle, 10.0), face_at_x(large, 10.0).Reversed());
  const TopoDS_Shape shared_middle = middle_context->Apply(middle);
  Handle(ShapeBuild_ReShape) end_context = new ShapeBuild_ReShape();
  end_context->Replace(face_at_x(end, 11.0), face_at_x(shared_middle, 11.0).Reversed());
  const TopoDS_Shape shared_end = end_context->Apply(end);
  Handle(ShapeBuild_ReShape) chain_context = new ShapeBuild_ReShape();
  print("merge_chain", fixer.Merge(compound(large, compound(shared_middle, shared_end)), chain_context));

  const TopoDS_Shape isolated = box(100, 0, 0, 1, 1, 1);
  Handle(ShapeBuild_ReShape) isolated_context = new ShapeBuild_ReShape();
  print("merge_isolated", fixer.Merge(compound(large, isolated), isolated_context));

  const TopoDS_Shape other = box(100, 0, 0, 1, 1, 1);
  TopoDS_Solid multi_shell;
  BRep_Builder builder;
  builder.MakeSolid(multi_shell);
  builder.Add(multi_shell, first_shell(shared_small));
  builder.Add(multi_shell, first_shell(other));
  Handle(ShapeBuild_ReShape) multi_context = new ShapeBuild_ReShape();
  ShapeFix_FixSmallSolid multi_fixer;
  multi_fixer.SetFixMode(2);
  multi_fixer.SetVolumeThreshold(102.0);
  const TopoDS_Shape multi_input = compound(large, multi_shell);
  const TopoDS_Shape multi_result = multi_fixer.Merge(multi_input, multi_context);
  print("merge_other_shell", multi_result);
  print("merge_shell_replacement", multi_context->Value(first_shell(large)));

}

static void print(const char* name, const TopoDS_Shape& shape) {
  std::cout << name << ' ' << count(shape, TopAbs_SOLID) << ' '
            << count(shape, TopAbs_SHELL) << ' ' << count(shape, TopAbs_FACE) << ' '
            << volume(shape) << '\n';
}

static void removal_cases() {
  const TopoDS_Shape small = box(0, 0, 0, 1, 1, 1);
  const TopoDS_Shape large = box(2, 0, 0, 10, 10, 10);
  ShapeFix_FixSmallSolid fixer;
  auto context = new ShapeBuild_ReShape();
  print("disabled", fixer.Remove(compound(small, large), context));
  fixer.SetFixMode(2);
  fixer.SetVolumeThreshold(1.0);
  context = new ShapeBuild_ReShape();
  print("volume", fixer.Remove(compound(small, large), context));
  fixer.SetFixMode(1);
  fixer.SetWidthFactorThreshold(1.0);
  context = new ShapeBuild_ReShape();
  print("width", fixer.Remove(compound(small, large), context));
  fixer.SetFixMode(99);
  fixer.SetVolumeThreshold(0.0);
  fixer.SetWidthFactorThreshold(1.0);
  context = new ShapeBuild_ReShape();
  print("invalid_mode", fixer.Remove(compound(small, large), context));
  fixer.SetFixMode(0);
  fixer.SetVolumeThreshold(1.1);
  fixer.SetWidthFactorThreshold(0.3);
  context = new ShapeBuild_ReShape();
  print("both_require_both", fixer.Remove(compound(small, large), context));
  fixer.SetVolumeThreshold(-1.0);
  fixer.SetWidthFactorThreshold(std::numeric_limits<double>::quiet_NaN());
  context = new ShapeBuild_ReShape();
  print("negative_nan_disable", fixer.Remove(compound(small, large), context));
}

int main() {
  removal_cases();
  merge_case();
  return 0;
}
