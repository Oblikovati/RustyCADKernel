#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Plane.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeFix_Wire.hxx>
#include <ShapeExtend_Status.hxx>
#include <NCollection_Array1.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Edge edgeOnPlane(const TopoDS_Face& face, const gp_Pnt& start,
                               const gp_Dir& direction, const gp_Pnt2d& uvStart,
                               const gp_Dir2d& uvDirection) {
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge, new Geom_Line(gp_Ax1(start, direction)), 1.e-7);
  TopoDS_Vertex first, last;
  builder.MakeVertex(first, start, 1.e-7);
  builder.MakeVertex(last, gp_Pnt(start.X() + 2. * direction.X(),
                                  start.Y() + 2. * direction.Y(),
                                  start.Z() + 2. * direction.Z()), 1.e-7);
  builder.Add(edge, first.Oriented(TopAbs_FORWARD));
  builder.Add(edge, last.Oriented(TopAbs_REVERSED));
  builder.UpdateEdge(edge, new Geom2d_Line(gp_Ax2d(uvStart, uvDirection)), face, 1.e-7);
  builder.Range(edge, face, 0., 2.);
  builder.Range(edge, 0., 2.);
  builder.SameRange(edge, true);
  builder.SameParameter(edge, true);
  return edge;
}

static TopoDS_Wire crossingWire(const TopoDS_Face& face) {
  BRep_Builder builder;
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, edgeOnPlane(face, gp_Pnt(-1., 0., 0.), gp_Dir(1., 0., 0.),
                                gp_Pnt2d(-1., 0.), gp_Dir2d(1., 0.)));
  builder.Add(wire, edgeOnPlane(face, gp_Pnt(3., 3., 0.), gp_Dir(1., 0., 0.),
                                gp_Pnt2d(3., 3.), gp_Dir2d(1., 0.)));
  builder.Add(wire, edgeOnPlane(face, gp_Pnt(0., -1., 0.), gp_Dir(0., 1., 0.),
                                gp_Pnt2d(0., -1.), gp_Dir2d(0., 1.)));
  builder.Add(wire, edgeOnPlane(face, gp_Pnt(-4., -4., 0.), gp_Dir(0., 1., 0.),
                                gp_Pnt2d(-4., -4.), gp_Dir2d(0., 1.)));
  return wire;
}

static void printStatuses(const ShapeFix_Wire& fixer) {
  for (int status = ShapeExtend_OK; status <= ShapeExtend_FAIL; ++status) {
    std::cout << " " << fixer.StatusSelfIntersection(static_cast<ShapeExtend_Status>(status));
  }
}

static void printRanges(const ShapeFix_Wire& fixer, const TopoDS_Face& face) {
  for (int index = 1; index <= fixer.WireData()->NbEdges(); ++index) {
    double first, last;
    BRep_Tool::Range(fixer.WireData()->Edge(index), face, first, last);
    std::cout << " " << first << " " << last;
  }
}

static int appliedEdgeCount(const Handle(ShapeBuild_ReShape)& context, const TopoDS_Wire& wire);

static void printObservation(const ShapeFix_Wire& fixer, const TopoDS_Wire& wire,
                            const TopoDS_Face& face,
                            const Handle(ShapeBuild_ReShape)& context, bool changed) {
  std::cout << std::setprecision(17) << changed << " " << fixer.WireData()->NbEdges()
            << " " << fixer.StatusRemovedSegment();
  printStatuses(fixer);
  printRanges(fixer, face);
  std::cout << " " << appliedEdgeCount(context, wire) << "\n";
}

static int appliedEdgeCount(const Handle(ShapeBuild_ReShape)& context, const TopoDS_Wire& wire) {
  TopoDS_Shape applied = context->Apply(wire);
  int count = 0;
  for (TopExp_Explorer edge(applied, TopAbs_EDGE); edge.More(); edge.Next()) ++count;
  return count;
}

static void configureNonAdjacentOnly(ShapeFix_Wire& fixer) {
  fixer.FixSelfIntersectingEdgeMode() = 0;
  fixer.FixIntersectingEdgesMode() = 0;
  fixer.FixNonAdjacentIntersectingEdgesMode() = 1;
}

