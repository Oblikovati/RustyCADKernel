#include <ShapeUpgrade_FixSmallCurves.hxx>
class ShSmallCurveProbe : public ShapeUpgrade_FixSmallCurves {
public:
  using ShapeUpgrade_FixSmallCurves::GetSplitCurve2dTool;
  using ShapeUpgrade_FixSmallCurves::GetSplitCurve3dTool;
  void SetRawStatus(int value) { myStatus = value; }
  const TopoDS_Edge &Edge() const { return myEdge; }
  const TopoDS_Face &Face() const { return myFace; }
};
struct ShSmallOutputs {
  occ::handle<Geom_Curve> spatial;
  occ::handle<Geom2d_Curve> planar, reversed;
  double first, last;
};
ShSmallOutputs sh_small_outputs(const TpShapes &shapes, GaReader &reader) {
  int spatial = reader.integer(), planar = reader.integer(),
      reversed = reader.integer();
  return {spatial < 0 ? occ::handle<Geom_Curve>{} : shapes.curves.at(spatial),
          planar < 0 ? occ::handle<Geom2d_Curve>{} : shapes.pcurves.at(planar),
          reversed < 0 ? occ::handle<Geom2d_Curve>{}
                       : shapes.pcurves.at(reversed),
          reader.real(), reader.real()};
}
void sh_small_state(const ShSmallCurveProbe &tool, const TpShapes &shapes,
                    const ShSmallOutputs &outputs) {
  ga_int(tp_same_index(shapes, tool.Edge()));
  ga_int(tp_same_index(shapes, tool.Face()));
  ga_flag(tool.GetSplitCurve3dTool().IsNull());
  ga_flag(tool.GetSplitCurve2dTool().IsNull());
  sh_flags(tool);
  ga_real(outputs.first);
  ga_real(outputs.last);
  shc_curve(outputs.spatial);
  shc_curve2d(outputs.planar);
  shc_curve2d(outputs.reversed);
}
void sh_small_tools(ShSmallCurveProbe &tool, GaReader &reader) {
  occ::handle<ShapeUpgrade_SplitCurve3d> spatial;
  occ::handle<ShapeUpgrade_SplitCurve2d> planar;
  if (reader.flag())
    spatial = new ShapeUpgrade_SplitCurve3d;
  if (reader.flag())
    planar = new ShapeUpgrade_SplitCurve2d;
  tool.SetSplitCurve3dTool(spatial);
  tool.SetSplitCurve2dTool(planar);
  ga_flag(tool.GetSplitCurve3dTool() == spatial);
  ga_flag(tool.GetSplitCurve2dTool() == planar);
}
void sh_small_operation(ShSmallCurveProbe &tool, const TpShapes &shapes,
                        ShSmallOutputs &outputs, GaReader &reader) {
  int op = reader.integer();
  if (op == 0) {
    auto edge = sh_edge(shapes, reader.integer());
    auto face = sh_face(shapes, reader.integer());
    tool.Init(edge, face);
  } else if (op == 1)
    sh_small_tools(tool, reader);
  else if (op == 2)
    ga_flag(tool.Approx(outputs.spatial, outputs.planar, outputs.reversed,
                        outputs.first, outputs.last));
  else
    tool.SetRawStatus(reader.integer());
}
void sh_fix_small_curves_case(std::istream &input) {
  GaReader reader{input};
  TpShapes shapes = tp_shapes(reader);
  ShSmallCurveProbe tool;
  auto outputs = sh_small_outputs(shapes, reader);
  auto originals = outputs;
  sh_small_state(tool, shapes, outputs);
  int count = reader.integer();
  for (int i = 0; i < count; i++) {
    GA_TRY(sh_small_operation(tool, shapes, outputs, reader))
    ga_flag(outputs.spatial == originals.spatial);
    ga_flag(outputs.planar == originals.planar);
    ga_flag(outputs.reversed == originals.reversed);
    sh_small_state(tool, shapes, outputs);
  }
  ga_end();
}
