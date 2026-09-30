#include <ShapeAnalysis_TransferParameters.hxx>
void sh_transfer_geometry(const TopoDS_Edge &edge, const TopoDS_Face &face) {
  ShapeAnalysis_Edge analysis;
  occ::handle<Geom_Curve> spatial;
  double first = 0., last = 0.;
  analysis.Curve3d(edge, spatial, first, last, false);
  sh_curve3d(spatial, first, last);
  if (!face.IsNull()) {
    occ::handle<Geom2d_Curve> planar;
    first = last = 0.;
    analysis.PCurve(edge, face, planar, first, last, false);
    sh_curve2d(planar, first, last);
  }
  ga_flag(BRep_Tool::SameRange(edge));
  ga_flag(BRep_Tool::SameParameter(edge));
}
void sh_transfer_sequence(ShapeAnalysis_TransferParameters &tool,
                          GaReader &reader) {
  bool to2d = reader.flag();
  int count = reader.integer();
  occ::handle<NCollection_HSequence<double>> parameters =
      new NCollection_HSequence<double>;
  for (int i = 0; i < count; ++i)
    parameters->Append(reader.real());
  auto result = tool.Perform(parameters, to2d);
  ga_flag(result == parameters);
  ga_int(result->Length());
  for (int i = 1; i <= result->Length(); ++i)
    ga_real(result->Value(i));
}
void sh_transfer_range(ShapeAnalysis_TransferParameters &tool,
                       const TopoDS_Edge &edge, const TopoDS_Face &face,
                       GaReader &reader) {
  double previous = reader.real(), current = reader.real();
  bool is2d = reader.flag();
  auto target = ShapeBuild_Edge().Copy(edge, true);
  GA_TRY(tool.TransferRange(target, previous, current, is2d));
  sh_transfer_geometry(target, face);
  sh_transfer_geometry(edge, face);
}
void sh_transfer_operation_code(int operation,
                                ShapeAnalysis_TransferParameters &tool,
                                const TpShapes &shapes, const TopoDS_Edge &edge,
                                const TopoDS_Face &face, GaReader &reader) {
  if (operation == 0) {
    double parameter = reader.real();
    ga_real(tool.Perform(parameter, reader.flag()));
  } else if (operation == 1)
    sh_transfer_sequence(tool, reader);
  else if (operation == 2)
    sh_transfer_range(tool, edge, face, reader);
  else if (operation == 3) {
    auto next = sh_edge(shapes, reader.integer());
    auto support = sh_face(shapes, reader.integer());
    tool.Init(next, support);
  } else
    tool.SetMaxTolerance(reader.real());
}
void sh_transfer_operation(ShapeAnalysis_TransferParameters &tool,
                           const TpShapes &shapes, const TopoDS_Edge &edge,
                           const TopoDS_Face &face, GaReader &reader) {
  sh_transfer_operation_code(reader.integer(), tool, shapes, edge, face,
                             reader);
}
void sh_transfer_parameters_case(std::istream &input) {
  GaReader reader{input};
  TpShapes shapes = tp_shapes(reader);
  auto edge = sh_edge(shapes, reader.integer());
  auto face = sh_face(shapes, reader.integer());
  occ::handle<ShapeAnalysis_TransferParameters> tool;
  if (reader.flag())
    tool = new ShapeAnalysis_TransferParameters(edge, face);
  else {
    tool = new ShapeAnalysis_TransferParameters;
    ga_flag(tool->IsSameRange());
    tool->Init(edge, face);
  }
  ga_flag(tool->IsSameRange());
  int count = reader.integer();
  for (int i = 0; i < count; ++i) {
    GA_TRY(sh_transfer_operation(*tool, shapes, edge, face, reader));
    ga_flag(tool->IsSameRange());
  }
  ga_end();
}
