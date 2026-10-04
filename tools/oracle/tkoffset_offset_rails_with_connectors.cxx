#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepOffset_Offset.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>

int main() {
  const auto path = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(0, 0, 10)).Edge();
  const auto rail1 = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), gp_Pnt(1, 0, 10)).Edge();
  const auto rail2 = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 1, 0), gp_Pnt(0, 1, 10)).Edge();
  TopoDS_Vertex first1, last1, first2, last2;
  TopExp::Vertices(rail1, first1, last1);
  TopExp::Vertices(rail2, first2, last2);
  const auto first = BRepBuilderAPI_MakeEdge(first2, first1).Edge();
  const auto last = BRepBuilderAPI_MakeEdge(last1, last2).Edge();
  const BRepOffset_Offset offset(path, rail1, rail2, 0.5, first, last, false, 1.e-4, GeomAbs_C1);
  const auto face = offset.Face();
  TopoDS_Vertex path_first, path_last;
  TopExp::Vertices(path, path_first, path_last);
  const auto first_image = TopoDS::Edge(offset.Generated(path_first));
  const auto last_image = TopoDS::Edge(offset.Generated(path_last));
  int edges = 0;
  bool contains_first = false;
  bool contains_last = false;
  int curves = 0;
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next()) {
    const auto edge = TopoDS::Edge(it.Current());
    ++edges;
    contains_first |= edge.IsSame(first);
    contains_last |= edge.IsSame(last);
    TopLoc_Location location;
    double u1 = 0.0;
    double u2 = 0.0;
    curves += !BRep_Tool::Curve(edge, location, u1, u2).IsNull();
  }
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(face, properties);
  std::cout << std::setprecision(17) << "status " << offset.Status() << " valid "
            << BRepCheck_Analyzer(face, true).IsValid() << " edges " << edges
            << " curves " << curves << " area " << properties.Mass()
            << " first " << first_image.IsSame(first) << " last " << last_image.IsSame(last)
            << " contains " << contains_first << " " << contains_last << "\n";
}
