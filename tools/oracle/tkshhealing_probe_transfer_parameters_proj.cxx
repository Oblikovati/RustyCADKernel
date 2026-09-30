#include <BRep_PointRepresentation.hxx>
#include <BRep_TVertex.hxx>
#include <ShapeAnalysis_TransferParametersProj.hxx>
void sh_transfer_point_representation(
    const occ::handle<BRep_PointRepresentation> &point) {
  ga_flag(point.IsNull());
  if (point.IsNull())
    return;
  ga_flag(point->IsPointOnCurve());
  ga_flag(point->IsPointOnCurveOnSurface());
  ga_flag(point->IsPointOnSurface());
  ga_real(point->Parameter());
  shc_location(point->Location());
  if (point->IsPointOnCurve())
    shc_curve(point->Curve());
  if (point->IsPointOnCurveOnSurface()) {
    shc_curve2d(point->PCurve());
    shc_surface(point->Surface());
  }
  if (point->IsPointOnSurface()) {
    ga_real(point->Parameter2());
    shc_surface(point->Surface());
  }
}
void sh_transfer_vertex_digest(const TopoDS_Vertex &vertex) {
  ga_flag(vertex.IsNull());
  if (vertex.IsNull())
    return;
  ga_int(vertex.Orientation());
  ga_pnt(BRep_Tool::Pnt(vertex));
  ga_real(BRep_Tool::Tolerance(vertex));
  const auto &points = occ::down_cast<BRep_TVertex>(vertex.TShape())->Points();
  ga_int(points.Size());
  for (const auto &point : points)
    sh_transfer_point_representation(point);
}
void sh_transfer_copy_vertex(const TpShapes &shapes, GaReader &reader,
                             bool onFace) {
  auto vertex = TopoDS::Vertex(sh_shape(shapes, reader.integer()));
  int target = reader.integer(), source = reader.integer();
  auto copy =
      onFace ? ShapeAnalysis_TransferParametersProj::CopyNMVertex(
                   vertex, sh_face(shapes, target), sh_face(shapes, source))
             : ShapeAnalysis_TransferParametersProj::CopyNMVertex(
                   vertex, sh_edge(shapes, target), sh_edge(shapes, source));
  ga_flag(copy.IsSame(vertex));
  sh_transfer_vertex_digest(copy);
  sh_transfer_vertex_digest(vertex);
  if (!copy.IsNull() && !onFace)
    ga_real(BRep_Tool::Parameter(copy, sh_edge(shapes, target)));
}
void sh_projection_transfer_operation(
    ShapeAnalysis_TransferParametersProj &tool, const TpShapes &shapes,
    const TopoDS_Edge &edge, const TopoDS_Face &face, GaReader &reader) {
  int operation = reader.integer();
  if (operation == 5) {
    tool.ForceProjection() = reader.flag();
    return;
  }
  if (operation == 6 || operation == 7) {
    sh_transfer_copy_vertex(shapes, reader, operation == 7);
    return;
  }
  sh_transfer_operation_code(operation, tool, shapes, edge, face, reader);
}
occ::handle<ShapeAnalysis_TransferParametersProj>
sh_transfer_projection_tool(const TopoDS_Edge &edge, const TopoDS_Face &face,
                            bool initialized) {
  if (initialized)
    return new ShapeAnalysis_TransferParametersProj(edge, face);
  occ::handle<ShapeAnalysis_TransferParametersProj> tool =
      new ShapeAnalysis_TransferParametersProj;
  ga_flag(tool->IsSameRange());
  ga_flag(tool->ForceProjection());
  tool->Init(edge, face);
  return tool;
}
void sh_transfer_projection_request(std::istream &input) {
  GaReader reader{input};
  TpShapes shapes = tp_shapes(reader);
  auto edge = sh_edge(shapes, reader.integer());
  auto face = sh_face(shapes, reader.integer());
  auto tool = sh_transfer_projection_tool(edge, face, reader.flag());
  ga_flag(tool->IsSameRange());
  ga_flag(tool->ForceProjection());
  int count = reader.integer();
  for (int i = 0; i < count; ++i) {
    GA_TRY(sh_projection_transfer_operation(*tool, shapes, edge, face, reader));
    ga_flag(tool->IsSameRange());
    ga_flag(tool->ForceProjection());
  }
}
void sh_transfer_parameters_proj_case(std::istream &input) {
  GA_TRY(sh_transfer_projection_request(input));
  ga_end();
}
