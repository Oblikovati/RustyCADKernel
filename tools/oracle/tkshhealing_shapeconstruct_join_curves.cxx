#include <Geom_BSplineCurve.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_Line.hxx>
#include <ShapeConstruct.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2d.hxx>

#include <iomanip>
#include <iostream>

static void report_3d_join() {
  const occ::handle<Geom_Curve> first =
      new Geom_Line(gp_Ax1(gp_Pnt(0., 0., 0.), gp_Dir(1., 0., 0.)));
  const occ::handle<Geom_Curve> second = occ::down_cast<Geom_Curve>(first->Copy());
  double first_start = 0., first_end = 1., second_start = 1., second_end = 2.;
  bool reversed_first = false, reversed_second = false;
  occ::handle<Geom_Curve> output;
  const bool joined = ShapeConstruct::JoinCurves(
      first, second, TopAbs_FORWARD, TopAbs_FORWARD, first_start, first_end,
      second_start, second_end, output, reversed_first, reversed_second);
  const occ::handle<Geom_BSplineCurve> spline =
      occ::down_cast<Geom_BSplineCurve>(output);
  const gp_Pnt start = spline->Value(spline->FirstParameter());
  const gp_Pnt end = spline->Value(spline->LastParameter());
  std::cout << "join3d " << joined << ' ' << reversed_first << ' ' << reversed_second << ' '
            << first_start << ' ' << first_end << ' ' << second_start << ' ' << second_end << ' '
            << spline->Degree() << ' ' << spline->NbPoles() << ' ' << start.X() << ' '
            << start.Y() << ' ' << end.X() << ' ' << end.Y() << '\n';
}

static void report_2d_join() {
  const occ::handle<Geom2d_Curve> first =
      new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.)));
  const occ::handle<Geom2d_Curve> second = occ::down_cast<Geom2d_Curve>(first->Copy());
  double first_start = 0., first_end = 1., second_start = 1., second_end = 2.;
  bool reversed_first = false, reversed_second = false;
  occ::handle<Geom2d_Curve> output;
  const bool joined = ShapeConstruct::JoinCurves(
      first, second, TopAbs_FORWARD, TopAbs_FORWARD, first_start, first_end,
      second_start, second_end, output, reversed_first, reversed_second, true);
  const occ::handle<Geom2d_BSplineCurve> spline =
      occ::down_cast<Geom2d_BSplineCurve>(output);
  const gp_Pnt2d start = spline->Value(spline->FirstParameter());
  const gp_Pnt2d end = spline->Value(spline->LastParameter());
  std::cout << "join2d " << joined << ' ' << reversed_first << ' ' << reversed_second << ' '
            << first_start << ' ' << first_end << ' ' << second_start << ' ' << second_end << ' '
            << spline->Degree() << ' ' << spline->NbPoles() << ' ' << start.X() << ' '
            << start.Y() << ' ' << end.X() << ' ' << end.Y() << '\n';
}

static void report_reversed_2d_join() {
  const occ::handle<Geom2d_Curve> first =
      new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.)));
  const occ::handle<Geom2d_Curve> second = occ::down_cast<Geom2d_Curve>(first->Copy());
  double first_start = 0., first_end = 1., second_start = 0., second_end = 1.;
  bool reversed_first = false, reversed_second = false;
  occ::handle<Geom2d_Curve> output;
  const bool joined = ShapeConstruct::JoinCurves(
      first, second, TopAbs_REVERSED, TopAbs_FORWARD, first_start, first_end,
      second_start, second_end, output, reversed_first, reversed_second, false);
  const occ::handle<Geom2d_BSplineCurve> spline =
      occ::down_cast<Geom2d_BSplineCurve>(output);
  const gp_Pnt2d start = spline->Value(spline->FirstParameter());
  const gp_Pnt2d end = spline->Value(spline->LastParameter());
  std::cout << "join2d_reversed " << joined << ' ' << reversed_first << ' '
            << reversed_second << ' ' << first_start << ' ' << first_end << ' '
            << second_start << ' ' << second_end << ' ' << spline->Degree() << ' '
            << spline->NbPoles() << ' ' << start.X() << ' ' << start.Y() << ' '
            << end.X() << ' ' << end.Y() << '\n';
}

int main() {
  std::cout << std::setprecision(17);
  report_3d_join();
  report_2d_join();
  report_reversed_2d_join();
}
