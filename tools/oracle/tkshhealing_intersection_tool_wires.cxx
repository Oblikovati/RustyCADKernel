#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeFix_IntersectionTool.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <Geom2d_Line.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Dir2d.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Edge edgeOnPlane(const TopoDS_Face& face,
                               const gp_Pnt& start,
                               const gp_Dir& direction,
                               const gp_Pnt2d& uvStart,
                               const gp_Dir2d& uvDirection,
                               double lastParameter = 2.) {
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge, new Geom_Line(gp_Ax1(start, direction)), 1.e-7);
  TopoDS_Vertex first, last;
  builder.MakeVertex(first, start, 1.e-7);
  builder.MakeVertex(last, gp_Pnt(start.X() + lastParameter * direction.X(),
                                  start.Y() + lastParameter * direction.Y(),
                                  start.Z() + lastParameter * direction.Z()), 1.e-7);
  builder.Add(edge, first.Oriented(TopAbs_FORWARD));
  builder.Add(edge, last.Oriented(TopAbs_REVERSED));
  builder.UpdateEdge(edge, new Geom2d_Line(gp_Ax2d(uvStart, uvDirection)), face, 1.e-7);
  builder.Range(edge, face, 0., lastParameter);
  builder.Range(edge, 0., lastParameter);
  builder.SameRange(edge, true);
  builder.SameParameter(edge, true);
  return edge;
}

static Handle(ShapeExtend_WireData) makeWire(const TopoDS_Face& face,
                                              const TopoDS_Edge& first,
                                              const TopoDS_Edge& second) {
  Handle(ShapeExtend_WireData) wire = new ShapeExtend_WireData;
  wire->Add(first);
  wire->Add(second);
  return wire;
}

static void runVertexCase(bool middleEndpoint) {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  gp_Pnt firstStart = middleEndpoint ? gp_Pnt(-1., 0., 0.) : gp_Pnt(0., 0., 0.);
  gp_Pnt2d firstUv = middleEndpoint ? gp_Pnt2d(-1., 0.) : gp_Pnt2d(0., 0.);
  TopoDS_Edge firstCross = edgeOnPlane(face, firstStart, gp_Dir(1., 0., 0.),
                                       firstUv, gp_Dir2d(1., 0.));
  TopoDS_Edge secondCross = edgeOnPlane(face, gp_Pnt(0., 0., 0.), gp_Dir(0., 1., 0.),
                                        gp_Pnt2d(0., 0.), gp_Dir2d(0., 1.));
  TopoDS_Edge firstFar = edgeOnPlane(face, gp_Pnt(3., 3., 0.), gp_Dir(1., 0., 0.),
                                     gp_Pnt2d(3., 3.), gp_Dir2d(1., 0.));
  TopoDS_Edge secondFar = edgeOnPlane(face, gp_Pnt(-4., -4., 0.), gp_Dir(0., 1., 0.),
                                      gp_Pnt2d(-4., -4.), gp_Dir2d(0., 1.));
  Handle(ShapeExtend_WireData) first = makeWire(face, firstCross, firstFar);
  Handle(ShapeExtend_WireData) second = makeWire(face, secondCross, secondFar);
  builder.Add(face, first->Wire());
  builder.Add(face, second->Wire());
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool tool(context, 1.e-7);
  bool done = tool.FixIntersectingWires(face);
  Handle(ShapeExtend_WireData) resultFirst = new ShapeExtend_WireData;
  Handle(ShapeExtend_WireData) resultSecond = new ShapeExtend_WireData;
  for (TopoDS_Iterator wires(face, false, true); wires.More(); wires.Next()) {
    if (wires.Value().ShapeType() != TopAbs_WIRE) continue;
    if (resultFirst->NbEdges() == 0) resultFirst->Init(TopoDS::Wire(wires.Value()));
    else resultSecond->Init(TopoDS::Wire(wires.Value()));
  }
  ShapeAnalysis_Edge analysis;
  TopoDS_Vertex firstShared = middleEndpoint
      ? analysis.LastVertex(resultFirst->Edge(1)) : analysis.FirstVertex(resultFirst->Edge(1));
  TopoDS_Vertex secondShared = analysis.FirstVertex(resultSecond->Edge(1));
  std::cout << done << " " << resultFirst->NbEdges() << " " << resultSecond->NbEdges()
            << " " << firstShared.IsSame(secondShared) << '\n';
}

