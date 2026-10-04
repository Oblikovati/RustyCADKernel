#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepTools_ReShape.hxx>
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <NCollection_Array1.hxx>

#include <algorithm>
#include <iomanip>
#include <iostream>
#include <utility>
#include <vector>

using EdgePair = std::pair<TopoDS_Edge, TopoDS_Edge>;

static TopoDS_Edge seam(const TopoDS_Face& face)
{
  for (TopExp_Explorer edges(face, TopAbs_EDGE); edges.More(); edges.Next())
  {
    const auto edge = TopoDS::Edge(edges.Current());
    if (BRep_Tool::IsClosed(edge, face))
      return edge;
  }
  return {};
}

static void addOccurrence(BRep_Builder& builder, TopoDS_Wire& wire,
                          const TopoDS_Edge& current, const TopoDS_Edge& seam,
                          const EdgePair& parts)
{
  if (!current.IsSame(seam))
  {
    builder.Add(wire, current);
    return;
  }
  TopoDS_Edge first = parts.first, second = parts.second;
  if (current.Orientation() == TopAbs_REVERSED)
  {
    first = parts.second;
    second = parts.first;
    first.Reverse();
    second.Reverse();
  }
  builder.Add(wire, first);
  builder.Add(wire, second);
}

static TopoDS_Wire splitWire(const TopoDS_Face& face, const TopoDS_Edge& seam,
                             const EdgePair& parts)
{
  TopoDS_Wire wire;
  BRep_Builder builder;
  builder.MakeWire(wire);
  for (BRepTools_WireExplorer edges(BRepTools::OuterWire(face), face); edges.More(); edges.Next())
    addOccurrence(builder, wire, edges.Current(), seam, parts);
  return wire;
}

static void addPcurve(const TopoDS_Edge& edge, const TopoDS_Edge& seam,
                      const TopoDS_Face& face, double first, double last)
{
  double f = 0.0, l = 0.0, rf = 0.0, rl = 0.0;
  const auto pcurve1 = BRep_Tool::CurveOnSurface(seam, face, f, l);
  TopoDS_Edge reversed = seam;
  reversed.Reverse();
  const auto pcurve2 = BRep_Tool::CurveOnSurface(reversed, face, rf, rl);
  TopLoc_Location location;
  const auto surface = BRep_Tool::Surface(face, location);
  NCollection_Array1<gp_Pnt2d> firstPoles(1, 2), secondPoles(1, 2);
  firstPoles(1) = pcurve1->Value(first);
  firstPoles(2) = pcurve1->Value(last);
  secondPoles(1) = pcurve2->Value(first);
  secondPoles(2) = pcurve2->Value(last);
  NCollection_Array1<double> knots(1, 2);
  knots(1) = first;
  knots(2) = last;
  NCollection_Array1<int> mults(1, 2);
  mults(1) = 2;
  mults(2) = 2;
  const auto firstPart = new Geom2d_BSplineCurve(firstPoles, knots, mults, 1);
  const auto secondPart = new Geom2d_BSplineCurve(secondPoles, knots, mults, 1);
  BRep_Builder().UpdateEdge(edge, firstPart, secondPart, surface, location,
                            BRep_Tool::Tolerance(seam));
  BRep_Builder().Range(edge, first, last);
}

static EdgePair split(const TopoDS_Edge& seam)
{
  TopoDS_Edge forward = seam;
  forward.Orientation(TopAbs_FORWARD);
  TopLoc_Location location;
  double first = 0.0, last = 0.0;
  const auto curve = BRep_Tool::Curve(forward, location, first, last);
  TopoDS_Vertex start, finish;
  TopExp::Vertices(forward, start, finish, false);
  const double middleParameter = (first + last) / 2.0;
  const auto middle = BRepBuilderAPI_MakeVertex(curve->Value(middleParameter)).Vertex();
  const auto lower = BRepBuilderAPI_MakeEdge(curve, start, middle, first, middleParameter).Edge();
  const auto upper = BRepBuilderAPI_MakeEdge(curve, middle, finish, middleParameter, last).Edge();
  return {lower, upper};
}

