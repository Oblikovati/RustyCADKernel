#include <BRepAdaptor_Curve.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRep_Tool.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Pnt.hxx>

#include <iomanip>
#include <iostream>

int main() {
  const gp_Ax2 frame(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1));
  const gp_Circ firstCircle(frame, 2.0);
  const gp_Circ secondCircle(frame, 2.0 + 5e-8);
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(firstCircle, 0.0, M_PI / 2.0));
  wireBuilder.Add(BRepBuilderAPI_MakeEdge(secondCircle, M_PI / 2.0, M_PI));
  TopoDS_Wire input = wireBuilder.Wire();
  ShapeUpgrade_UnifySameDomain unifier(input, true, false, false);
  unifier.Build();
  const TopoDS_Shape result = unifier.Shape();
  int edgeCount = 0;
  double radius = 0.0;
  double first = 0.0;
  double last = 0.0;
  for (TopExp_Explorer explorer(result, TopAbs_EDGE); explorer.More(); explorer.Next()) {
    ++edgeCount;
    const TopoDS_Edge edge = TopoDS::Edge(explorer.Current());
    BRepAdaptor_Curve curve(edge);
    radius = curve.Circle().Radius();
    first = curve.FirstParameter();
    last = curve.LastParameter();
  }
  std::cout << std::setprecision(17) << edgeCount << ' ' << radius << ' ' << first << ' ' << last
            << '\n';
}
