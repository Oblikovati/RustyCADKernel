#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepClass3d_SolidClassifier.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <GProp_GProps.hxx>
#include <ShapeUpgrade_ShellSewing.hxx>
TopoDS_Shape sh_sewing_rectangle(double first, double last) {
  BRepBuilderAPI_MakePolygon polygon;
  for (const auto &point : {gp_Pnt(first, 0, 0), gp_Pnt(last, 0, 0),
                            gp_Pnt(last, 1, 0), gp_Pnt(first, 1, 0)})
    polygon.Add(point);
  polygon.Close();
  return BRepBuilderAPI_MakeFace(polygon.Wire()).Face();
}
TopoDS_Shape sh_sewing_shell(const std::vector<TopoDS_Shape> &faces) {
  BRep_Builder builder;
  TopoDS_Shell shell;
  builder.MakeShell(shell);
  for (const auto &face : faces)
    builder.Add(shell, face);
  return shell;
}
TopoDS_Shape sh_sewing_compound(const std::vector<TopoDS_Shape> &shapes) {
  BRep_Builder builder;
  TopoDS_Compound result;
  builder.MakeCompound(result);
  for (const auto &shape : shapes)
    builder.Add(result, shape);
  return result;
}
TopoDS_Shape sh_sewing_open(double gap) {
  return sh_sewing_shell(
      {sh_sewing_rectangle(0, 1), sh_sewing_rectangle(1 + gap, 2)});
}
TopoDS_Shape sh_sewing_group(int kind, double gap, double offset,
                             double scale) {
  if (kind == 5)
    return sh_sewing_compound(
        {BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), 2, 3, 4).Shape(),
         BRepPrimAPI_MakeBox(gp_Pnt(offset, 0, 0), scale, 2 * scale, 3 * scale)
             .Shape()});
  if (kind == 6)
    return sh_sewing_compound(
        {sh_sewing_open(gap), sh_sewing_rectangle(offset, offset + scale)});
  if (kind == 7)
    return sh_sewing_shell({sh_sewing_rectangle(offset, offset + scale)});
  return sh_sewing_compound(
      {sh_sewing_rectangle(0, 1), sh_sewing_rectangle(offset, offset + scale)});
}
TopoDS_Shape sh_sewing_geometry(int kind, double gap, double offset,
                                double scale) {
  if (kind == 0)
    return TopoDS_Shape();
  if (kind == 1)
    return sh_sewing_rectangle(offset, offset + scale);
  if (kind == 2)
    return sh_sewing_shell({});
  if (kind == 3)
    return sh_sewing_open(gap);
  if (kind == 4)
    return BRepPrimAPI_MakeBox(gp_Pnt(offset, 0, 0), 2 * scale, 3 * scale,
                               4 * scale)
        .Shape();
  return sh_sewing_group(kind, gap, offset, scale);
}
TopoDS_Shape sh_sewing_input(GaReader &reader) {
  int kind = reader.integer();
  double gap = reader.real(), offset = reader.real(), scale = reader.real();
  bool reverse = reader.flag();
  auto shape = sh_sewing_geometry(kind, gap, offset, scale);
  return reverse ? shape.Reversed() : shape;
}
void sh_sewing_result(const TopoDS_Shape &result, const TopoDS_Shape &input) {
  ga_flag(result.IsSame(input));
  tp_shape(result);
  if (result.IsNull())
    return;
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(result, properties);
  ga_real(properties.Mass());
  ga_flag(BRepCheck_Analyzer(result).IsValid());
  for (TopExp_Explorer exp(result, TopAbs_SOLID); exp.More(); exp.Next()) {
    BRepClass3d_SolidClassifier classifier(TopoDS::Solid(exp.Current()));
    classifier.PerformInfinitePoint(1e-6);
    ga_int(classifier.State());
  }
}
void sh_shell_sewing_case(std::istream &input) {
  GaReader reader{input};
  ShapeUpgrade_ShellSewing sewer;
  TopoDS_Shape result;
  int count = reader.integer();
  for (int index = 0; index < count; ++index) {
    auto shape = sh_sewing_input(reader);
    double tolerance = reader.real();
    bool previous = reader.flag();
    if (previous)
      shape = result;
    GA_TRY(result = sewer.ApplySewing(shape, tolerance))
    GA_TRY(sh_sewing_result(result, shape))
  }
  ga_end();
}
