#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>

#include <algorithm>
#include <iomanip>
#include <iostream>
#include <vector>

using EdgePair = std::pair<TopoDS_Edge, TopoDS_Edge>;

static void storeSplitPCurves(const TopoDS_Edge& result, const TopoDS_Edge& source,
                              const TopoDS_Face& face, double first, double last,
                              TopLoc_Location& surfaceLocation);
static void addReplacedOccurrence(BRep_Builder& builder, TopoDS_Wire& wire,
                                  const TopoDS_Edge& current, const TopoDS_Edge& seam,
                                  const EdgePair& segments);

static TopoDS_Edge splitEdge(const TopoDS_Edge& edge, const TopoDS_Face& face,
                             const TopoDS_Vertex& middle, double first, double last,
                             double partFirst, double partLast, bool upper)
{
  TopLoc_Location curveLocation, surfaceLocation;
  double actualFirst = 0.0, actualLast = 0.0;
  const auto curve = BRep_Tool::Curve(edge, curveLocation, actualFirst, actualLast);
  TopoDS_Vertex start, finish;
  TopExp::Vertices(edge, start, finish, false);
  const auto result = upper ? BRepBuilderAPI_MakeEdge(curve, middle, finish, first, last).Edge()
                            : BRepBuilderAPI_MakeEdge(curve, start, middle, first, last).Edge();
  storeSplitPCurves(result, edge, face, partFirst, partLast, surfaceLocation);
  return result;
}

static void storeSplitPCurves(const TopoDS_Edge& result, const TopoDS_Edge& source,
                              const TopoDS_Face& face, double first, double last,
                              TopLoc_Location& surfaceLocation)
{
  double f = 0.0, l = 0.0, rf = 0.0, rl = 0.0;
  const auto curve1 = BRep_Tool::CurveOnSurface(source, face, f, l);
  TopoDS_Edge reversed = source;
  reversed.Reverse();
  const auto curve2 = BRep_Tool::CurveOnSurface(reversed, face, rf, rl);
  const auto surface = BRep_Tool::Surface(face, surfaceLocation);
  const auto trimmed1 = new Geom2d_TrimmedCurve(curve1, first, last);
  const auto trimmed2 = new Geom2d_TrimmedCurve(curve2, first, last);
  BRep_Builder().UpdateEdge(result, trimmed1, trimmed2, surface, surfaceLocation,
                            BRep_Tool::Tolerance(source));
}

static EdgePair splitSeam(const TopoDS_Edge& source, const TopoDS_Face& face)
{
  TopoDS_Edge edge = source;
  edge.Orientation(TopAbs_FORWARD);
  TopLoc_Location location;
  double first = 0.0, last = 0.0;
  const auto curve = BRep_Tool::Curve(edge, location, first, last);
  const double middleParameter = (first + last) * 0.5;
  const auto middle = BRepBuilderAPI_MakeVertex(curve->Value(middleParameter)).Vertex();
  return {splitEdge(edge, face, middle, first, middleParameter, first, middleParameter, false),
          splitEdge(edge, face, middle, middleParameter, last, middleParameter, last, true)};
}

static TopoDS_Wire splitWire(const TopoDS_Face& face, const TopoDS_Edge& seam,
                             const EdgePair& segments)
{
  TopoDS_Wire result;
  BRep_Builder builder;
  builder.MakeWire(result);
  const TopoDS_Wire source = BRepTools::OuterWire(face);
  for (BRepTools_WireExplorer edges(source, face); edges.More(); edges.Next())
    addReplacedOccurrence(builder, result, edges.Current(), seam, segments);
  return result;
}

static void addReplacedOccurrence(BRep_Builder& builder, TopoDS_Wire& wire,
                                  const TopoDS_Edge& current, const TopoDS_Edge& seam,
                                  const EdgePair& segments)
{
  if (!current.IsSame(seam))
  {
    builder.Add(wire, current);
    return;
  }
  TopoDS_Edge first = segments.first, second = segments.second;
  if (current.Orientation() == TopAbs_REVERSED)
  {
    first = segments.second;
    second = segments.first;
    first.Reverse();
    second.Reverse();
  }
  builder.Add(wire, first);
  builder.Add(wire, second);
}

static TopoDS_Face splitSeamFace(const TopoDS_Face& face)
{
  TopoDS_Edge seam;
  for (TopExp_Explorer edges(face, TopAbs_EDGE); edges.More(); edges.Next())
    if (BRep_Tool::IsClosed(TopoDS::Edge(edges.Current()), face))
      seam = TopoDS::Edge(edges.Current());
  const EdgePair segments = splitSeam(seam, face);
  TopLoc_Location location;
  const auto surface = BRep_Tool::Surface(face, location);
  TopoDS_Face result;
  BRep_Builder builder;
  builder.MakeFace(result, surface, location, BRep_Tool::Tolerance(face));
  builder.Add(result, splitWire(face, seam, segments));
  return result;
}

static std::vector<TopoDS_Edge> uniqueEdges(const TopoDS_Shape& shape)
{
  std::vector<TopoDS_Edge> result;
  for (TopExp_Explorer edges(shape, TopAbs_EDGE); edges.More(); edges.Next())
  {
    const auto edge = TopoDS::Edge(edges.Current());
    if (std::none_of(result.begin(), result.end(), [&](const auto& known) { return known.IsSame(edge); }))
      result.push_back(edge);
  }
  return result;
}

static void emitResultSeams(const TopoDS_Face& face,
                            const std::vector<TopoDS_Edge>& edges)
{
  for (const auto& edge : edges)
  {
    if (!BRep_Tool::IsClosed(edge, face))
      continue;
    double first = 0.0, last = 0.0, reverseFirst = 0.0, reverseLast = 0.0;
    const auto pcurve1 = BRep_Tool::CurveOnSurface(edge, face, first, last);
    TopoDS_Edge reversed = edge;
    reversed.Reverse();
    const auto pcurve2 = BRep_Tool::CurveOnSurface(reversed, face, reverseFirst, reverseLast);
    std::cout << "seam " << first << ' ' << last << ' ' << pcurve1->Value(first).X() << ' '
              << pcurve1->Value(first).Y() << ' ' << pcurve1->Value(last).X() << ' '
              << pcurve1->Value(last).Y() << ' ' << pcurve2->Value(reverseFirst).X() << ' '
              << pcurve2->Value(reverseFirst).Y() << ' ' << pcurve2->Value(reverseLast).X() << ' '
              << pcurve2->Value(reverseLast).Y() << '\n';
  }
}

int main()
{
  const TopoDS_Face input = splitSeamFace(BRepPrimAPI_MakeCylinder(2.0, 5.0).Face());
  ShapeUpgrade_UnifySameDomain unifier(input, true, false, true);
  unifier.Build();
  const auto result = unifier.Shape();
  const auto edges = uniqueEdges(result);
  std::cout << std::setprecision(17) << "input_valid " << BRepCheck_Analyzer(input, true).IsValid()
            << " input_edges " << uniqueEdges(input).size() << " result_valid "
            << BRepCheck_Analyzer(result, true).IsValid() << " result_edges " << edges.size() << '\n';
  emitResultSeams(TopoDS::Face(result), edges);
}
