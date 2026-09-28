// Convert package conversions at pinned OCCT 3d097a0. Arrays arrive as
// "lower count values..."; grids as "lowerRow rows lowerCol cols values..."; 2D frames
// as "ox oy dx dy sense"; 3D frames as "ox oy oz nx ny nz xx xy xz indirect".
// Arrays print as "lower length values...", grids as "rows cols values...".
#include <Convert_CircleToBSplineCurve.hxx>
#include <Convert_CompBezierCurves2dToBSplineCurve2d.hxx>
#include <Convert_CompBezierCurvesToBSplineCurve.hxx>
#include <Convert_CompPolynomialToPoles.hxx>
#include <Convert_ConeToBSplineSurface.hxx>
#include <Convert_CylinderToBSplineSurface.hxx>
#include <Convert_EllipseToBSplineCurve.hxx>
#include <Convert_GridPolynomialToPoles.hxx>
#include <Convert_HyperbolaToBSplineCurve.hxx>
#include <Convert_ParabolaToBSplineCurve.hxx>
#include <Convert_PolynomialCosAndSin.hxx>
#include <Convert_SphereToBSplineSurface.hxx>
#include <Convert_TorusToBSplineSurface.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Cone.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Elips2d.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Parab2d.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>
namespace convert_probe {
using bspl_probe::fail;
using bspl_probe::In;
using bspl_probe::o;
using bspl_probe::oa;
using bspl_probe::oi;
using bspl_probe::oia;
using bspl_probe::op;
using bspl_probe::opa;
Standard_DISABLE_DEPRECATION_WARNINGS
#define CV_TRY(...) try { __VA_ARGS__; } catch (const Standard_Failure& cv_error) { fail(cv_error); }
void arr(const NCollection_Array1<double>& a) { oi(a.Lower()); oi(a.Length()); oa(a); }
void arr(const NCollection_Array1<int>& a) { oi(a.Lower()); oi(a.Length()); oia(a); }
template <class P> void pts(const NCollection_Array1<P>& a) { oi(a.Lower()); oi(a.Length()); opa(a); }
template <class T> void grid(const NCollection_Array2<T>& a) {
  oi(a.NbRows()); oi(a.NbColumns());
  for (int i = a.LowerRow(); i <= a.UpperRow(); ++i) for (int j = a.LowerCol(); j <= a.UpperCol(); ++j) {
    if constexpr (std::is_same_v<T, gp_Pnt>) op(a(i, j)); else if constexpr (std::is_same_v<T, int>) oi(a(i, j)); else o(a(i, j));
  }
}
NCollection_Array2<double> rgrid(In& in) {
  int rl = in.i(), nr = in.i(), cl = in.i(), nc = in.i(); NCollection_Array2<double> a(rl, rl + nr - 1, cl, cl + nc - 1);
  for (int i = rl; i < rl + nr; ++i) for (int j = cl; j < cl + nc; ++j) a(i, j) = in.r();
  return a;
}
NCollection_Array2<int> igrid(In& in) {
  int rl = in.i(), nr = in.i(), cl = in.i(), nc = in.i(); NCollection_Array2<int> a(rl, rl + nr - 1, cl, cl + nc - 1);
  for (int i = rl; i < rl + nr; ++i) for (int j = cl; j < cl + nc; ++j) a(i, j) = in.i();
  return a;
}
gp_Ax22d frame2(double v[5]) { return gp_Ax22d(gp_Pnt2d(v[0], v[1]), gp_Dir2d(v[2], v[3]), v[4] != 0); }
gp_Ax3 frame3(double v[10]) {
  gp_Ax3 a(gp_Pnt(v[0], v[1], v[2]), gp_Dir(v[3], v[4], v[5]), gp_Dir(v[6], v[7], v[8]));
  if (v[9] != 0) a.YReverse();
  return a;
}

void conic_out(const Convert_ConicToBSplineCurve& c) {
  oi(c.Degree()); oi(c.IsPeriodic()); oi(c.NbPoles()); oi(c.NbKnots());
  pts(c.Poles()); arr(c.Weights()); arr(c.Knots()); arr(c.Multiplicities());
  for (int k : {0, 1, c.NbPoles(), c.NbPoles() + 1}) { CV_TRY(op(c.Pole(k))); CV_TRY(o(c.Weight(k))); }
  for (int k : {0, 1, c.NbKnots(), c.NbKnots() + 1}) { CV_TRY(o(c.Knot(k))); CV_TRY(oi(c.Multiplicity(k))); }
}

// conic: kind(0 circle,1 ellipse,2 hyperbola,3 parabola) arc frame2 a b u1 u2 param
void conic_case(std::istream& s) {
  In in{s};
  int kind = in.i(); bool arc = in.i() != 0; double f[5]; for (auto& v : f) v = in.r();
  double a = in.r(), b = in.r(), u1 = in.r(), u2 = in.r(); auto p = Convert_ParameterisationType(in.i());
  try {
    gp_Ax22d ax = frame2(f);
    if (kind == 0) { gp_Circ2d c(ax, a); if (arc) conic_out(Convert_CircleToBSplineCurve(c, u1, u2, p)); else conic_out(Convert_CircleToBSplineCurve(c, p)); }
    if (kind == 1) { gp_Elips2d e(ax, a, b); if (arc) conic_out(Convert_EllipseToBSplineCurve(e, u1, u2, p)); else conic_out(Convert_EllipseToBSplineCurve(e, p)); }
    if (kind == 2) conic_out(Convert_HyperbolaToBSplineCurve(gp_Hypr2d(ax, a, b), u1, u2));
    if (kind == 3) conic_out(Convert_ParabolaToBSplineCurve(gp_Parab2d(ax, a), u1, u2));
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// cossin: periodic param u1 u2 (the public legacy BuildCosAndSin overloads)
void cossin_case(std::istream& s) {
  In in{s};
  bool periodic = in.i() != 0; auto p = Convert_ParameterisationType(in.i()); double u1 = in.r(), u2 = in.r();
  const Convert_ParabolaToBSplineCurve host(gp_Parab2d(gp_Ax22d(), 1.0), 0.0, 1.0);
  occ::handle<NCollection_HArray1<double>> cs, sn, den, knots; occ::handle<NCollection_HArray1<int>> mults; int degree = -7;
  try {
    if (periodic) host.BuildCosAndSin(p, cs, sn, den, degree, knots, mults);
    else host.BuildCosAndSin(p, u1, u2, cs, sn, den, degree, knots, mults);
    arr(cs->Array1()); arr(sn->Array1()); arr(den->Array1()); oi(degree); arr(knots->Array1()); arr(mults->Array1());
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// polycs: u1 u2 (BuildPolynomialCosAndSin on 8 poles)
void polycs_case(std::istream& s) {
  In in{s};
  double u1 = in.r(), u2 = in.r();
  NCollection_Array1<double> cs(1, 8), sn(1, 8), den(1, 8); cs.Init(91); sn.Init(91); den.Init(91);
  CV_TRY(BuildPolynomialCosAndSin(u1, u2, 8, cs, sn, den); arr(cs); arr(sn); arr(den));
  std::cout << '\n';
}

void surface_out(const Convert_ElementarySurfaceToBSplineSurface& c) {
  oi(c.UDegree()); oi(c.VDegree()); oi(c.NbUPoles()); oi(c.NbVPoles()); oi(c.NbUKnots()); oi(c.NbVKnots());
  oi(c.IsUPeriodic()); oi(c.IsVPeriodic());
  grid(c.Poles()); grid(c.Weights()); arr(c.UKnots()); arr(c.VKnots()); arr(c.UMultiplicities()); arr(c.VMultiplicities());
  int nu = c.NbUPoles(), nv = c.NbVPoles();
  for (auto [i, j] : {std::pair{0, 1}, {1, 0}, {1, 1}, {nu, nv}, {nu + 1, nv}, {nu, nv + 1}}) { CV_TRY(op(c.Pole(i, j))); CV_TRY(o(c.Weight(i, j))); }
  for (int k : {0, 1, c.NbUKnots(), c.NbUKnots() + 1}) { CV_TRY(o(c.UKnot(k))); CV_TRY(oi(c.UMultiplicity(k))); }
  for (int k : {0, 1, c.NbVKnots(), c.NbVKnots() + 1}) { CV_TRY(o(c.VKnot(k))); CV_TRY(oi(c.VMultiplicity(k))); }
}

// surface: kind(0 sphere,1 torus,2 cylinder,3 cone) mode(0 whole/periodic,1 patch,2 band) frame3 a b p1 p2 p3 p4 utrim
void surface_case(std::istream& s) {
  In in{s};
  int kind = in.i(), mode = in.i(); double f[10]; for (auto& v : f) v = in.r();
  double a = in.r(), b = in.r(), p1 = in.r(), p2 = in.r(), p3 = in.r(), p4 = in.r(); bool utrim = in.i() != 0;
  try {
    gp_Ax3 ax = frame3(f);
    if (kind == 0) {
      gp_Sphere q(ax, a);
      if (mode == 0) surface_out(Convert_SphereToBSplineSurface(q));
      if (mode == 1) surface_out(Convert_SphereToBSplineSurface(q, p1, p2, p3, p4));
      if (mode == 2) surface_out(Convert_SphereToBSplineSurface(q, p1, p2, utrim));
    }
    if (kind == 1) {
      gp_Torus q(ax, a, b);
      if (mode == 0) surface_out(Convert_TorusToBSplineSurface(q));
      if (mode == 1) surface_out(Convert_TorusToBSplineSurface(q, p1, p2, p3, p4));
      if (mode == 2) surface_out(Convert_TorusToBSplineSurface(q, p1, p2, utrim));
    }
    if (kind == 2) {
      gp_Cylinder q(ax, a);
      if (mode == 0) surface_out(Convert_CylinderToBSplineSurface(q, p1, p2));
      if (mode == 1) surface_out(Convert_CylinderToBSplineSurface(q, p1, p2, p3, p4));
    }
    if (kind == 3) {
      gp_Cone q(ax, b, a);
      if (mode == 0) surface_out(Convert_ConeToBSplineSurface(q, p1, p2));
      if (mode == 1) surface_out(Convert_ConeToBSplineSurface(q, p1, p2, p3, p4));
    }
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

template <class C, class P> void bezier_out(C& conv) {
  oi(conv.Degree()); oi(conv.NbPoles()); oi(conv.NbKnots());
  NCollection_Array1<P> poles(1, conv.NbPoles()); NCollection_Array1<double> knots(1, conv.NbKnots()); NCollection_Array1<int> mults(1, conv.NbKnots());
  conv.Poles(poles); conv.KnotsAndMults(knots, mults); pts(poles); arr(knots); arr(mults);
  NCollection_Array1<P> shifted(0, conv.NbPoles() - 1); conv.Poles(shifted); pts(shifted);
  NCollection_Array1<P> longer(1, conv.NbPoles() + 1); CV_TRY(conv.Poles(longer));
}

template <class C, class P> void bezier_run(In& in, double angular) {
  C conv(angular);
  conv.Perform(); oi(conv.Degree()); oi(conv.NbPoles());
  int n = in.i();
  for (int k = 0; k < n; ++k) {
    NCollection_Array1<P> poles = [&] { if constexpr (std::is_same_v<P, gp_Pnt>) return in.p3(); else return in.p2(); }();
    conv.AddCurve(poles);
  }
  CV_TRY(conv.Perform(); bezier_out<C, P>(conv));
  CV_TRY(conv.Perform(); bezier_out<C, P>(conv));
}

// bezier: dim angular ncurves (lower count coords...)...
void bezier_case(std::istream& s) {
  In in{s};
  int dim = in.i(); double angular = in.r();
  if (dim == 2) bezier_run<Convert_CompBezierCurves2dToBSplineCurve2d, gp_Pnt2d>(in, angular);
  else bezier_run<Convert_CompBezierCurvesToBSplineCurve, gp_Pnt>(in, angular);
  std::cout << '\n';
}

void comppoly_out(const Convert_CompPolynomialToPoles& c) {
  oi(c.IsDone()); oi(c.Degree()); oi(c.NbPoles()); oi(c.NbKnots());
  CV_TRY(grid(c.Poles())); CV_TRY(arr(c.Knots())); CV_TRY(arr(c.Multiplicities()));
  occ::handle<NCollection_HArray2<double>> p; occ::handle<NCollection_HArray1<double>> k; occ::handle<NCollection_HArray1<int>> m;
  c.Poles(p); c.Knots(k); c.Multiplicities(m);
  oi(!p.IsNull()); if (!p.IsNull()) grid(p->Array2());
  oi(!k.IsNull()); if (!k.IsNull()) arr(k->Array1());
  oi(!m.IsNull()); if (!m.IsNull()) arr(m->Array1());
}

// comppoly: mode 0 uniform: ncurves continuity dim maxdeg nullmask counts coeffs intervals true
//           mode 1 continuities: ncurves dim maxdeg continuity counts coeffs intervals true
//           mode 2 single: dim maxdeg degree coeffs intervals true
void comppoly_case(std::istream& s) {
  In in{s};
  int mode = in.i();
  try {
    if (mode == 0) {
      int nc = in.i(), cont = in.i(), dim = in.i(), maxdeg = in.i(), mask = in.i();
      auto counts = in.ints(); auto coeffs = in.reals(); auto intervals = rgrid(in); auto trues = in.reals();
      occ::handle<NCollection_HArray1<int>> hc; occ::handle<NCollection_HArray1<double>> hk, ht; occ::handle<NCollection_HArray2<double>> hi;
      if (!(mask & 1)) hc = new NCollection_HArray1<int>(counts);
      if (!(mask & 2)) hk = new NCollection_HArray1<double>(coeffs);
      if (!(mask & 4)) hi = new NCollection_HArray2<double>(intervals);
      if (!(mask & 8)) ht = new NCollection_HArray1<double>(trues);
      comppoly_out(Convert_CompPolynomialToPoles(nc, cont, dim, maxdeg, hc, hk, hi, ht));
    } else if (mode == 1) {
      int nc = in.i(), dim = in.i(), maxdeg = in.i();
      auto cont = in.ints(); auto counts = in.ints(); auto coeffs = in.reals(); auto intervals = rgrid(in); auto trues = in.reals();
      comppoly_out(Convert_CompPolynomialToPoles(nc, dim, maxdeg, cont, counts, coeffs, intervals, trues));
    } else {
      int dim = in.i(), maxdeg = in.i(), degree = in.i();
      auto coeffs = in.reals(); auto intervals = in.reals(); auto trues = in.reals();
      comppoly_out(Convert_CompPolynomialToPoles(dim, maxdeg, degree, coeffs, intervals, trues));
    }
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

void grid_out(const Convert_GridPolynomialToPoles& c) {
  oi(c.IsDone());
  CV_TRY(oi(c.NbUPoles())); CV_TRY(oi(c.NbVPoles())); CV_TRY(oi(c.UDegree())); CV_TRY(oi(c.VDegree()));
  CV_TRY(oi(c.NbUKnots())); CV_TRY(oi(c.NbVKnots())); CV_TRY(grid(c.Poles()));
  CV_TRY(arr(c.UKnots())); CV_TRY(arr(c.VKnots())); CV_TRY(arr(c.UMultiplicities())); CV_TRY(arr(c.VMultiplicities()));
}

// grid: mode 0 patch: maxu maxv counts coeffs polyu polyv
//       mode 1 grid: nbu nbv ucont vcont maxu maxv countsgrid coeffs polyu polyv trueu truev
void grid_case(std::istream& s) {
  In in{s};
  int mode = in.i();
  try {
    if (mode == 0) {
      int mu = in.i(), mv = in.i(); auto counts = in.ints(); auto coeffs = in.reals(); auto pu = in.reals(); auto pv = in.reals();
      grid_out(Convert_GridPolynomialToPoles(mu, mv, counts, coeffs, pu, pv));
    } else {
      int nu = in.i(), nv = in.i(), cu = in.i(), cv = in.i(), mu = in.i(), mv = in.i();
      auto counts = igrid(in); auto coeffs = in.reals(); auto pu = in.reals(); auto pv = in.reals(); auto tu = in.reals(); auto tv = in.reals();
      grid_out(Convert_GridPolynomialToPoles(nu, nv, cu, cv, mu, mv, counts, coeffs, pu, pv, tu, tv));
    }
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}
Standard_ENABLE_DEPRECATION_WARNINGS
} // namespace convert_probe
