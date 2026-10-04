#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRep_Tool.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <NCollection_IndexedDataMap.hxx>
#include <NCollection_List.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pln.hxx>

#include <iostream>
#include <utility>
#include <vector>

static TopoDS_Wire makeLineChain(int segmentCount = 2) {
  BRepBuilderAPI_MakeWire wire;
  for (int index = 0; index < segmentCount; ++index) {
    wire.Add(BRepBuilderAPI_MakeEdge(gp_Pnt(index, 0, 0), gp_Pnt(index + 1, 0, 0)));
  }
  return wire.Wire();
}

static int edgeCount(const TopoDS_Shape& shape) {
  int count = 0;
  for (TopExp_Explorer edge(shape, TopAbs_EDGE); edge.More(); edge.Next()) ++count;
  return count;
}

static std::vector<TopoDS_Edge> wireEdges(const TopoDS_Wire& wire) {
  std::vector<TopoDS_Edge> edges;
  TopExp_Explorer explorer(wire, TopAbs_EDGE);
  for (; explorer.More(); explorer.Next()) edges.push_back(TopoDS::Edge(explorer.Current()));
  return edges;
}

static void keepTwoJunctions(ShapeUpgrade_UnifySameDomain& unify,
                             const std::vector<TopoDS_Edge>& edges) {
  NCollection_Map<TopoDS_Shape, TopTools_ShapeMapHasher> kept;
  for (int index = 0; index < 2; ++index) {
    TopoDS_Vertex junction;
    TopExp::CommonVertex(edges[index], edges[index + 1], junction);
    kept.Add(junction);
  }
  unify.KeepShapes(kept);
}

static void runKeptVertex(bool usePlural) {
  const TopoDS_Wire wire = makeLineChain(usePlural ? 3 : 2);
  const auto edges = wireEdges(wire);
  TopoDS_Vertex junction;
  TopExp::CommonVertex(edges[0], edges[1], junction);
  ShapeUpgrade_UnifySameDomain unify(wire, true, false, false);
  if (usePlural) keepTwoJunctions(unify, edges);
  else unify.KeepShape(junction);
  unify.Build();
  std::cout << (usePlural ? "keep_shapes_vertex" : "keep_shape_vertex") << " edges "
            << edgeCount(unify.Shape()) << '\n';
}

static std::pair<TopoDS_Edge, TopoDS_Edge> lineChainEdges(const TopoDS_Wire& wire) {
  TopExp_Explorer edges(wire, TopAbs_EDGE);
  const TopoDS_Edge first = TopoDS::Edge(edges.Current());
  edges.Next();
  return {first, TopoDS::Edge(edges.Current())};
}

static std::pair<bool, bool> outputEndpointIdentity(const TopoDS_Shape& shape,
                                                   const TopoDS_Vertex& inputFirst,
                                                   const TopoDS_Vertex& inputLast) {
  const TopoDS_Edge output = TopoDS::Edge(TopExp_Explorer(shape, TopAbs_EDGE).Current());
  TopoDS_Vertex outputFirst, outputLast;
  TopExp::Vertices(output, outputFirst, outputLast, true);
  return {outputFirst.IsSame(inputFirst), outputLast.IsSame(inputLast)};
}

static std::pair<TopoDS_Vertex, TopoDS_Vertex> inputEndpoints(const TopoDS_Edge& first,
                                                             const TopoDS_Edge& second) {
  TopoDS_Vertex inputFirst, ignored, ignoredAgain, inputLast;
  TopExp::Vertices(first, inputFirst, ignored, true);
  TopExp::Vertices(second, ignoredAgain, inputLast, true);
  return {inputFirst, inputLast};
}

static void runSafeInput(bool safe) {
  const TopoDS_Wire wire = makeLineChain();
  const auto [first, second] = lineChainEdges(wire);
  const auto [inputFirst, inputLast] = inputEndpoints(first, second);
  ShapeUpgrade_UnifySameDomain unify(wire, true, false, false);
  unify.SetSafeInputMode(safe);
  unify.Build();
  const auto identities = outputEndpointIdentity(unify.Shape(), inputFirst, inputLast);
  std::cout << (safe ? "safe_input" : "unsafe_input") << " endpoint_identity "
            << identities.first << ' ' << identities.second << '\n';
}

static TopoDS_Shape sewAdjacentFaces(TopoDS_Face& left) {
  const gp_Pln plane(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1));
  left = BRepBuilderAPI_MakeFace(plane, 0, 1, 0, 1);
  const TopoDS_Face right = BRepBuilderAPI_MakeFace(plane, 1, 2, 0, 1);
  BRepBuilderAPI_Sewing sewing;
  sewing.Add(left);
  sewing.Add(right);
  sewing.Perform();
  return sewing.SewedShape();
}

static TopoDS_Edge sharedEdge(const TopoDS_Shape& shape) {
  NCollection_IndexedDataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>
      edgeFaces;
  TopExp::MapShapesAndAncestors(shape, TopAbs_EDGE, TopAbs_FACE, edgeFaces);
  for (int i = 1; i <= edgeFaces.Extent(); ++i) {
    if (edgeFaces.FindFromIndex(i).Extent() == 2) {
      return TopoDS::Edge(edgeFaces.FindKey(i));
    }
  }
  return TopoDS_Edge();
}

static int faceCount(const TopoDS_Shape& shape) {
  int count = 0;
  for (TopExp_Explorer face(shape, TopAbs_FACE); face.More(); face.Next()) ++count;
  return count;
}

static void runIgnoredFace(const TopoDS_Face& left, const TopoDS_Shape& sewed);

static void runKeptFaceEdge() {
  TopoDS_Face left;
  const TopoDS_Shape sewed = sewAdjacentFaces(left);
  ShapeUpgrade_UnifySameDomain unify(sewed, false, true, false);
  unify.KeepShape(sharedEdge(sewed));
  unify.Build();
  std::cout << "keep_shape_edge faces " << faceCount(unify.Shape()) << '\n';
  runIgnoredFace(left, sewed);
}

static void runIgnoredFace(const TopoDS_Face& left, const TopoDS_Shape& sewed) {
  ShapeUpgrade_UnifySameDomain ignoreFace(sewed, false, true, false);
  ignoreFace.KeepShape(left);
  ignoreFace.Build();
  std::cout << "keep_shape_face_ignored faces " << faceCount(ignoreFace.Shape()) << '\n';
}

static void runInitializeControls() {
  const TopoDS_Wire wire = makeLineChain();
  const auto [first, second] = lineChainEdges(wire);
  TopoDS_Vertex junction;
  TopExp::CommonVertex(first, second, junction);
  ShapeUpgrade_UnifySameDomain unify(wire, true, false, false);
  unify.KeepShape(junction);
  unify.SetSafeInputMode(false);
  unify.Initialize(wire, true, false, false);
  unify.Build();
  const auto [inputFirst, inputLast] = inputEndpoints(first, second);
  const auto identities = outputEndpointIdentity(unify.Shape(), inputFirst, inputLast);
  std::cout << "initialize_resets_kept_and_preserves_safe_mode edges " << edgeCount(unify.Shape())
            << " identities " << identities.first << ' ' << identities.second << '\n';
}

int main() {
  runKeptVertex(false);
  runKeptVertex(true);
  runSafeInput(true);
  runSafeInput(false);
  runKeptFaceEdge();
  runInitializeControls();
}