static TopoDS_Face faceWithSplitSeam(const TopoDS_Face& source, const TopoDS_Edge& seam,
                                     const EdgePair& parts)
{
  TopLoc_Location location;
  const auto surface = BRep_Tool::Surface(source, location);
  TopoDS_Face face;
  BRep_Builder builder;
  builder.MakeFace(face, surface, location, BRep_Tool::Tolerance(source));
  builder.Add(face, splitWire(source, seam, parts));
  return face;
}

static TopoDS_Face shareSeamVertices(const TopoDS_Face& face, const TopoDS_Edge& seam,
                                     const EdgePair& parts)
{
  TopoDS_Vertex oldStart, oldEnd, newStart, ignored, ignored2, newEnd;
  TopExp::Vertices(seam, oldStart, oldEnd, false);
  TopExp::Vertices(parts.first, newStart, ignored, false);
  TopExp::Vertices(parts.second, ignored2, newEnd, false);
  Handle(BRepTools_ReShape) reshape = new BRepTools_ReShape;
  reshape->Replace(oldStart, newStart);
  reshape->Replace(oldEnd, newEnd);
  return TopoDS::Face(reshape->Apply(face, TopAbs_VERTEX));
}

static std::vector<TopoDS_Edge> uniqueEdges(const TopoDS_Shape& shape)
{
  std::vector<TopoDS_Edge> result;
  for (TopExp_Explorer edges(shape, TopAbs_EDGE); edges.More(); edges.Next())
  {
    const auto edge = TopoDS::Edge(edges.Current());
    const auto found = std::find_if(result.begin(), result.end(), [&](const auto& known) {
      return known.IsSame(edge);
    });
    if (found == result.end())
      result.push_back(edge);
  }
  return result;
}

static void emitSeamOnFace(const TopoDS_Edge& edge, const TopoDS_Face& face, int faceIndex);

static void emitSeams(const TopoDS_Shape& shape)
{
  for (const auto& edge : uniqueEdges(shape))
  {
    int faceIndex = 0;
    for (TopExp_Explorer faces(shape, TopAbs_FACE); faces.More(); faces.Next(), ++faceIndex)
      emitSeamOnFace(edge, TopoDS::Face(faces.Current()), faceIndex);
  }
}

static int countFaceBSplinePCurves(const TopoDS_Edge& edge, const TopoDS_Face& face);

static int countInputBSplinePCurves(const TopoDS_Shape& shape)
{
  int count = 0;
  for (const auto& edge : uniqueEdges(shape))
  {
    for (TopExp_Explorer faces(shape, TopAbs_FACE); faces.More(); faces.Next())
      count += countFaceBSplinePCurves(edge, TopoDS::Face(faces.Current()));
  }
  return count;
}

static int countFaceBSplinePCurves(const TopoDS_Edge& edge, const TopoDS_Face& face)
{
  if (!BRep_Tool::IsClosed(edge, face))
    return 0;
  double first = 0.0, last = 0.0;
  const auto pcurve1 = BRep_Tool::CurveOnSurface(edge, face, first, last);
  TopoDS_Edge reversed = edge;
  reversed.Reverse();
  const auto pcurve2 = BRep_Tool::CurveOnSurface(reversed, face, first, last);
  return (pcurve1->IsKind(STANDARD_TYPE(Geom2d_BSplineCurve)) ? 1 : 0)
         + (pcurve2->IsKind(STANDARD_TYPE(Geom2d_BSplineCurve)) ? 1 : 0);
}

static void emitObservation(int faceIndex, double axisX, double first, double last,
                            const occ::handle<Geom2d_Curve>& pcurve1, double reverseFirst,
                            double reverseLast, const occ::handle<Geom2d_Curve>& pcurve2);

