#include <ShapeUpgrade_EdgeDivide.hxx>
template <class Base> class ShEdgeMidpoint : public Base {
public:
  bool split = false;
  void SetRawStatus(int value) { this->myStatus = value; }
  void Compute() override {
    Base::Compute();
    if (!split)
      return;
    auto values = this->SplitValues();
    occ::handle<NCollection_HSequence<double>> midpoint =
        new NCollection_HSequence<double>;
    midpoint->Append((values->Value(1) + values->Value(values->Length())) * .5);
    this->SetSplitValues(midpoint);
    this->myStatus = 1;
  }
};
class ShEdgeDivideProbe : public ShapeUpgrade_EdgeDivide {
public:
  const TopoDS_Face &Face() const { return myFace; }
};
struct ShEdgeProbeTools {
  occ::handle<ShEdgeMidpoint<ShapeUpgrade_SplitCurve3d>> spatial;
  occ::handle<ShEdgeMidpoint<ShapeUpgrade_SplitCurve2d>> planar;
};
ShEdgeProbeTools sh_ed_tools(GaReader &reader) {
  int a = reader.integer(), b = reader.integer();
  ShEdgeProbeTools tools;
  if (a >= 0) {
    tools.spatial = new ShEdgeMidpoint<ShapeUpgrade_SplitCurve3d>;
    tools.spatial->split = a != 0;
  }
  if (b >= 0) {
    tools.planar = new ShEdgeMidpoint<ShapeUpgrade_SplitCurve2d>;
    tools.planar->split = b != 0;
  }
  return tools;
}
void sh_ed_replace(ShEdgeDivideProbe &divider, const ShEdgeProbeTools &tools) {
  divider.SetSplitCurve3dTool(tools.spatial);
  divider.SetSplitCurve2dTool(tools.planar);
  ga_flag(divider.GetSplitCurve3dTool() == tools.spatial);
  ga_flag(divider.GetSplitCurve2dTool() == tools.planar);
}
void sh_ed_knots(const occ::handle<NCollection_HSequence<double>> &knots) {
  ga_flag(knots.IsNull());
  if (knots.IsNull())
    return;
  ga_int(knots->Length());
  for (double value : *knots)
    ga_real(value);
}
void sh_ed_tool(const occ::handle<ShapeUpgrade_SplitCurve> &tool,
                const occ::handle<NCollection_HSequence<double>> &knots) {
  ga_flag(tool.IsNull());
  if (tool.IsNull())
    return;
  ga_flag(tool->SplitValues() == knots);
  sh_ed_knots(tool->SplitValues());
  sh_flags(*tool);
}
void sh_ed_state(ShEdgeDivideProbe &divider, const TpShapes &shapes,
                 const occ::handle<NCollection_HSequence<double>> &previous3,
                 const occ::handle<NCollection_HSequence<double>> &previous2) {
  ga_int(tp_same_index(shapes, divider.Face()));
  ga_flag(divider.HasCurve3d());
  ga_flag(divider.HasCurve2d());
  ga_flag(divider.Knots3d() == previous3);
  sh_ed_knots(divider.Knots3d());
  sh_ed_knots(previous3);
  ga_flag(divider.Knots2d() == previous2);
  sh_ed_knots(divider.Knots2d());
  sh_ed_knots(previous2);
  auto spatial = divider.GetSplitCurve3dTool();
  auto planar = divider.GetSplitCurve2dTool();
  sh_ed_tool(spatial, divider.Knots3d());
  sh_ed_tool(planar, divider.Knots2d());
  if (!spatial.IsNull())
    sh_split3d::curves(spatial->GetCurves());
  if (!planar.IsNull())
    sh_split2d::curves(planar->GetCurves());
}
void sh_ed_operation(ShEdgeDivideProbe &divider, const TpShapes &shapes,
                     ShEdgeProbeTools &tools, GaReader &reader) {
  int op = reader.integer();
  if (op == 0)
    divider.SetFace(sh_face(shapes, reader.integer()));
  else if (op == 1) {
    GA_TRY(ga_flag(divider.Compute(sh_edge(shapes, reader.integer()))))
  } else if (op == 2)
    divider.Clear();
  else if (op == 3) {
    bool spatial = reader.flag();
    int status = reader.integer();
    if (spatial)
      tools.spatial->SetRawStatus(status);
    else
      tools.planar->SetRawStatus(status);
  } else if (op == 4) {
    auto knots = reader.flag() ? divider.Knots3d() : divider.Knots2d();
    int index = reader.integer();
    double value = reader.real();
    knots->SetValue(index, value);
  } else if (op == 5) {
    tools = sh_ed_tools(reader);
    sh_ed_replace(divider, tools);
  } else {
    bool segment = reader.flag();
    GA_TRY(tools.spatial->Build(segment)) GA_TRY(tools.planar->Build(segment))
  }
}
void sh_edge_divide_case(std::istream &input) {
  GaReader reader{input};
  TpShapes shapes = tp_shapes(reader);
  ShEdgeDivideProbe divider;
  ga_flag(divider.GetSplitCurve3dTool().IsNull());
  ga_flag(divider.GetSplitCurve2dTool().IsNull());
  auto tools = sh_ed_tools(reader);
  sh_ed_replace(divider, tools);
  divider.Clear();
  sh_ed_state(divider, shapes, {}, {});
  int count = reader.integer();
  for (int i = 0; i < count; i++) {
    auto previous3 = divider.Knots3d(), previous2 = divider.Knots2d();
    GA_TRY(sh_ed_operation(divider, shapes, tools, reader))
    sh_ed_state(divider, shapes, previous3, previous2);
  }
  ga_end();
}