static void runOracle(const TopoDS_Wire& wire, const TopoDS_Face& face, bool isolateNonAdjacent) {
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_Wire fixer(wire, face, 1.e-7);
  fixer.SetContext(context);
  if (isolateNonAdjacent) configureNonAdjacentOnly(fixer);
  bool changed = fixer.FixSelfIntersection();
  printObservation(fixer, wire, face, context, changed);
}

static Handle(Geom_BezierCurve) loopCurve3d() {
  NCollection_Array1<gp_Pnt> poles(1, 4);
  poles.SetValue(1, gp_Pnt(0., 0., 0.)); poles.SetValue(2, gp_Pnt(3., 3., 0.));
  poles.SetValue(3, gp_Pnt(-3., 3., 0.)); poles.SetValue(4, gp_Pnt(1., 0., 0.));
  return new Geom_BezierCurve(poles);
}

static Handle(Geom2d_BezierCurve) loopPCurve() {
  NCollection_Array1<gp_Pnt2d> poles(1, 4);
  poles.SetValue(1, gp_Pnt2d(0., 0.)); poles.SetValue(2, gp_Pnt2d(3., 3.));
  poles.SetValue(3, gp_Pnt2d(-3., 3.)); poles.SetValue(4, gp_Pnt2d(1., 0.));
  return new Geom2d_BezierCurve(poles);
}

static TopoDS_Edge loopEdge(const TopoDS_Face& face) {
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge, loopCurve3d(), 1.e-7);
  TopoDS_Vertex first, last;
  builder.MakeVertex(first, gp_Pnt(0., 0., 0.), 1.e-7);
  builder.MakeVertex(last, gp_Pnt(1., 0., 0.), 1.e-7);
  builder.Add(edge, first.Oriented(TopAbs_FORWARD));
  builder.Add(edge, last.Oriented(TopAbs_REVERSED));
  builder.UpdateEdge(edge, loopPCurve(), face, 1.e-7);
  builder.Range(edge, face, 0., 1.);
  builder.Range(edge, 0., 1.);
  builder.SameRange(edge, false);
  builder.SameParameter(edge, false);
  return edge;
}

static TopoDS_Wire loopWire(const TopoDS_Face& face) {
  BRep_Builder builder;
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, loopEdge(face));
  return wire;
}

static void runRemoveLoopModeOne(const TopoDS_Face& face) {
  TopoDS_Wire wire = loopWire(face);
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_Wire fixer(wire, face, 1.e-7);
  fixer.SetContext(context);
  fixer.FixSelfIntersectingEdgeMode() = 1;
  fixer.FixIntersectingEdgesMode() = 0;
  fixer.FixNonAdjacentIntersectingEdgesMode() = 0;
  fixer.ModifyRemoveLoopMode() = 1;
  printObservation(fixer, wire, face, context, fixer.FixSelfIntersection());
}

static void setGeometryModesNonDefault(ShapeFix_Wire& fixer) {
  fixer.ModifyTopologyMode() = true; fixer.ModifyGeometryMode() = false;
  fixer.ClosedWireMode() = false; fixer.PreferencePCurveMode() = false;
  fixer.FixGapsByRangesMode() = true; fixer.ModifyRemoveLoopMode() = 1;
  fixer.FixReversed2dMode() = 1; fixer.FixRemovePCurveMode() = 1;
  fixer.FixRemoveCurve3dMode() = 1; fixer.FixAddPCurveMode() = 1;
  fixer.FixAddCurve3dMode() = 1; fixer.FixSeamMode() = 1;
  fixer.FixShiftedMode() = 1; fixer.FixSameParameterMode() = 1;
  fixer.FixVertexToleranceMode() = 1;
}

