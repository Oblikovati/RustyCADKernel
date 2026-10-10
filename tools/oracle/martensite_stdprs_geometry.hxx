#pragma once
// All StdPrs_Curve overloads, using real geometry and the existing recording
// driver.
#include <GeomAdaptor_Curve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Line.hxx>
#include <Prs3d_ArrowAspect.hxx>
static occ::handle<Geom_BSplineCurve> stdprs_spline() {
  NCollection_Array1<gp_Pnt> poles(1, 10);
  NCollection_Array1<double> knots(1, 10);
  NCollection_Array1<int> mults(1, 10);
  for (int i = 0; i < 10; ++i) {
    poles(i + 1) = gp_Pnt(i - 4.5, i % 3 - 1., 0);
    knots(i + 1) = i;
    mults(i + 1) = i == 0 || i == 9 ? 2 : 1;
  }
  return new Geom_BSplineCurve(poles, knots, mults, 1, false);
}
static GeomAdaptor_Curve stdprs_geometry(int kind) {
  occ::handle<Geom_Curve> curve;
  if (kind == 1)
    curve = new Geom_Circle(gp_Ax2(), 3);
  else if (kind == 2)
    curve = new Geom_Ellipse(gp_Ax2(), 5, 2);
  else if (kind == 3) {
    NCollection_Array1<gp_Pnt> poles(1, 4);
    poles(1) = gp_Pnt(-4, -3, 0);
    poles(2) = gp_Pnt(-2, 4, 0);
    poles(3) = gp_Pnt(2, -4, 0);
    poles(4) = gp_Pnt(4, 3, 0);
    curve = new Geom_BezierCurve(poles);
  } else if (kind == 4)
    curve = stdprs_spline();
  else
    curve = new Geom_Line(gp_Pnt(-3, -2, 0), gp::DX());
  if (kind == 0)
    return GeomAdaptor_Curve(curve, 0, 6);
  if (kind == 6)
    return GeomAdaptor_Curve(curve, -Precision::Infinite(), 3);
  if (kind == 7)
    return GeomAdaptor_Curve(curve, 2, Precision::Infinite());
  return GeomAdaptor_Curve(curve);
}
