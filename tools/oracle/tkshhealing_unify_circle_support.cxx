#include <BRepAdaptor_Curve.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <ElCLib.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <Geom_Circle.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Pnt.hxx>

#include <cmath>
#include <iomanip>
#include <iostream>
#include <utility>
#include <vector>

static void run_case(const char* name, const gp_Circ& firstCircle, double firstStart,
                     double firstEnd, const gp_Circ& secondCircle, double secondStart,
                     double secondEnd) {
  const TopoDS_Edge first = BRepBuilderAPI_MakeEdge(firstCircle, firstStart, firstEnd);
  const TopoDS_Edge second = BRepBuilderAPI_MakeEdge(secondCircle, secondStart, secondEnd);
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(first);
  wireBuilder.Add(second);
  TopoDS_Shape input;
  if (!wireBuilder.IsDone()) {
    BRep_Builder builder;
    TopoDS_Compound compound;
    builder.MakeCompound(compound);
    builder.Add(compound, first);
    builder.Add(compound, second);
    input = compound;
  } else {
    input = wireBuilder.Wire();
  }
  ShapeUpgrade_UnifySameDomain unifier(input, true, false, false);
  unifier.Build();
  const TopoDS_Shape result = unifier.Shape();
  int edgeCount = 0;
  std::vector<double> radii;
  std::vector<std::pair<double, double>> ranges;
  for (TopExp_Explorer explorer(result, TopAbs_EDGE); explorer.More(); explorer.Next()) {
    ++edgeCount;
    const TopoDS_Edge edge = TopoDS::Edge(explorer.Current());
    BRepAdaptor_Curve curve(edge);
    radii.push_back(curve.Circle().Radius());
    ranges.emplace_back(curve.FirstParameter(), curve.LastParameter());
  }
  std::cout << std::setprecision(17) << name << ' ' << edgeCount;
  for (std::size_t index = 0; index < radii.size(); ++index) {
    std::cout << ' ' << radii[index] << ' ' << ranges[index].first << ' ' << ranges[index].second;
  }
  std::cout << '\n';
}

static void run_shared_case(const char* name, const gp_Circ& firstCircle,
                            const gp_Circ& secondCircle, const gp_Pnt& firstPoint,
                            const gp_Pnt& sharedPoint, const gp_Pnt& lastPoint) {
  const TopoDS_Edge first = BRepBuilderAPI_MakeEdge(firstCircle, firstPoint, sharedPoint);
  const TopoDS_Edge second = BRepBuilderAPI_MakeEdge(secondCircle, sharedPoint, lastPoint);
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(first);
  wireBuilder.Add(second);
  const TopoDS_Wire input = wireBuilder.Wire();
  ShapeUpgrade_UnifySameDomain unifier(input, true, false, false);
  unifier.Build();
  const TopoDS_Shape result = unifier.Shape();
  int edgeCount = 0;
  std::cout << std::setprecision(17) << name;
  for (TopExp_Explorer explorer(result, TopAbs_EDGE); explorer.More(); explorer.Next()) {
    ++edgeCount;
  }
  std::cout << ' ' << edgeCount;
  for (TopExp_Explorer explorer(result, TopAbs_EDGE); explorer.More(); explorer.Next()) {
    BRepAdaptor_Curve curve(TopoDS::Edge(explorer.Current()));
    std::cout << ' ' << curve.Circle().Radius() << ' ' << curve.FirstParameter() << ' '
              << curve.LastParameter();
  }
  std::cout << '\n';
}

int main() {
  const gp_Ax2 frame(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1));
  const gp_Circ firstCircle(frame, 2.0);
  const gp_Circ nearRadius(frame, 2.0 + 5e-8);
  run_case("near_radius", firstCircle, 0.0, M_PI / 2.0, nearRadius, M_PI / 2.0, M_PI);
  const gp_Circ farRadius(frame, 2.0 + 5e-7);
  run_case("far_radius", firstCircle, 0.0, M_PI / 2.0, farRadius, M_PI / 2.0, M_PI);
  const gp_Ax2 perpendicularFrame(gp_Pnt(0, 0, 0), gp_Dir(1, 0, 0), gp_Dir(0, 1, 0));
  run_shared_case("nonparallel_axes", firstCircle, gp_Circ(perpendicularFrame, 2.0),
                  gp_Pnt(2, 0, 0), gp_Pnt(0, 2, 0), gp_Pnt(0, 0, 2));
  const gp_Circ offsetCenter(gp_Ax2(gp_Pnt(0.2, 0, 0), gp_Dir(0, 0, 1)), 2.0);
  const gp_Pnt sharedPoint(0.1, std::sqrt(3.99), 0.0);
  const double sharedAngle = std::atan2(sharedPoint.Y(), -0.1);
  run_shared_case("nonconcentric_centers", firstCircle, offsetCenter,
                  ElCLib::Value(0.0, firstCircle), sharedPoint,
                  ElCLib::Value(sharedAngle + M_PI / 2.0, offsetCenter));
}