static void runShortOverlapCase() {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge firstOverlap = edgeOnPlane(face, gp_Pnt(0., 0., 0.), gp_Dir(1., 0., 0.),
                                         gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.), 10.);
  TopoDS_Edge secondOverlap = edgeOnPlane(face, gp_Pnt(5., 0., 5.e-8), gp_Dir(1., 0., 0.),
                                          gp_Pnt2d(5., 0.), gp_Dir2d(1., 0.), 10.);
  TopoDS_Edge firstFar = edgeOnPlane(face, gp_Pnt(30., 30., 0.), gp_Dir(1., 0., 0.),
                                     gp_Pnt2d(30., 30.), gp_Dir2d(1., 0.));
  TopoDS_Edge secondFar = edgeOnPlane(face, gp_Pnt(-30., -30., 0.), gp_Dir(0., 1., 0.),
                                      gp_Pnt2d(-30., -30.), gp_Dir2d(0., 1.));
  Handle(ShapeExtend_WireData) first = makeWire(face, firstOverlap, firstFar);
  Handle(ShapeExtend_WireData) second = makeWire(face, secondOverlap, secondFar);
  builder.Add(face, first->Wire());
  builder.Add(face, second->Wire());
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool tool(context, 1.e-7);
  bool done = tool.FixIntersectingWires(face);
  int wireCounts[2] = {0, 0};
  TopoDS_Wire resultWires[2];
  int wireIndex = 0;
  for (TopoDS_Iterator wires(face, false, true); wires.More(); wires.Next()) {
    if (wires.Value().ShapeType() != TopAbs_WIRE) continue;
    resultWires[wireIndex] = TopoDS::Wire(wires.Value());
    for (TopoDS_Iterator edges(wires.Value(), false, true); edges.More(); edges.Next()) ++wireCounts[wireIndex];
    ++wireIndex;
  }
  std::cout << std::setprecision(17) << done << " " << wireCounts[0] << " " << wireCounts[1];
  for (const TopoDS_Wire& wire : resultWires) {
    Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData(wire);
    for (int index = 1; index <= data->NbEdges(); ++index) {
      double first, last;
      BRep_Tool::Range(data->Edge(index), face, first, last);
      std::cout << " " << first << " " << last;
    }
  }
  std::cout << '\n';
}

static void runLongOverlapCase() {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge firstOverlap = edgeOnPlane(face, gp_Pnt(0., 0., 0.), gp_Dir(1., 0., 0.),
                                         gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.), 10.);
  TopoDS_Edge secondOverlap = edgeOnPlane(face, gp_Pnt(2., 0., 0.), gp_Dir(1., 0., 0.),
                                          gp_Pnt2d(2., 0.), gp_Dir2d(1., 0.), 6.);
  ShapeAnalysis_Edge analysis;
  builder.UpdateVertex(analysis.FirstVertex(firstOverlap), 1.e-5);
  TopoDS_Edge firstFar = edgeOnPlane(face, gp_Pnt(30., 30., 0.), gp_Dir(1., 0., 0.),
                                     gp_Pnt2d(30., 30.), gp_Dir2d(1., 0.));
  TopoDS_Edge secondFar = edgeOnPlane(face, gp_Pnt(-30., -30., 0.), gp_Dir(0., 1., 0.),
                                      gp_Pnt2d(-30., -30.), gp_Dir2d(0., 1.));
  Handle(ShapeExtend_WireData) first = makeWire(face, firstOverlap, firstFar);
  Handle(ShapeExtend_WireData) second = makeWire(face, secondOverlap, secondFar);
  builder.Add(face, first->Wire());
  builder.Add(face, second->Wire());
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool tool(context, 1.e-7);
  bool done = tool.FixIntersectingWires(face);
  Handle(ShapeExtend_WireData) result[2];
  int wireIndex = 0;
  for (TopoDS_Iterator wires(face, false, true); wires.More(); wires.Next()) {
    if (wires.Value().ShapeType() == TopAbs_WIRE) {
      result[wireIndex++] = new ShapeExtend_WireData(TopoDS::Wire(wires.Value()));
    }
  }
  std::cout << std::setprecision(17) << done << " " << result[0]->NbEdges() << " " << result[1]->NbEdges();
  for (int wire = 0; wire < 2; ++wire) {
    for (int index = 1; index <= result[wire]->NbEdges(); ++index) {
      double firstParameter, lastParameter;
      BRep_Tool::Range(result[wire]->Edge(index), face, firstParameter, lastParameter);
      std::cout << " " << firstParameter << " " << lastParameter;
    }
  }
  std::cout << '\n';
}

