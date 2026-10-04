#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeFix_Wireframe.hxx>
#include <NCollection_DataMap.hxx>
#include <NCollection_List.hxx>
#include <NCollection_Map.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pln.hxx>

#include <iostream>

static TopoDS_Wire tiny_wire() {
  return BRepBuilderAPI_MakePolygon(gp_Pnt(0, 0, 0), gp_Pnt(0.01, 0, 0),
                                    gp_Pnt(1, 0, 0), gp_Pnt(1, 1, 0), true)
      .Wire();
}

static void report_join(bool reverse_second) {
  const TopoDS_Wire wire = tiny_wire();
  const TopoDS_Face first = BRepBuilderAPI_MakeFace(gp_Pln(), wire).Face();
  TopoDS_Face second = BRepBuilderAPI_MakeFace(gp_Pln(), wire).Face();
  if (reverse_second) {
    second.Reverse();
  }
  BRep_Builder builder;
  TopoDS_Compound compound;
  builder.MakeCompound(compound);
  builder.Add(compound, first);
  builder.Add(compound, second);
  ShapeFix_Wireframe fixer(compound);
  fixer.SetPrecision(0.1);
  fixer.SetContext(new ShapeBuild_ReShape);
  NCollection_Map<TopoDS_Shape, TopTools_ShapeMapHasher> small, multiple;
  NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>, TopTools_ShapeMapHasher>
      edge_faces, face_small;
  TopExp_Explorer edges(wire, TopAbs_EDGE);
  const TopoDS_Edge small_edge = TopoDS::Edge(edges.Current());
  for (; edges.More(); edges.Next()) {
    NCollection_List<TopoDS_Shape> faces;
    faces.Append(first);
    faces.Append(second);
    edge_faces.Bind(edges.Current(), faces);
  }
  small.Add(small_edge);
  NCollection_List<TopoDS_Shape> face_edges;
  face_edges.Append(small_edge);
  face_small.Bind(first, face_edges);
  const bool changed = fixer.MergeSmallEdges(small, edge_faces, face_small, multiple, false, -1.);
  const TopoDS_Shape result = fixer.Context()->Apply(first);
  int second_face_pcurves = 0;
  int edge_count = 0;
  for (TopExp_Explorer it(result, TopAbs_EDGE); it.More(); it.Next()) {
    ++edge_count;
    double first_parameter = 0., last_parameter = 0.;
    if (!BRep_Tool::CurveOnSurface(TopoDS::Edge(it.Current()), second,
                                   first_parameter, last_parameter).IsNull()) {
      ++second_face_pcurves;
    }
  }
  const char* label = reverse_second ? "shared_face_reversed" : "shared_face_forward";
  std::cout << label << ' ' << changed << ' ' << edge_count << ' '
            << second_face_pcurves << ' ' << fixer.StatusSmallEdges(ShapeExtend_DONE) << ' '
            << fixer.StatusSmallEdges(ShapeExtend_FAIL) << '\n';
}

int main() {
  report_join(false);
  report_join(true);
}
