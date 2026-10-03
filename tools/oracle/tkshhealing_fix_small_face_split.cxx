#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRep_Tool.hxx>
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <ShapeFix_Root.hxx>
#define private public
#include <ShapeFix_FixSmallFace.hxx>
#undef private
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <NCollection_DataMap.hxx>
#include <NCollection_List.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <gp_Pln.hxx>

#include <iostream>

static int count(const TopoDS_Shape& shape, TopAbs_ShapeEnum kind) {
  int count = 0;
  for (TopExp_Explorer it(shape, kind); it.More(); it.Next()) ++count;
  return count;
}

static int count_faces(const TopoDS_Shape& shape) {
  return count(shape, TopAbs_FACE);
}

static TopoDS_Face near_boundary_vertex_face() {
  BRep_Builder builder;
  TopoDS_Vertex a, b, c;
  builder.MakeVertex(a, gp_Pnt(0.0, 0.0, 0.0), 1.0e-4);
  builder.MakeVertex(b, gp_Pnt(1.0, 0.0, 0.0), 1.0e-4);
  builder.MakeVertex(c, gp_Pnt(0.5, 1.0e-5, 0.0), 1.0e-4);
  const TopoDS_Edge ab = BRepBuilderAPI_MakeEdge(a, b).Edge();
  const TopoDS_Edge bc = BRepBuilderAPI_MakeEdge(b, c).Edge();
  const TopoDS_Edge ca = BRepBuilderAPI_MakeEdge(c, a).Edge();
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, ab);
  builder.Add(wire, bc);
  builder.Add(wire, ca);
  return BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), wire, true).Face();
}

static void probe_fix_split_face() {
  TopoDS_Face face = near_boundary_vertex_face();
  ShapeFix_FixSmallFace fixer;
  fixer.Init(face);
  fixer.myAnalyzer.SetTolerance(1.0e-4);
  const TopoDS_Shape result = fixer.FixSplitFace(face);
  std::cout << "aggregate " << count_faces(result) << ' ' << result.ShapeType() << ' '
            << fixer.myStatus << ' ' << count(result, TopAbs_EDGE) << ' '
            << count(result, TopAbs_VERTEX) << '\n';
}

int main() {
  TopoDS_Face face = near_boundary_vertex_face();
  TopoDS_Compound split_faces;
  BRep_Builder builder;
  builder.MakeCompound(split_faces);
  ShapeFix_FixSmallFace fixer;
  fixer.Init(face);
  fixer.myAnalyzer.SetTolerance(1.0e-4);
  NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher> edges;
  NCollection_DataMap<TopoDS_Shape, NCollection_List<double>, TopTools_ShapeMapHasher> parameters;
  TopoDS_Compound found_vertices;
  builder.MakeCompound(found_vertices);
  const int found = fixer.myAnalyzer.CheckSplittingVertices(face, edges, parameters, found_vertices);
  std::cout << "detected " << found << ' ' << edges.Extent() << '\n';
  const bool split = fixer.SplitOneFace(face, split_faces);
  std::cout << split << ' ' << count_faces(split_faces) << ' '
            << count(split_faces, TopAbs_EDGE) << ' ' << count(split_faces, TopAbs_VERTEX) << '\n';
  probe_fix_split_face();
}