static void runContainedFirstLongOverlapCase() {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge firstOverlap = edgeOnPlane(face, gp_Pnt(2., 0., 0.), gp_Dir(1., 0., 0.),
                                         gp_Pnt2d(2., 0.), gp_Dir2d(1., 0.), 6.);
  TopoDS_Edge secondOverlap = edgeOnPlane(face, gp_Pnt(0., 0., 0.), gp_Dir(1., 0., 0.),
                                          gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.), 10.);
  ShapeAnalysis_Edge analysis;
  builder.UpdateVertex(analysis.FirstVertex(secondOverlap), 1.e-5);
  TopoDS_Edge firstFar = edgeOnPlane(face, gp_Pnt(30., 30., 0.), gp_Dir(1., 0., 0.),
                                     gp_Pnt2d(30., 30.), gp_Dir2d(1., 0.));
  TopoDS_Edge secondFar = edgeOnPlane(face, gp_Pnt(-30., -30., 0.), gp_Dir(0., 1., 0.),
                                      gp_Pnt2d(-30., -30.), gp_Dir2d(0., 1.));
  Handle(ShapeExtend_WireData) first = makeWire(face, firstOverlap, firstFar);
  Handle(ShapeExtend_WireData) second = makeWire(face, secondOverlap, secondFar);
  builder.Add(face, first->Wire());
  builder.Add(face, second->Wire());
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool tool(context, 1.e-7);
  bool done = tool.FixIntersectingWires(face);
  Handle(ShapeExtend_WireData) result[2];
  int wireIndex = 0;
  for (TopoDS_Iterator wires(face, false, true); wires.More(); wires.Next()) {
    if (wires.Value().ShapeType() == TopAbs_WIRE) {
      result[wireIndex++] = new ShapeExtend_WireData(TopoDS::Wire(wires.Value()));
    }
  }
  std::cout << std::setprecision(17) << done << " " << result[0]->NbEdges() << " " << result[1]->NbEdges();
  for (int wire = 0; wire < 2; ++wire) {
    for (int index = 1; index <= result[wire]->NbEdges(); ++index) {
      double firstParameter, lastParameter;
      BRep_Tool::Range(result[wire]->Edge(index), face, firstParameter, lastParameter);
      std::cout << " " << firstParameter << " " << lastParameter;
    }
  }
  std::cout << '\n';
}

