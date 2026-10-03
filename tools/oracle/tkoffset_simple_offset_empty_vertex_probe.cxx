// Verify FillVertexData's zero-sample branch at OCCT 3d097a0328e7.
// FillVertexData returns only when the incident-edge list is empty; free-edge entries
// have incident edges but no offset curves, so the empty centroid is still bound.
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepOffset_SimpleOffset.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pnt.hxx>

#include <cmath>
#include <iostream>

int main()
{
  const TopoDS_Edge edge =
    BRepBuilderAPI_MakeEdge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(1.0, 0.0, 0.0));
  BRepOffset_SimpleOffset offset(edge, 10.0, 1.0e-7);
  for (TopExp_Explorer vertices(edge, TopAbs_VERTEX); vertices.More(); vertices.Next())
  {
    gp_Pnt point;
    double tolerance = -1.0;
    const bool found = offset.NewPoint(TopoDS::Vertex(vertices.Current()), point, tolerance);
    std::cout << found << ' ' << std::isnan(point.X()) << ' ' << std::isnan(point.Y()) << ' '
              << std::isnan(point.Z()) << ' ' << tolerance << '\n';
  }
}
