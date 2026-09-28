// BSplSLib evaluation, isoparametric curves, flat pole arrays and knot operations at
// pinned OCCT 3d097a0. Grids are sent as "lowerRow rows lowerCol cols values..." in
// row-major order; a surface as "udeg vdeg uper vper urat vrat usemults uknots umults
// vknots vmults poles hasweights weights".
#include <BSplSLib.hxx>
#include <BSplSLib_Cache.hxx>
#include <BSplSLib_EvaluatorFunction.hxx>
namespace bsps_probe {
using bspl_probe::fail;
using bspl_probe::In;
using bspl_probe::o;
using bspl_probe::oa;
using bspl_probe::ob;
using bspl_probe::oi;
using bspl_probe::oia;
using bspl_probe::op;
NCollection_Array2<gp_Pnt> grid3(In& in) {
  int rl = in.i(), nr = in.i(), cl = in.i(), nc = in.i(); NCollection_Array2<gp_Pnt> a(rl, rl + nr - 1, cl, cl + nc - 1);
  for (int i = rl; i < rl + nr; ++i) for (int j = cl; j < cl + nc; ++j) { double x = in.r(), y = in.r(), z = in.r(); a(i, j) = gp_Pnt(x, y, z); }
  return a;
}
NCollection_Array2<double> grid1(In& in) {
  int rl = in.i(), nr = in.i(), cl = in.i(), nc = in.i(); NCollection_Array2<double> a(rl, rl + nr - 1, cl, cl + nc - 1);
  for (int i = rl; i < rl + nr; ++i) for (int j = cl; j < cl + nc; ++j) a(i, j) = in.r();
  return a;
}
void og(const NCollection_Array2<gp_Pnt>& a) { for (int i = a.LowerRow(); i <= a.UpperRow(); ++i) for (int j = a.LowerCol(); j <= a.UpperCol(); ++j) op(a(i, j)); }
void og(const NCollection_Array2<double>& a) { for (int i = a.LowerRow(); i <= a.UpperRow(); ++i) for (int j = a.LowerCol(); j <= a.UpperCol(); ++j) o(a(i, j)); }
NCollection_Array2<gp_Pnt> pgrid(int rows, int cols) { NCollection_Array2<gp_Pnt> a(1, rows, 1, cols); a.Init(gp_Pnt(91, 91, 91)); return a; }
NCollection_Array2<double> wgrid(int rows, int cols) { NCollection_Array2<double> a(1, rows, 1, cols); a.Init(91.0); return a; }

struct Surf {
  int ud, vd; bool up, vp, ur, vr, um;
  NCollection_Array1<double> UK, VK; NCollection_Array1<int> UM, VM;
  NCollection_Array2<gp_Pnt> P; bool hw; NCollection_Array2<double> W;
  const NCollection_Array2<double>* w() const { return hw ? &W : BSplSLib::NoWeights(); }
  const NCollection_Array1<int>* umults() const { return um ? &UM : nullptr; }
  const NCollection_Array1<int>* vmults() const { return um ? &VM : nullptr; }
};
Surf surf(In& in) {
  Surf s; s.ud = in.i(); s.vd = in.i(); s.up = in.i() != 0; s.vp = in.i() != 0; s.ur = in.i() != 0; s.vr = in.i() != 0; s.um = in.i() != 0;
  s.UK = in.reals(); s.UM = in.ints(); s.VK = in.reals(); s.VM = in.ints();
  s.P = grid3(in); s.hw = in.i() != 0; s.W = grid1(in);
  return s;
}

// eval: surface uindex vindex nu nv count (u v)...
void eval_case(std::istream& is) {
  In in{is};
  Surf s = surf(in); int ui = in.i(), vi = in.i(), nu = in.i(), nv = in.i(), n = in.i();
  for (int k = 0; k < n; ++k) {
    double u = in.r(), v = in.r(); gp_Pnt P; gp_Vec Vu, Vv, Vuu, Vvv, Vuv, Vuuu, Vvvv, Vuuv, Vuvv, Vn;
    try { BSplSLib::D0(u, v, ui, vi, s.P, s.w(), s.UK, s.VK, s.umults(), s.vmults(), s.ud, s.vd, s.ur, s.vr, s.up, s.vp, P); op(P); } catch (const Standard_Failure& f) { fail(f); }
    try { BSplSLib::D1(u, v, ui, vi, s.P, s.w(), s.UK, s.VK, s.umults(), s.vmults(), s.ud, s.vd, s.ur, s.vr, s.up, s.vp, P, Vu, Vv); op(P); op(Vu); op(Vv); } catch (const Standard_Failure& f) { fail(f); }
    try {
      BSplSLib::D2(u, v, ui, vi, s.P, s.w(), s.UK, s.VK, s.umults(), s.vmults(), s.ud, s.vd, s.ur, s.vr, s.up, s.vp, P, Vu, Vv, Vuu, Vvv, Vuv);
      op(P); op(Vu); op(Vv); op(Vuu); op(Vvv); op(Vuv);
    } catch (const Standard_Failure& f) { fail(f); }
    try {
      BSplSLib::D3(u, v, ui, vi, s.P, s.w(), s.UK, s.VK, s.umults(), s.vmults(), s.ud, s.vd, s.ur, s.vr, s.up, s.vp, P, Vu, Vv, Vuu, Vvv, Vuv, Vuuu, Vvvv, Vuuv, Vuvv);
      op(P); op(Vu); op(Vv); op(Vuu); op(Vvv); op(Vuv); op(Vuuu); op(Vvvv); op(Vuuv); op(Vuvv);
    } catch (const Standard_Failure& f) { fail(f); }
    try { BSplSLib::DN(u, v, nu, nv, ui, vi, s.P, s.w(), s.UK, s.VK, s.umults(), s.vmults(), s.ud, s.vd, s.ur, s.vr, s.up, s.vp, Vn); op(Vn); } catch (const Standard_Failure& f) { fail(f); }
    try { double w = 91; BSplSLib::HomogeneousD0(u, v, ui, vi, s.P, s.w(), s.UK, s.VK, s.umults(), s.vmults(), s.ud, s.vd, s.ur, s.vr, s.up, s.vp, w, P); o(w); op(P); } catch (const Standard_Failure& f) { fail(f); }
    try {
      double d = 91, du = 91, dv = 91;
      BSplSLib::HomogeneousD1(u, v, ui, vi, s.P, s.w(), s.UK, s.VK, s.umults(), s.vmults(), s.ud, s.vd, s.ur, s.vr, s.up, s.vp, P, Vu, Vv, d, du, dv);
      op(P); op(Vu); op(Vv); o(d); o(du); o(dv);
    } catch (const Standard_Failure& f) { fail(f); }
  }
  std::cout << '\n';
}

// rational: udeg vdeg n m all ders
void rational_case(std::istream& is) {
  In in{is};
  int ud = in.i(), vd = in.i(), n = in.i(), m = in.i(); bool all = in.i() != 0; auto H = in.buf();
  std::vector<double> R(3 * (n + 1) * (m + 1), 91.0);
  try { BSplSLib::RationalDerivative(ud, vd, n, m, H[0], R[0], all); ob(R); } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// iso: surface param isu cweights clower count | pairs (i1 i2 j1 j2 eps)... | last udir
void iso_case(std::istream& is) {
  In in{is};
  Surf s = surf(in); double param = in.r(); bool isu = in.i() != 0, cw = in.i() != 0; int cl = in.i(), cn = in.i();
  NCollection_Array1<gp_Pnt> CP(cl, cl + cn - 1); CP.Init(gp_Pnt(91, 91, 91)); NCollection_Array1<double> CW(cl, cl + cn - 1); CW.Init(91.0);
  try {
    BSplSLib::Iso(param, isu, s.P, s.w(), isu ? s.UK : s.VK, isu ? s.umults() : s.vmults(), isu ? s.ud : s.vd, isu ? s.up : s.vp, CP, cw ? &CW : nullptr);
    bspl_probe::opa(CP); if (cw) oa(CW);
  } catch (const Standard_Failure& f) { fail(f); }
  int np = in.i();
  for (int k = 0; k < np; ++k) {
    int i1 = in.i(), i2 = in.i(), j1 = in.i(), j2 = in.i(); double eps = in.r();
    try { oi(BSplSLib::IsRational(s.W, i1, i2, j1, j2, eps)); } catch (const Standard_Failure& f) { fail(f); }
  }
  int last = in.i(); bool udir = in.i() != 0;
  try { NCollection_Array2<gp_Pnt> R(s.P.LowerRow(), s.P.UpperRow(), s.P.LowerCol(), s.P.UpperCol()); R = s.P; BSplSLib::Reverse(R, last, udir); og(R); } catch (const Standard_Failure& f) { fail(f); }
  try { NCollection_Array2<double> R(s.W.LowerRow(), s.W.UpperRow(), s.W.LowerCol(), s.W.UpperCol()); R = s.W; BSplSLib::Reverse(R, last, udir); og(R); } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// poles: poles weights udir fplower fp
void poles_case(std::istream& is) {
  In in{is};
  auto P = grid3(in); auto W = grid1(in); bool udir = in.i() != 0; int fl = in.i(); int n = P.Size();
  NCollection_Array1<double> F3(fl, fl + 3 * n - 1), F4(fl, fl + 4 * n - 1); F3.Init(91.0); F4.Init(91.0);
  try { BSplSLib::SetPoles(P, F3, udir); oa(F3); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplSLib::SetPoles(P, W, F4, udir); oa(F4); } catch (const Standard_Failure& f) { fail(f); }
  auto G = in.reals();
  NCollection_Array2<gp_Pnt> Q(P.LowerRow(), P.UpperRow(), P.LowerCol(), P.UpperCol()); Q.Init(gp_Pnt(91, 91, 91));
  NCollection_Array2<double> V(P.LowerRow(), P.UpperRow(), P.LowerCol(), P.UpperCol()); V.Init(91.0);
  try { BSplSLib::GetPoles(G, Q, udir); og(Q); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplSLib::GetPoles(G, Q, V, udir); og(Q); og(V); } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// Output grids of a surface knot operation along one direction.
struct Grids {
  NCollection_Array2<gp_Pnt> P; NCollection_Array2<double> W; NCollection_Array1<double> K; NCollection_Array1<int> M;
  Grids(const Surf& s, bool udir, int np, int nk)
    : P(pgrid(udir ? np : s.P.ColLength(), udir ? s.P.RowLength() : np)), W(wgrid(udir ? np : s.P.ColLength(), udir ? s.P.RowLength() : np)), K(1, nk), M(1, nk) {
    K.Init(91.0); M.Init(-7);
  }
  NCollection_Array2<double>* w(const Surf& s) { return s.hw ? &W : nullptr; }
  void print(const Surf& s) { og(P); if (s.hw) og(W); oa(K); oia(M); }
};

// ops: surface udir | insert: addknots useaddmults addmults eps add | remove: index mult tol np nk
//      | degree: newdeg np | (unperiodize)
void ops_case(std::istream& is) {
  In in{is};
  Surf s = surf(in); bool udir = in.i() != 0; int deg = udir ? s.ud : s.vd; bool per = udir ? s.up : s.vp;
  const auto& K = udir ? s.UK : s.VK; const auto& M = udir ? s.UM : s.VM;
  auto AK = in.reals(); bool useam = in.i() != 0; auto AM = in.ints(); double eps = in.r(); bool add = in.i() != 0;
  int np = -7, nk = -7;
  bool ok = BSplCLib::PrepareInsertKnots(deg, per, K, M, AK, useam ? &AM : nullptr, np, nk, eps, add);
  oi(ok); oi(np); oi(nk);
  if (ok) {
    Grids r(s, udir, np, nk);
    try { BSplSLib::InsertKnots(udir, deg, per, s.P, s.w(), K, M, AK, useam ? &AM : nullptr, r.P, r.w(s), r.K, r.M, eps, add); r.print(s); } catch (const Standard_Failure& f) { fail(f); }
  }
  int index = in.i(), mult = in.i(); double tol = in.r(); np = in.i(); nk = in.i();
  { Grids r(s, udir, np, nk);
    try { oi(BSplSLib::RemoveKnot(udir, index, mult, deg, per, s.P, s.w(), K, M, r.P, r.w(s), r.K, r.M, tol)); r.print(s); } catch (const Standard_Failure& f) { fail(f); } }
  int nd = in.i(); np = in.i();
  try {
    nk = BSplCLib::IncreaseDegreeCountKnots(deg, nd, per, M); oi(nk); Grids r(s, udir, np, nk);
    BSplSLib::IncreaseDegree(udir, deg, nd, per, s.P, s.w(), K, M, r.P, r.w(s), r.K, r.M); r.print(s);
  } catch (const Standard_Failure& f) { fail(f); }
  if (per) {
    try {
      BSplCLib::PrepareUnperiodize(deg, M, nk, np); oi(nk); oi(np); Grids r(s, udir, np, nk);
      BSplSLib::Unperiodize(udir, deg, M, K, s.P, s.w(), r.M, r.K, r.P, r.w(s)); r.print(s);
    } catch (const Standard_Failure& f) { fail(f); }
  }
  std::cout << '\n';
}
} // namespace bsps_probe