static void runPartialLongOverlapCase(double firstStart,
                                      double firstDirection,
                                      double secondStart,
                                      double secondDirection = 1.) {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge firstOverlap = edgeOnPlane(face, gp_Pnt(firstStart, 0., 0.),
                                         gp_Dir(firstDirection, 0., 0.),
                                         gp_Pnt2d(firstStart, 0.),
                                         gp_Dir2d(firstDirection, 0.), 10.);
  ShapeAnalysis_Edge analysis;
  builder.UpdateVertex(analysis.FirstVertex(firstOverlap), 1.e-5);
  TopoDS_Edge secondOverlap = edgeOnPlane(face, gp_Pnt(secondStart, 0., 0.),
                                          gp_Dir(secondDirection, 0., 0.),
                                          gp_Pnt2d(secondStart, 0.),
                                          gp_Dir2d(secondDirection, 0.), 10.);
  TopoDS_Edge firstFar = edgeOnPlane(face, gp_Pnt(30., 30., 0.), gp_Dir(1., 0., 0.),
                                     gp_Pnt2d(30., 30.), gp_Dir2d(1., 0.));
  TopoDS_Edge secondFar = edgeOnPlane(face, gp_Pnt(-30., -30., 0.), gp_Dir(0., 1., 0.),
                                      gp_Pnt2d(-30., -30.), gp_Dir2d(0., 1.));
  Handle(ShapeExtend_WireData) first = makeWire(face, firstOverlap, firstFar);
  Handle(ShapeExtend_WireData) second = makeWire(face, secondOverlap, secondFar);
  builder.Add(face, first->Wire());
  builder.Add(face, second->Wire());
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool tool(context, 1.e-7);
  bool done = tool.FixIntersectingWires(face);
  Handle(ShapeExtend_WireData) result[2];
  int wireIndex = 0;
  for (TopoDS_Iterator wires(face, false, true); wires.More(); wires.Next()) {
    if (wires.Value().ShapeType() == TopAbs_WIRE) {
      result[wireIndex++] = new ShapeExtend_WireData(TopoDS::Wire(wires.Value()));
    }
  }
  std::cout << std::setprecision(17) << done << " " << result[0]->NbEdges() << " " << result[1]->NbEdges();
  for (int wire = 0; wire < 2; ++wire) {
    for (int index = 1; index <= result[wire]->NbEdges(); ++index) {
      double firstParameter, lastParameter;
      BRep_Tool::Range(result[wire]->Edge(index), face, firstParameter, lastParameter);
      std::cout << " " << firstParameter << " " << lastParameter;
    }
  }
  std::cout << '\n';
}

static void runReversedLongOverlapCase() {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge firstOverlap = edgeOnPlane(face, gp_Pnt(0., 0., 0.), gp_Dir(1., 0., 0.),
                                         gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.), 10.);
  ShapeAnalysis_Edge analysis;
  builder.UpdateVertex(analysis.FirstVertex(firstOverlap), 1.e-5);
  TopoDS_Edge secondOverlap = edgeOnPlane(face, gp_Pnt(8., 0., 0.), gp_Dir(-1., 0., 0.),
                                          gp_Pnt2d(8., 0.), gp_Dir2d(-1., 0.), 6.);
  TopoDS_Edge firstFar = edgeOnPlane(face, gp_Pnt(30., 30., 0.), gp_Dir(1., 0., 0.),
                                     gp_Pnt2d(30., 30.), gp_Dir2d(1., 0.));
  TopoDS_Edge secondFar = edgeOnPlane(face, gp_Pnt(-30., -30., 0.), gp_Dir(0., 1., 0.),
                                      gp_Pnt2d(-30., -30.), gp_Dir2d(0., 1.));
  Handle(ShapeExtend_WireData) first = makeWire(face, firstOverlap, firstFar);
  Handle(ShapeExtend_WireData) second = makeWire(face, secondOverlap, secondFar);
  builder.Add(face, first->Wire());
  builder.Add(face, second->Wire());
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool tool(context, 1.e-7);
  bool done = tool.FixIntersectingWires(face);
  Handle(ShapeExtend_WireData) result[2];
  int wireIndex = 0;
  for (TopoDS_Iterator wires(face, false, true); wires.More(); wires.Next()) {
    if (wires.Value().ShapeType() == TopAbs_WIRE) {
      result[wireIndex++] = new ShapeExtend_WireData(TopoDS::Wire(wires.Value()));
    }
  }
  std::cout << std::setprecision(17) << done << " " << result[0]->NbEdges() << " " << result[1]->NbEdges();
  for (int wire = 0; wire < 2; ++wire) {
    for (int index = 1; index <= result[wire]->NbEdges(); ++index) {
      double firstParameter, lastParameter;
      BRep_Tool::Range(result[wire]->Edge(index), face, firstParameter, lastParameter);
      std::cout << " " << firstParameter << " " << lastParameter;
    }
  }
  std::cout << '\n';
}