static void emitSeamOnFace(const TopoDS_Edge& edge, const TopoDS_Face& face, int faceIndex)
{
  if (!BRep_Tool::IsClosed(edge, face))
    return;
  double first = 0.0, last = 0.0, reverseFirst = 0.0, reverseLast = 0.0;
  const auto pcurve1 = BRep_Tool::CurveOnSurface(edge, face, first, last);
  TopoDS_Edge reversed = edge;
  reversed.Reverse();
  const auto pcurve2 = BRep_Tool::CurveOnSurface(reversed, face, reverseFirst, reverseLast);
  BRepAdaptor_Surface adaptor(face, false);
  emitObservation(faceIndex, adaptor.Cylinder().Axis().Location().X(), first, last, pcurve1,
                  reverseFirst, reverseLast, pcurve2);
}

static void emitObservation(int faceIndex, double axisX, double first, double last,
                            const occ::handle<Geom2d_Curve>& pcurve1, double reverseFirst,
                            double reverseLast, const occ::handle<Geom2d_Curve>& pcurve2)
{
  const gp_Pnt2d forwardStart = pcurve1->Value(first), forwardEnd = pcurve1->Value(last);
  const gp_Pnt2d reverseStart = pcurve2->Value(reverseFirst), reverseEnd = pcurve2->Value(reverseLast);
  std::cout << "support " << faceIndex << " axis_x " << axisX << " seam " << first << ' ' << last
            << ' ' << forwardStart.X() << ' ' << forwardStart.Y() << ' ' << forwardEnd.X() << ' '
            << forwardEnd.Y() << ' ' << reverseStart.X() << ' ' << reverseStart.Y() << ' '
            << reverseEnd.X() << ' ' << reverseEnd.Y() << " types "
            << pcurve1->DynamicType()->Name() << ' ' << pcurve2->DynamicType()->Name() << '\n';
}

static TopoDS_Shell buildTangentCylinderShell()
{
  const auto firstFace = BRepPrimAPI_MakeCylinder(2.0, 5.0).Face();
  const gp_Ax2 secondAxes(gp_Pnt(4.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0),
                          gp_Dir(-1.0, 0.0, 0.0));
  const auto secondFace = BRepPrimAPI_MakeCylinder(secondAxes, 2.0, 5.0).Face();
  const auto firstSeam = seam(firstFace), secondSeam = seam(secondFace);
  const auto parts = split(firstSeam);
  addPcurve(parts.first, firstSeam, firstFace, 0.0, 2.5);
  addPcurve(parts.second, firstSeam, firstFace, 2.5, 5.0);
  addPcurve(parts.first, secondSeam, secondFace, 0.0, 2.5);
  addPcurve(parts.second, secondSeam, secondFace, 2.5, 5.0);
  const auto first = faceWithSplitSeam(firstFace, firstSeam, parts);
  const auto second = shareSeamVertices(faceWithSplitSeam(secondFace, secondSeam, parts),
                                        secondSeam, parts);
  TopoDS_Shell shell;
  BRep_Builder builder;
  builder.MakeShell(shell);
  builder.Add(shell, first);
  builder.Add(shell, second);
  return shell;
}

int main()
{
  std::cout << std::setprecision(17);
  const auto input = buildTangentCylinderShell();
  std::cout << "input_valid " << BRepCheck_Analyzer(input, true).IsValid() << " input_faces 2 input_edges "
            << uniqueEdges(input).size() << " input_bspline_pcurves "
            << countInputBSplinePCurves(input) << '\n';
  ShapeUpgrade_UnifySameDomain unifier(input, true, false, true);
  unifier.Build();
  const auto result = unifier.Shape();
  int faceCount = 0;
  for (TopExp_Explorer faces(result, TopAbs_FACE); faces.More(); faces.Next())
    ++faceCount;
  std::cout << "result_valid " << BRepCheck_Analyzer(result, true).IsValid() << " result_faces "
            << faceCount << " result_edges " << uniqueEdges(result).size() << '\n';
  emitSeams(result);
}