static void setAggregateModesNonDefault(ShapeFix_Wire& fixer) {
  fixer.FixNotchedEdgesMode() = 1; fixer.FixSelfIntersectingEdgeMode() = 1;
  fixer.FixIntersectingEdgesMode() = 1; fixer.FixNonAdjacentIntersectingEdgesMode() = 1;
  fixer.FixTailMode() = 1; fixer.FixReorderMode() = 1; fixer.FixSmallMode() = 1;
  fixer.FixConnectedMode() = 1; fixer.FixEdgeCurvesMode() = 1;
  fixer.FixDegeneratedMode() = 1; fixer.FixSelfIntersectionMode() = 1;
  fixer.FixLackingMode() = 1; fixer.FixGaps3dMode() = 1; fixer.FixGaps2dMode() = 1;
}

static void printGeometryModeDefaults(ShapeFix_Wire& fixer) {
  std::cout << fixer.ModifyTopologyMode() << " " << fixer.ModifyGeometryMode()
            << " " << fixer.ClosedWireMode() << " " << fixer.PreferencePCurveMode()
            << " " << fixer.FixGapsByRangesMode() << " " << fixer.ModifyRemoveLoopMode();
  std::cout << " " << fixer.FixReversed2dMode() << " " << fixer.FixRemovePCurveMode()
            << " " << fixer.FixRemoveCurve3dMode() << " " << fixer.FixAddPCurveMode()
            << " " << fixer.FixAddCurve3dMode() << " " << fixer.FixSeamMode()
            << " " << fixer.FixShiftedMode() << " " << fixer.FixSameParameterMode()
            << " " << fixer.FixVertexToleranceMode();
}

static void printAggregateModeDefaults(ShapeFix_Wire& fixer) {
  std::cout << " " << fixer.FixNotchedEdgesMode() << " " << fixer.FixSelfIntersectingEdgeMode()
            << " " << fixer.FixIntersectingEdgesMode() << " " << fixer.FixNonAdjacentIntersectingEdgesMode()
            << " " << fixer.FixTailMode() << " " << fixer.FixReorderMode() << " " << fixer.FixSmallMode()
            << " " << fixer.FixConnectedMode() << " " << fixer.FixEdgeCurvesMode()
            << " " << fixer.FixDegeneratedMode() << " " << fixer.FixSelfIntersectionMode()
            << " " << fixer.FixLackingMode() << " " << fixer.FixGaps3dMode() << " " << fixer.FixGaps2dMode();
}

static void printStatusesAfterClear(ShapeFix_Wire& fixer) {
  std::cout << fixer.LastFixStatus(ShapeExtend_DONE) << " " << fixer.StatusReorder(ShapeExtend_DONE)
            << " " << fixer.StatusSmall(ShapeExtend_DONE) << " " << fixer.StatusConnected(ShapeExtend_DONE)
            << " " << fixer.StatusEdgeCurves(ShapeExtend_DONE) << " " << fixer.StatusDegenerated(ShapeExtend_DONE)
            << " " << fixer.StatusClosed(ShapeExtend_DONE) << " " << fixer.StatusSelfIntersection(ShapeExtend_DONE)
            << " " << fixer.StatusLacking(ShapeExtend_DONE) << " " << fixer.StatusGaps3d(ShapeExtend_DONE)
            << " " << fixer.StatusGaps2d(ShapeExtend_DONE) << " " << fixer.StatusNotches(ShapeExtend_DONE)
            << " " << fixer.StatusFixTails(ShapeExtend_DONE) << " " << fixer.StatusRemovedSegment();
}

static void runClearModes(const TopoDS_Face& face) {
  TopoDS_Wire wire = crossingWire(face);
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_Wire fixer(wire, face, 1.e-7);
  fixer.SetContext(context);
  fixer.FixSelfIntersection();
  setGeometryModesNonDefault(fixer);
  setAggregateModesNonDefault(fixer);
  fixer.ClearModes();
  printGeometryModeDefaults(fixer);
  printAggregateModeDefaults(fixer);
  std::cout << " " << fixer.StatusSelfIntersection(ShapeExtend_DONE);
  fixer.ClearStatuses();
  std::cout << " ";
  printStatusesAfterClear(fixer);
  std::cout << "\n";
}

int main() {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  runOracle(crossingWire(face), face, true);
  runOracle(crossingWire(face), face, false);
  runRemoveLoopModeOne(face);
  runClearModes(face);
}
