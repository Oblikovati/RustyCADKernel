// BSplCLib knot management, local schemes and span extraction at pinned OCCT 3d097a0.
// Arrays are sent as "lower count values..."; points as "lower count dim coords...".
#include <BSplCLib.hxx>
#include <BSplCLib_Cache.hxx>
#include <gp_Vec.hxx>
#include <gp_Vec2d.hxx>
#include <math_Matrix.hxx>
#include <vector>
namespace bspl_probe {
struct In {
  std::istream& s;
  double r() { double v; s >> v; return v; }
  int i() { return (int)r(); }
  NCollection_Array1<double> reals() { int lo = i(), n = i(); NCollection_Array1<double> a(lo, lo + n - 1); for (int k = lo; k < lo + n; ++k) a(k) = r(); return a; }
  NCollection_Array1<int> ints() { int lo = i(), n = i(); NCollection_Array1<int> a(lo, lo + n - 1); for (int k = lo; k < lo + n; ++k) a(k) = i(); return a; }
  std::vector<double> buf() { int n = i(); std::vector<double> v(n > 0 ? n : 0); for (auto& x : v) x = r(); return v; }
  NCollection_Array1<gp_Pnt> p3() { int lo = i(), n = i(); NCollection_Array1<gp_Pnt> a(lo, lo + n - 1); for (int k = lo; k < lo + n; ++k) { double x = r(), y = r(), z = r(); a(k) = gp_Pnt(x, y, z); } return a; }
  NCollection_Array1<gp_Pnt2d> p2() { int lo = i(), n = i(); NCollection_Array1<gp_Pnt2d> a(lo, lo + n - 1); for (int k = lo; k < lo + n; ++k) { double x = r(), y = r(); a(k) = gp_Pnt2d(x, y); } return a; }
};
void o(double v) { std::cout << bits(v) << ' '; }
void oi(int v) { std::cout << v << ' '; }
void fail(const Standard_Failure& f) { std::cout << f.ExceptionType() << ' '; }
void oa(const NCollection_Array1<double>& a) { for (int k = a.Lower(); k <= a.Upper(); ++k) o(a(k)); }
void oia(const NCollection_Array1<int>& a) { for (int k = a.Lower(); k <= a.Upper(); ++k) oi(a(k)); }
void ob(const std::vector<double>& v) { for (double x : v) o(x); }
void op(const gp_Pnt& p) { o(p.X()); o(p.Y()); o(p.Z()); }
void op(const gp_Pnt2d& p) { o(p.X()); o(p.Y()); }
void op(const gp_Vec& p) { o(p.X()); o(p.Y()); o(p.Z()); }
void op(const gp_Vec2d& p) { o(p.X()); o(p.Y()); }
template <class P> void opa(const NCollection_Array1<P>& a) { for (int k = a.Lower(); k <= a.Upper(); ++k) op(a(k)); }
NCollection_Array1<double> copy(const NCollection_Array1<double>& a) { NCollection_Array1<double> c(a.Lower(), a.Upper()); c = a; return c; }

// knots: deg periodic knots mults xs u1 u2 seq_ok weights nidx(i1,i2)... last
void knots_case(std::istream& s) {
  In in{s};
  int deg = in.i(); bool per = in.i() != 0;
  auto K = in.reals(); auto M = in.ints(); auto X = in.buf(); double u1 = in.r(), u2 = in.r();
  bool seqok = in.i() != 0; auto W = in.reals(); int npairs = in.i();
  for (double x : X) { int pos = -7; BSplCLib::Hunt(K, x, pos); oi(pos); }
  try { oi(BSplCLib::FirstUKnotIndex(deg, M)); } catch (const Standard_Failure& f) { fail(f); }
  try { oi(BSplCLib::LastUKnotIndex(deg, M)); } catch (const Standard_Failure& f) { fail(f); }
  for (int k = M.Lower(); k <= M.Upper(); ++k) oi(BSplCLib::FlatIndex(deg, k, M, per));
  oi(BSplCLib::MaxKnotMult(M, M.Lower(), M.Upper())); oi(BSplCLib::MinKnotMult(M, M.Lower(), M.Upper()));
  oi(BSplCLib::NbPoles(deg, per, M));
  int len = BSplCLib::KnotSequenceLength(M, deg, per); oi(len);
  if (seqok) {
    NCollection_Array1<double> seq(1, len);
    BSplCLib::KnotSequence(K, M, deg, per, seq); oa(seq);
    int nk = BSplCLib::KnotsLength(seq, per); oi(nk);
    NCollection_Array1<double> k2(1, nk); NCollection_Array1<int> m2(1, nk);
    BSplCLib::Knots(seq, k2, m2, per); oa(k2); oia(m2);
    if (!per) { NCollection_Array1<double> seq0(1, BSplCLib::KnotSequenceLength(M, 0, false)); BSplCLib::KnotSequence(K, M, seq0); oa(seq0); }
  }
  oi((int)BSplCLib::KnotForm(K, K.Lower(), K.Upper())); oi((int)BSplCLib::KnotForm(K, K.Lower() + 1, K.Upper()));
  oi((int)BSplCLib::MultForm(M, M.Lower(), M.Upper())); oi((int)BSplCLib::MultForm(M, M.Upper(), M.Lower()));
  try { GeomAbs_BSplKnotDistribution form; int mm = -7; BSplCLib::KnotAnalysis(deg, per, K, M, form, mm); oi((int)form); oi(mm); } catch (const Standard_Failure& f) { fail(f); }
  for (int k = M.Lower(); k <= M.Upper(); ++k) oi(BSplCLib::PoleIndex(deg, k, per, M));
  auto R = copy(K); BSplCLib::Reparametrize(u1, u2, R); oa(R);
  auto V = copy(K); BSplCLib::Reverse(V); oa(V);
  NCollection_Array1<int> MV(M.Lower(), M.Upper()); MV = M; BSplCLib::Reverse(MV); oia(MV);
  for (int p = 0; p < npairs; ++p) { int i1 = in.i(), i2 = in.i(); oi(BSplCLib::IsRational(W, i1, i2)); }
  int last = in.i();
  auto WR = copy(W); BSplCLib::Reverse(WR, last); oa(WR);
  NCollection_Array1<gp_Pnt> P3(W.Lower(), W.Upper()); NCollection_Array1<gp_Pnt2d> P2(W.Lower(), W.Upper());
  for (int k = W.Lower(); k <= W.Upper(); ++k) { P3(k) = gp_Pnt(W(k), -W(k), 2 * W(k)); P2(k) = gp_Pnt2d(W(k), k); }
  BSplCLib::Reverse(P3, last); opa(P3); BSplCLib::Reverse(P2, last); opa(P2);
  std::cout << '\n';
}

// locate: mode deg periodic knots usemults mults u k1 k2 hint
void locate_case(std::istream& s) {
  In in{s};
  int mode = in.i(), deg = in.i(); bool per = in.i() != 0;
  auto K = in.reals(); bool use = in.i() != 0; auto M = in.ints();
  double u = in.r(); int k1 = in.i(), k2 = in.i(), hint = in.i();
  int idx = hint; double nu = 91.0;
  try {
    if (mode == 0) BSplCLib::LocateParameter(deg, K, M, u, per, k1, k2, idx, nu);
    else if (mode == 1) BSplCLib::LocateParameter(deg, K, u, per, k1, k2, idx, nu);
    else BSplCLib::LocateParameter(deg, K, use ? &M : BSplCLib::NoMults(), u, per, idx, nu);
    oi(idx); o(nu);
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// scheme: op deg dim u depth length tol knots poles
void scheme_case(std::istream& s) {
  In in{s};
  int op = in.i(), deg = in.i(), dim = in.i(); double u = in.r(); int depth = in.i(), length = in.i(); double tol = in.r();
  auto K = in.buf(); auto P = in.buf();
  switch (op) {
    case 0: BSplCLib::Eval(u, deg, K[0], dim, P[0]); break;
    case 1: BSplCLib::BoorScheme(u, deg, K[0], dim, P[0], depth, length); break;
    case 2: oi(BSplCLib::AntiBoorScheme(u, deg, K[0], dim, P[0], depth, length, tol)); break;
    case 3: BSplCLib::Derivative(deg, K[0], dim, length, depth, P[0]); break;
    default: BSplCLib::Bohm(u, deg, depth, K[0], dim, P[0]);
  }
  ob(P); std::cout << '\n';
}

// build: deg index periodic usemults knots mults | weights poles(dim 1) p3 | boor: index length dim poles | getpole
void build_case(std::istream& s) {
  In in{s};
  int deg = in.i(), index = in.i(); bool per = in.i() != 0, use = in.i() != 0;
  auto K = in.reals(); auto M = in.ints();
  std::vector<double> lk(2 * deg > 0 ? 2 * deg : 1, 91.0);
  try { BSplCLib::BuildKnots(deg, index, per, K, use ? &M : BSplCLib::NoMults(), lk[0]); ob(lk); } catch (const Standard_Failure& f) { fail(f); }
  int eindex = in.i(); bool rat = in.i() != 0; auto W = in.reals(); auto P1 = in.reals(); auto P3 = in.p3(); auto P2 = in.p2();
  const NCollection_Array1<double>* w = rat ? &W : BSplCLib::NoWeights();
  std::vector<double> lp(4 * (deg + 1), 91.0);
  try { BSplCLib::BuildEval(deg, eindex, P1, w, lp[0]); ob(lp); } catch (const Standard_Failure& f) { fail(f); }
  std::fill(lp.begin(), lp.end(), 91.0);
  try { BSplCLib::BuildEval(deg, eindex, P3, w, lp[0]); ob(lp); } catch (const Standard_Failure& f) { fail(f); }
  std::fill(lp.begin(), lp.end(), 91.0);
  try { BSplCLib::BuildEval(deg, eindex, P2, w, lp[0]); ob(lp); } catch (const Standard_Failure& f) { fail(f); }
  int bindex = in.i(), blength = in.i(), bdim = in.i(), depth = in.i();
  std::vector<double> boor((2 * blength + 1) * bdim, 91.0);
  try { BSplCLib::BuildBoor(bindex, blength, bdim, P1, boor[0]); ob(boor); } catch (const Standard_Failure& f) { fail(f); }
  for (int k = -1; k <= blength + 2; ++k) oi(BSplCLib::BoorIndex(k, blength, depth));
  NCollection_Array1<double> pole(1, bdim * 2); pole.Init(91.0); int pos = in.i();
  for (int k = 0; k <= blength && BSplCLib::BoorIndex(k, blength, depth) <= 2 * blength; ++k) {
    try { BSplCLib::GetPole(k, blength, depth, bdim, boor[0], pos, pole); oi(pos); } catch (const Standard_Failure& f) { fail(f); break; }
  }
  oa(pole); std::cout << '\n';
}
} // namespace bspl_probe
