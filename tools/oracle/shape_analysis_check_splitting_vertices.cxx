#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <Geom_Plane.hxx>
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <ShapeExtend.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pnt.hxx>
#include <NCollection_DataMap.hxx>
#include <NCollection_List.hxx>
#include <iostream>

using ShapeListMap = NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>;
using ParamListMap = NCollection_DataMap<TopoDS_Shape, NCollection_List<double>, TopTools_ShapeMapHasher>;

static TopoDS_Face make_face(const double split_y) {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1e-7);
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Wire wire = BRepBuilderAPI_MakeWire(edge);
  builder.Add(face, wire);
  if (split_y >= 0) {
    TopoDS_Vertex split;
    builder.MakeVertex(split, gp_Pnt(0.5, split_y, 0), 1e-3);
    builder.Add(face, split);
  }
  return face;
}

static void probe(const char* name, const TopoDS_Face& face) {
  ShapeAnalysis_CheckSmallFace analyzer;
  analyzer.SetTolerance(1e-3);
  ShapeListMap edges;
  ParamListMap parameters;
  TopoDS_Compound vertices;
  BRep_Builder{}.MakeCompound(vertices);
  const int count = analyzer.CheckSplittingVertices(face, edges, parameters, vertices);
  std::cout << name << ' ' << count << ' ' << edges.Extent() << ' ' << parameters.Extent() << ' '
            << analyzer.StatusSplitVert(ShapeExtend_DONE) << ' ' << vertices.NbChildren();
  if (!parameters.IsEmpty()) {
    ParamListMap::Iterator item(parameters);
    std::cout << ' ' << item.Value().First();
  } else {
    std::cout << " -1";
  }
  std::cout << '\n';
}

int main() {
  probe("split", make_face(1e-4));
  probe("remote", make_face(1.0));
  probe("no_vertices", make_face(-1.0));
}
