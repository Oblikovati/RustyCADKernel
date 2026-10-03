#include <Geom_BezierCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Line.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Line.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_HSequence.hxx>
#include <ShapeExtend_Status.hxx>
#include <ShapeUpgrade_ConvertCurve2dToBezier.hxx>
#include <ShapeUpgrade_ConvertCurve3dToBezier.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Vec2d.hxx>
#include <iomanip>
#include <iostream>
#include <string>

static void printSeq(const char* name, const occ::handle<NCollection_HSequence<double>>& seq) {
  std::cout << name << ' ';
  for (int i = 1; i <= seq->Length(); ++i) std::cout << std::setprecision(17) << seq->Value(i) << ' ';
  std::cout << '\n';
}
static void print3(const char* name, const occ::handle<Geom_Curve>& curve, double first, double last, bool lineMode = true, bool circleMode = true, bool conicMode = true) {
  ShapeUpgrade_ConvertCurve3dToBezier converter;
  converter.SetLineMode(lineMode);
  converter.SetCircleMode(circleMode);
  converter.SetConicMode(conicMode);
  converter.Init(curve, first, last);
  auto split = new NCollection_HSequence<double>(); split->Append((first + last) / 2.);
  converter.SetSplitValues(split);
  converter.Compute(); converter.Build(true);
  std::cout << name << " " << converter.Status(ShapeExtend_DONE1) << ' ' << converter.Status(ShapeExtend_DONE2) << ' ' << converter.Status(ShapeExtend_DONE3) << '\n';
  printSeq("values", converter.SplitValues()); printSeq("params", converter.SplitParams());
  const auto& curves = converter.GetCurves();
  for (int i = curves->Lower(); i <= curves->Upper(); ++i) {
    const auto& c = curves->Value(i);
    if (c->IsKind(STANDARD_TYPE(Geom_Line))) { std::cout << c->DynamicType()->Name() << " unbounded\n"; continue; }
    auto p = c->Value(c->FirstParameter()); auto q = c->Value(c->LastParameter());
    std::cout << c->DynamicType()->Name() << ' ' << std::setprecision(17) << p.X() << ' ' << p.Y() << ' ' << p.Z() << ' ' << q.X() << ' ' << q.Y() << ' ' << q.Z() << '\n';
  }
  std::cout << '\n';
}
static void print2(const char* name, const occ::handle<Geom2d_Curve>& curve, double first, double last) {
  ShapeUpgrade_ConvertCurve2dToBezier converter;
  converter.Init(curve, first, last);
  auto split = new NCollection_HSequence<double>(); split->Append((first + last) / 2.);
  converter.SetSplitValues(split);
  converter.Compute(); converter.Build(true);
  std::cout << name << " " << converter.Status(ShapeExtend_DONE1) << ' ' << converter.Status(ShapeExtend_DONE2) << ' ' << converter.Status(ShapeExtend_DONE3) << '\n';
  printSeq("values", converter.SplitValues()); printSeq("params", converter.SplitParams());
  const auto& curves = converter.GetCurves();
  for (int i = curves->Lower(); i <= curves->Upper(); ++i) {
    const auto& c = curves->Value(i);
    auto p = c->Value(c->FirstParameter()); auto q = c->Value(c->LastParameter());
    std::cout << c->DynamicType()->Name() << ' ' << std::setprecision(17) << p.X() << ' ' << p.Y() << ' ' << q.X() << ' ' << q.Y() << '\n';
  }
  std::cout << '\n';
}
int main() {
  NCollection_Array1<gp_Pnt> poles3(1, 3);
  poles3(1) = gp_Pnt(0, 0, 0); poles3(2) = gp_Pnt(1, 2, 0); poles3(3) = gp_Pnt(2, 0, 0);
  print3("line", new Geom_Line(gp_Ax1(gp_Pnt(0, 0, 0), gp_Dir(1, 0, 0))), 0, 4);
  print3("line-mode-off", new Geom_Line(gp_Ax1(gp_Pnt(0, 0, 0), gp_Dir(1, 0, 0))), 0, 4, false);
  print3("trimmed-line-mode-off", new Geom_TrimmedCurve(new Geom_Line(gp_Ax1(gp_Pnt(0, 0, 0), gp_Dir(1, 0, 0))), 0, 4), 0, 4, false);
  print3("bezier-full", new Geom_BezierCurve(poles3), 0, 1);
  print3("bezier-trim", new Geom_BezierCurve(poles3), 0.2, 0.8);
  print3("circle-mode-off", new Geom_Circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 2.), 0, 1, true, false);
  print3("circle-arcs", new Geom_Circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 2.), 0, 1);
  NCollection_Array1<gp_Pnt2d> poles2(1, 3);
  poles2(1) = gp_Pnt2d(0, 0); poles2(2) = gp_Pnt2d(1, 2); poles2(3) = gp_Pnt2d(2, 0);
  NCollection_Array1<gp_Pnt2d> linear2(1, 3);
  linear2(1) = gp_Pnt2d(0, 0); linear2(2) = gp_Pnt2d(2, 0); linear2(3) = gp_Pnt2d(4, 0);
  print2("line2d", new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)), 0, 4);
  print2("bezier2d-trim", new Geom2d_BezierCurve(poles2), 0.2, 0.8);
  print2("linear-bezier2d", new Geom2d_BezierCurve(linear2), 0, 1);
  print2("circle2d-arcs", new Geom2d_Circle(gp_Ax22d(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)), 2.), 0, 1);
}