static void runTwoSidedEndpointCutCase() {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge firstOverlap = edgeOnPlane(face, gp_Pnt(0., 0., 0.), gp_Dir(1., 0., 0.),
                                         gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.), 1000.);
  ShapeAnalysis_Edge analysis;
  builder.UpdateVertex(analysis.FirstVertex(firstOverlap), 2.);
  TopoDS_Edge secondOverlap = edgeOnPlane(face, gp_Pnt(.5, 0., 0.), gp_Dir(1., 0., 0.),
                                          gp_Pnt2d(.5, 0.), gp_Dir2d(1., 0.), 1.);
  TopoDS_Edge firstFar = edgeOnPlane(face, gp_Pnt(3000., 3000., 0.), gp_Dir(1., 0., 0.),
                                     gp_Pnt2d(3000., 3000.), gp_Dir2d(1., 0.));
  TopoDS_Edge secondFar = edgeOnPlane(face, gp_Pnt(-3000., -3000., 0.), gp_Dir(0., 1., 0.),
                                      gp_Pnt2d(-3000., -3000.), gp_Dir2d(0., 1.));
  Handle(ShapeExtend_WireData) first = makeWire(face, firstOverlap, firstFar);
  Handle(ShapeExtend_WireData) second = makeWire(face, secondOverlap, secondFar);
  builder.Add(face, first->Wire());
  builder.Add(face, second->Wire());
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool tool(context, 1.e-7);
  bool done = tool.FixIntersectingWires(face);
  std::cout << std::setprecision(17) << done << " " << first->NbEdges() << " " << second->NbEdges();
  for (const TopoDS_Edge& edge : {first->Edge(1), second->Edge(1)}) {
    double firstParameter, lastParameter;
    BRep_Tool::Range(edge, face, firstParameter, lastParameter);
    std::cout << " " << firstParameter << " " << lastParameter;
  }
  std::cout << '\n';
}

static void runFarEndpointCutCase() {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge longEdge = edgeOnPlane(face, gp_Pnt(0., 0., 0.), gp_Dir(1., 0., 0.),
                                      gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.), 1000.);
  ShapeAnalysis_Edge analysis;
  builder.UpdateVertex(analysis.LastVertex(longEdge), 2.);
  TopoDS_Edge shortEdge = edgeOnPlane(face, gp_Pnt(999., 0., 0.), gp_Dir(1., 0., 0.),
                                       gp_Pnt2d(999., 0.), gp_Dir2d(1., 0.), 1.);
  TopoDS_Edge firstFar = edgeOnPlane(face, gp_Pnt(3000., 3000., 0.), gp_Dir(1., 0., 0.),
                                     gp_Pnt2d(3000., 3000.), gp_Dir2d(1., 0.));
  TopoDS_Edge secondFar = edgeOnPlane(face, gp_Pnt(-3000., -3000., 0.), gp_Dir(0., 1., 0.),
                                      gp_Pnt2d(-3000., -3000.), gp_Dir2d(0., 1.));
  Handle(ShapeExtend_WireData) first = makeWire(face, longEdge, firstFar);
  Handle(ShapeExtend_WireData) second = makeWire(face, shortEdge, secondFar);
  builder.Add(face, first->Wire());
  builder.Add(face, second->Wire());
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool tool(context, 1.e-7);
  bool done = tool.FixIntersectingWires(face);
  std::cout << std::setprecision(17) << done << " " << first->NbEdges() << " " << second->NbEdges();
  for (const TopoDS_Edge& edge : {first->Edge(1), second->Edge(1)}) {
    double firstParameter, lastParameter;
    BRep_Tool::Range(edge, face, firstParameter, lastParameter);
    std::cout << " " << firstParameter << " " << lastParameter;
  }
  std::cout << '\n';
}

