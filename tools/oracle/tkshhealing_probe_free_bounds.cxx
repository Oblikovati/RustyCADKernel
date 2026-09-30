#include <ShapeAnalysis_FreeBounds.hxx>
#include <ShapeAnalysis_Wire.hxx>
using ShBoundSeq = occ::handle<NCollection_HSequence<TopoDS_Shape>>;
void sh_boundary_shapes(const ShBoundSeq &shapes) {
  ga_int(shapes->Length());
  for (const auto &shape : *shapes)
    tp_shape(shape);
}
using ShBoundaryMap =
    NCollection_DataMap<TopoDS_Shape, TopoDS_Shape, TopTools_ShapeMapHasher>;
ShBoundSeq sh_boundary_chain_return(const ShBoundSeq &wires, int mode,
                                    double tolerance, bool shared,
                                    ShBoundaryMap &vertices) {
  if (mode == 0)
    return ShapeAnalysis_FreeBounds::ConnectEdgesToWires(wires, tolerance,
                                                         shared);
  if (mode == 1)
    return ShapeAnalysis_FreeBounds::ConnectWiresToWires(wires, tolerance,
                                                         shared);
  return ShapeAnalysis_FreeBounds::ConnectWiresToWires(wires, tolerance, shared,
                                                       vertices);
}
ShBoundSeq sh_boundary_chain_output(const ShBoundSeq &wires, int mode,
                                    double tolerance, bool shared,
                                    ShBoundaryMap &vertices) {
  ShBoundSeq result = new NCollection_HSequence<TopoDS_Shape>;
  if (wires->Length())
    result->Append(wires->Value(1));
  GA_TRY(if (mode == 8) ShapeAnalysis_FreeBounds::ConnectEdgesToWires(
             wires, tolerance, shared, result);
         else if (mode == 9) ShapeAnalysis_FreeBounds::ConnectWiresToWires(
             wires, tolerance, shared, result);
         else ShapeAnalysis_FreeBounds::ConnectWiresToWires(
             wires, tolerance, shared, result, vertices))
  return result;
}
void sh_boundary_chain(const TpShapes &table, const ShBoundSeq &wires, int mode,
                       double tolerance, bool shared) {
  ShBoundaryMap vertices;
  ShBoundSeq result =
      mode >= 8
          ? sh_boundary_chain_output(wires, mode, tolerance, shared, vertices)
          : sh_boundary_chain_return(wires, mode, tolerance, shared, vertices);
  sh_boundary_shapes(result);
  if (mode != 2 && mode != 10)
    return;
  for (const auto &original : table.shapes) {
    auto found = vertices.Seek(original);
    ga_flag(found != nullptr);
    if (found)
      tp_shape(*found);
  }
}
void sh_boundary_connection_status(const ShapeAnalysis_Wire &analyzer) {
  ga_real(analyzer.MinDistance3d());
  for (auto status : {ShapeExtend_OK, ShapeExtend_DONE, ShapeExtend_DONE1,
                      ShapeExtend_DONE2, ShapeExtend_DONE3, ShapeExtend_FAIL,
                      ShapeExtend_FAIL1, ShapeExtend_FAIL2}) {
    ga_flag(analyzer.LastCheckStatus(status));
    ga_flag(analyzer.StatusConnected(status));
  }
}
void sh_boundary_connection(const ShBoundSeq &wires, double tolerance,
                            GaReader &reader) {
  ShapeAnalysis_Wire analyzer;
  if (wires->Length())
    analyzer.Load(TopoDS::Wire(wires->Value(1)));
  analyzer.SetPrecision(tolerance);
  int count = reader.integer();
  for (int i = 0; i < count; i++) {
    int index = reader.integer();
    double precision = reader.real();
    GA_TRY(ga_flag(index == -99 ? analyzer.CheckConnected(precision)
                                : analyzer.CheckConnected(index, precision));
           sh_boundary_connection_status(analyzer))
  }
}
void sh_boundary_split(const ShBoundSeq &wires, double tolerance, bool shared) {
  ShBoundSeq closed = new NCollection_HSequence<TopoDS_Shape>,
             open = new NCollection_HSequence<TopoDS_Shape>;
  GA_TRY(ShapeAnalysis_FreeBounds::SplitWires(wires, tolerance, shared, closed,
                                              open))
  sh_boundary_shapes(closed);
  sh_boundary_shapes(open);
}
void sh_boundary_dispatch(const ShBoundSeq &wires) {
  TopoDS_Compound closed, open;
  ShapeAnalysis_FreeBounds::DispatchWires(wires, closed, open);
  tp_shape(closed);
  tp_shape(open);
}
void sh_boundary_result(const ShapeAnalysis_FreeBounds &result) {
  tp_shape(result.GetClosedWires());
  tp_shape(result.GetOpenWires());
}
void sh_boundary_construct(const ShBoundSeq &wires, int mode, double tolerance,
                           bool shared, GaReader &reader) {
  bool closed = reader.flag(), open = reader.flag();
  if (mode == 5)
    sh_boundary_result(
        ShapeAnalysis_FreeBounds(wires->Value(1), tolerance, closed, open));
  else
    sh_boundary_result(
        ShapeAnalysis_FreeBounds(wires->Value(1), closed, open, shared));
}
void sh_boundary_other(const ShBoundSeq &wires, int mode, double tolerance,
                       bool shared, GaReader &reader) {
  if (mode == 3 || mode == 11)
    sh_boundary_split(wires, tolerance, shared);
  else if (mode == 4)
    sh_boundary_dispatch(wires);
  else if (mode == 12)
    sh_boundary_result(ShapeAnalysis_FreeBounds());
  else if (mode == 7)
    sh_boundary_connection(wires, tolerance, reader);
  else
    sh_boundary_construct(wires, mode, tolerance, shared, reader);
}
void sh_free_bounds_case(std::istream &input) {
  GaReader reader{input};
  GA_TRY(auto table = tp_shapes(reader); int mode = reader.integer();
         double tolerance = reader.real(); bool shared = reader.flag();
         ShBoundSeq wires = new NCollection_HSequence<TopoDS_Shape>;
         int count = reader.integer();
         for (int i = 0; i < count; i++)
             wires->Append(sh_shape(table, reader.integer()));
         if (mode <= 2 || (mode >= 8 && mode <= 10))
             sh_boundary_chain(table, wires, mode, tolerance, shared);
         else sh_boundary_other(wires, mode, tolerance, shared, reader);
         sh_boundary_shapes(wires))
  ga_end();
}
