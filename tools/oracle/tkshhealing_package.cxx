#include <ShapeFix.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <Geom_Circle.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Circ.hxx>
#include <iostream>
#include <iomanip>
static void observe(const char* name, TopoDS_Shape shape) {
  std::cout << name << ' ' << std::setprecision(17) << ShapeFix::LeastEdgeSize(shape) << '\n';
}
static void regularity() {
  TopoDS_Shape cylinder = BRepPrimAPI_MakeCylinder(2., 5.).Shape();
  ShapeFix::EncodeRegularity(cylinder, 1e-10);
  int count = 0;
  for (TopExp_Explorer it(cylinder, TopAbs_EDGE); it.More(); it.Next()) {
    if (BRep_Tool::HasContinuity(TopoDS::Edge(it.Current()))) ++count;
  }
  std::cout << "regularity " << count << '\n';
}
int main() {
  BRep_Builder builder;
  TopoDS_Compound empty; builder.MakeCompound(empty); observe("empty", empty);
  TopoDS_Edge noCurve; builder.MakeEdge(noCurve); observe("no_curve", noCurve);
  observe("line", BRepBuilderAPI_MakeEdge(gp_Pnt(1.,2.,3.), gp_Pnt(4.,6.,3.)));
  observe("circle", BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(), 2.)));
  observe("box", BRepPrimAPI_MakeBox(2.,3.,4.).Shape());
  regularity();
}