static void runEndpointCutCase() {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge longEdge = edgeOnPlane(face, gp_Pnt(0., 0., 0.), gp_Dir(1., 0., 0.),
                                      gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.), 1000.);
  ShapeAnalysis_Edge analysis;
  builder.UpdateVertex(analysis.FirstVertex(longEdge), 2.);
  TopoDS_Edge firstFar = edgeOnPlane(face, gp_Pnt(3000., 3000., 0.), gp_Dir(1., 0., 0.),
                                     gp_Pnt2d(3000., 3000.), gp_Dir2d(1., 0.));
  TopoDS_Edge shortEdge = edgeOnPlane(face, gp_Pnt(0., 0., 0.), gp_Dir(1., 0., 0.),
                                      gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.), 1.);
  TopoDS_Edge secondFar = edgeOnPlane(face, gp_Pnt(-3000., -3000., 0.), gp_Dir(0., 1., 0.),
                                      gp_Pnt2d(-3000., -3000.), gp_Dir2d(0., 1.));
  Handle(ShapeExtend_WireData) first = makeWire(face, longEdge, firstFar);
  Handle(ShapeExtend_WireData) second = makeWire(face, shortEdge, secondFar);
  builder.Add(face, first->Wire());
  builder.Add(face, second->Wire());
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool tool(context, 1.e-7);
  bool done = tool.FixIntersectingWires(face);
  std::cout << std::setprecision(17) << done << " " << first->NbEdges() << " " << second->NbEdges();
  for (const TopoDS_Edge& edge : {first->Edge(1), second->Edge(1)}) {
    double firstParameter, lastParameter;
    BRep_Tool::Range(edge, face, firstParameter, lastParameter);
    std::cout << " " << firstParameter << " " << lastParameter;
  }
  std::cout << '\n';
}

int main() {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge firstCross = edgeOnPlane(face, gp_Pnt(-1., 0., 0.), gp_Dir(1., 0., 0.),
                                       gp_Pnt2d(-1., 0.), gp_Dir2d(1., 0.));
  TopoDS_Edge firstFar = edgeOnPlane(face, gp_Pnt(3., 3., 0.), gp_Dir(1., 0., 0.),
                                     gp_Pnt2d(3., 3.), gp_Dir2d(1., 0.));
  TopoDS_Edge secondCross = edgeOnPlane(face, gp_Pnt(0., -1., 0.), gp_Dir(0., 1., 0.),
                                        gp_Pnt2d(0., -1.), gp_Dir2d(0., 1.));
  TopoDS_Edge secondFar = edgeOnPlane(face, gp_Pnt(-4., -4., 0.), gp_Dir(0., 1., 0.),
                                      gp_Pnt2d(-4., -4.), gp_Dir2d(0., 1.));
  Handle(ShapeExtend_WireData) first = makeWire(face, firstCross, firstFar);
  Handle(ShapeExtend_WireData) second = makeWire(face, secondCross, secondFar);
  builder.Add(face, first->Wire());
  builder.Add(face, second->Wire());
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ShapeFix_IntersectionTool tool(context, 1.e-7);
  bool done = tool.FixIntersectingWires(face);
  std::cout << std::setprecision(17) << done << " " << face.NbChildren();
  for (TopoDS_Iterator wires(face, false, true); wires.More(); wires.Next()) {
    if (wires.Value().ShapeType() != TopAbs_WIRE) continue;
    const TopoDS_Wire& wire = TopoDS::Wire(wires.Value());
    int count = 0;
    for (TopoDS_Iterator edges(wire, false, true); edges.More(); edges.Next()) ++count;
    std::cout << " " << count;
    Handle(ShapeExtend_WireData) extracted = new ShapeExtend_WireData(wire);
    for (int index = 1; index <= extracted->NbEdges(); ++index) {
      double firstParameter, lastParameter;
      BRep_Tool::Range(extracted->Edge(index), face, firstParameter, lastParameter);
      std::cout << " " << firstParameter << " " << lastParameter;
    }
  }
  std::cout << " " << context->Apply(face).IsSame(face) << '\n';
  runVertexCase(true);
  runVertexCase(false);
  runShortOverlapCase();
  runLongOverlapCase();
  runContainedFirstLongOverlapCase();
  runPartialLongOverlapCase(0., 1., 2.);
  runPartialLongOverlapCase(0., 1., -2.);
  runPartialLongOverlapCase(0., 1., 8., -1.);
  runPartialLongOverlapCase(8., -1., 2.);
  runReversedLongOverlapCase();
  runTwoSidedEndpointCutCase();
  runFarEndpointCutCase();
  runEndpointCutCase();
}
