#include <BRepLib_MakeWire.hxx>
class ShWireState : public ShapeUpgrade_WireDivide {
public:
  using ShapeUpgrade_WireDivide::GetSplitCurve2dTool;
  using ShapeUpgrade_WireDivide::GetSplitCurve3dTool;
  void Print() {
    sh_flags(*this);
    ga_int(myEdgeMode);
    tp_shape(myFace);
    tp_shape(myWire);
    ga_flag(GetSplitCurve3dTool().IsNull());
    ga_flag(GetSplitCurve2dTool().IsNull());
    ga_flag(GetTransferParamTool().IsNull());
    ga_flag(GetEdgeDivideTool().IsNull());
    ga_flag(GetFixSmallCurveTool().IsNull());
  }
};
void sh_wire_initialize(ShWireState &divider, const TopoDS_Wire &wire,
                        const TopoDS_Edge &edge, const TopoDS_Face &face,
                        const occ::handle<Geom_Surface> &surface,
                        const TopLoc_Location &location) {
  divider.Init(wire, face);
  divider.Print();
  divider.Init(wire, surface);
  divider.Print();
  divider.SetFace(face);
  divider.Print();
  divider.SetSurface(surface);
  divider.Print();
  divider.SetSurface(surface, location);
  divider.Print();
  divider.Load(TopoDS_Wire());
  divider.Print();
  divider.Load(edge);
  divider.Print();
  divider.Load(wire);
  divider.Print();
}
void sh_wire_replace_tools(ShWireState &divider) {
  occ::handle<ShapeAnalysis_TransferParameters> transfer =
      new ShapeAnalysis_TransferParameters;
  occ::handle<ShapeUpgrade_EdgeDivide> criterion = new ShapeUpgrade_EdgeDivide;
  occ::handle<ShapeUpgrade_FixSmallCurves> small =
      new ShapeUpgrade_FixSmallCurves;
  divider.SetTransferParamTool(transfer);
  divider.SetEdgeDivideTool(criterion);
  divider.SetFixSmallCurveTool(small);
  ga_flag(divider.GetTransferParamTool() == transfer);
  ga_flag(divider.GetEdgeDivideTool() == criterion);
  ga_flag(divider.GetFixSmallCurveTool() == small);
}
void sh_wire_finish_state(ShWireState &divider, const TopoDS_Wire &wire,
                          const TopoDS_Edge &edge, const TopoDS_Face &face,
                          GaReader &reader) {
  divider.SetEdgeMode(reader.integer());
  divider.Print();
  divider.SetFace(face);
  GA_TRY(divider.Perform()) divider.Print();
  divider.Load(edge);
  divider.Print();
  divider.Init(wire, face);
  divider.Print();
  divider.SetTransferParamTool({});
  divider.SetEdgeDivideTool({});
  divider.SetFixSmallCurveTool({});
  divider.Print();
}
void sh_wire_configure(ShWireState &divider, const TopoDS_Wire &wire,
                       const TopoDS_Edge &edge, const TopoDS_Face &face,
                       GaReader &reader) {
  auto tools = sh_ed_tools(reader);
  divider.SetSplitCurve3dTool(tools.spatial);
  divider.SetSplitCurve2dTool(tools.planar);
  ga_flag(divider.GetSplitCurve3dTool() == tools.spatial);
  ga_flag(divider.GetSplitCurve2dTool() == tools.planar);
  sh_wire_replace_tools(divider);
  sh_wire_finish_state(divider, wire, edge, face, reader);
}
void sh_wire_state_case(std::istream &input) {
  GaReader reader{input};
  GA_TRY({
    TpShapes shapes = tp_shapes(reader);
    ShWireState divider;
    auto edge = sh_edge(shapes, reader.integer());
    auto face = sh_face(shapes, reader.integer());
    auto surface = shapes.surfaces.at(reader.integer());
    auto location = shapes.locations.at(reader.integer());
    auto wire = BRepLib_MakeWire(edge).Wire();
    divider.SetContext(new ShapeBuild_ReShape);
    divider.Print();
    sh_wire_initialize(divider, wire, edge, face, surface, location);
    sh_wire_configure(divider, wire, edge, face, reader);
  })
  ga_end();
}
