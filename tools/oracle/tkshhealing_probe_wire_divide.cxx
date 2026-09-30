#include <ShapeBuild_ReShape.hxx>
#include <ShapeUpgrade_WireDivide.hxx>
void sh_wire_repeat(ShapeUpgrade_WireDivide &divider,
                    const occ::handle<ShapeBuild_ReShape> &context,
                    const TopoDS_Edge &edge, int count) {
  for (int i = 0; i < count; ++i) {
    GA_TRY(divider.Perform())
    sh_flags(divider);
    tp_shape(divider.Wire());
    tp_shape(edge);
    tp_shape(context->Apply(edge));
  }
}
void sh_wire_divide_case(std::istream &input) {
  GaReader reader{input};
  GA_TRY({
    TpShapes shapes = tp_shapes(reader);
    ShapeUpgrade_WireDivide divider;
    auto tools = sh_ed_tools(reader);
    divider.SetSplitCurve3dTool(tools.spatial);
    divider.SetSplitCurve2dTool(tools.planar);
    occ::handle<ShapeBuild_ReShape> context = new ShapeBuild_ReShape;
    divider.SetContext(context);
    int edgeIndex = reader.integer(), faceIndex = reader.integer();
    auto edge = sh_edge(shapes, edgeIndex);
    divider.Load(edge);
    divider.SetFace(sh_face(shapes, faceIndex));
    divider.SetEdgeMode(reader.integer());
    divider.SetMinTolerance(reader.real());
    if (!reader.flag())
      divider.SetContext({});
    sh_wire_repeat(divider, context, edge, reader.integer());
  })
  ga_end();
}
