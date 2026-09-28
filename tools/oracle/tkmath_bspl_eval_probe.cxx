// BSplCLib evaluation: D0..DN, flat-knot Eval, basis, banded solves, interpolation,
// caches and Bezier syntaxes.
namespace bspl_probe {
template <class P, class V>
void points_eval(double u, int index, const Curve& c, const NCollection_Array1<P>& poles, const NCollection_Array1<int>* m, int n) {
  P p; V v1, v2, v3;
  try { BSplCLib::D0(u, index, c.deg, c.per, poles, c.w(), c.K, m, p); op(p); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplCLib::D1(u, index, c.deg, c.per, poles, c.w(), c.K, m, p, v1); op(p); op(v1); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplCLib::D2(u, index, c.deg, c.per, poles, c.w(), c.K, m, p, v1, v2); op(p); op(v1); op(v2); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplCLib::D3(u, index, c.deg, c.per, poles, c.w(), c.K, m, p, v1, v2, v3); op(p); op(v1); op(v2); op(v3); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplCLib::DN(u, n, index, c.deg, c.per, poles, c.w(), c.K, m, v1); op(v1); } catch (const Standard_Failure& f) { fail(f); }
}

// eval: curve flat u index n
void eval_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); bool flat = in.i() != 0; double u = in.r(); int index = in.i(), n = in.i();
  const NCollection_Array1<int>* m = flat ? BSplCLib::NoMults() : &c.M;
  if (c.kind == 2) { points_eval<gp_Pnt2d, gp_Vec2d>(u, index, c, c.P2, m, n); std::cout << '\n'; return; }
  if (c.kind == 3) { points_eval<gp_Pnt, gp_Vec>(u, index, c, c.P3, m, n); std::cout << '\n'; return; }
  double p = 91, v1 = 91, v2 = 91, v3 = 91;
  try { BSplCLib::D0(u, index, c.deg, c.per, c.F, c.w(), c.K, m, p); o(p); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplCLib::D1(u, index, c.deg, c.per, c.F, c.w(), c.K, m, p, v1); o(p); o(v1); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplCLib::D2(u, index, c.deg, c.per, c.F, c.w(), c.K, m, p, v1, v2); o(p); o(v1); o(v2); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplCLib::D3(u, index, c.deg, c.per, c.F, c.w(), c.K, m, p, v1, v2, v3); o(p); o(v1); o(v2); o(v3); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplCLib::DN(u, n, index, c.deg, c.per, c.F, c.w(), c.K, m, v1); o(v1); } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// flat_eval: curve(knots = flat knots) param request em0 em1 homogeneous
void flat_eval_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); double u = in.r(); int req = in.i(); int em[2]; em[0] = in.i(); em[1] = in.i(); bool hom = in.i() != 0;
  int dim = c.kind == 1 ? c.dim : c.kind;
  std::vector<double> flat(c.kind == 1 ? c.F.Length() : c.count() * dim);
  for (int k = 0; k < c.count(); ++k) for (int d = 0; d < dim; ++d)
    flat[k * dim + d] = c.kind == 1 ? c.F(c.F.Lower() + k * dim + d) : (c.kind == 2 ? c.P2(c.P2.Lower() + k).Coord(d + 1) : c.P3(c.P3.Lower() + k).Coord(d + 1));
  std::vector<double> res((req + 1) * dim, 91.0), wres((req + 1) * dim + 1, 91.0);
  try { BSplCLib::Eval(u, c.per, req, em[0], c.deg, c.K, dim, flat[0], res[0]); ob(res); } catch (const Standard_Failure& f) { fail(f); }
  if (c.rat) {
    std::fill(res.begin(), res.end(), 91.0);
    std::vector<double> wv(c.W.begin(), c.W.end());
    try { BSplCLib::Eval(u, c.per, req, em[0], c.deg, c.K, dim, flat[0], wv[0], res[0], wres[0]); ob(res); ob(wres); } catch (const Standard_Failure& f) { fail(f); }
  }
  if (c.kind > 1 && c.rat) {
    double w = 91.0;
    try {
      if (c.kind == 2) { gp_Pnt2d p(91, 91); BSplCLib::Eval(u, c.per, hom, em[0], c.deg, c.K, c.P2, c.W, p, w); op(p); }
      else { gp_Pnt p(91, 91, 91); BSplCLib::Eval(u, c.per, hom, em[0], c.deg, c.K, c.P3, c.W, p, w); op(p); }
      o(w);
    } catch (const Standard_Failure& f) { fail(f); }
  }
  std::cout << '\n';
}

// basis: request order flat param periodic rows cols
void basis_case(std::istream& s) {
  In in{s};
  int req = in.i(), order = in.i(); auto K = in.reals(); double u = in.r(); bool per = in.i() != 0; int rows = in.i(), cols = in.i();
  math_Matrix B(1, rows, 1, cols); B.Init(91.0); int first = -7;
  try {
    oi(BSplCLib::EvalBsplineBasis(req, order, K, u, first, B, per)); oi(first);
    for (int r = 1; r <= rows; ++r) for (int q = 1; q <= cols; ++q) o(B(r, q));
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

void matrix_out(const math_Matrix& m) { for (int r = m.LowerRow(); r <= m.UpperRow(); ++r) for (int q = m.LowerCol(); q <= m.UpperCol(); ++q) o(m(r, q)); }

// interpolate: curve(knots = flat, poles = values) params contacts
void interpolate_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); auto T = in.reals(); auto C = in.ints();
  int dim = c.kind == 1 ? c.dim : c.kind, n = c.K.Length() - c.deg - 1;
  NCollection_Array1<double> S(1, n > 0 ? n : 0);
  try { BSplCLib::BuildSchoenbergPoints(c.deg, c.K, S); oa(S); } catch (const Standard_Failure& f) { fail(f); }
  math_Matrix M(T.Lower(), T.Upper(), 1, 2 * c.deg + 1); M.Init(91.0); int up = -7, lo = -7;
  int code = BSplCLib::BuildBSpMatrix(T, C, c.K, c.deg, M, up, lo); oi(code); oi(up); oi(lo); matrix_out(M);
  int pivot = -7; code = BSplCLib::FactorBandedMatrix(M, up, lo, pivot); oi(code); oi(pivot); matrix_out(M);
  std::vector<double> values(T.Length() * dim), weights(T.Length());
  for (int k = 0; k < T.Length(); ++k) {
    weights[k] = c.W.IsEmpty() ? 1.0 : c.W(c.W.Lower() + k);
    for (int d = 0; d < dim; ++d) values[k * dim + d] = c.kind == 1 ? c.F(c.F.Lower() + k * dim + d) : (c.kind == 2 ? c.P2(c.P2.Lower() + k).Coord(d + 1) : c.P3(c.P3.Lower() + k).Coord(d + 1));
  }
  auto v = values; auto w = weights;
  oi(BSplCLib::SolveBandedSystem(M, up, lo, dim, v[0])); ob(v);
  v = values; oi(BSplCLib::SolveBandedSystem(M, up, lo, false, dim, v[0], w[0])); ob(v); ob(w);
  v = values; w = weights; oi(BSplCLib::SolveBandedSystem(M, up, lo, true, dim, v[0], w[0])); ob(v); ob(w);
  int problem = -7; v = values;
  try { BSplCLib::Interpolate(c.deg, c.K, T, C, dim, v[0], problem); oi(problem); ob(v); } catch (const Standard_Failure& f) { oi(problem); fail(f); }
  problem = -7; v = values; w = weights;
  try { BSplCLib::Interpolate(c.deg, c.K, T, C, dim, v[0], w[0], problem); oi(problem); ob(v); ob(w); } catch (const Standard_Failure& f) { oi(problem); fail(f); }
  if (c.kind > 1) {
    NCollection_Array1<double> W(1, T.Length()); for (int k = 1; k <= T.Length(); ++k) W(k) = weights[k - 1];
    problem = -7;
    try {
      if (c.kind == 2) { auto P = c.P2; BSplCLib::Interpolate(c.deg, c.K, T, C, P, problem); oi(problem); opa(P); P = c.P2; BSplCLib::Interpolate(c.deg, c.K, T, C, P, W, problem); opa(P); oa(W); }
      else { auto P = c.P3; BSplCLib::Interpolate(c.deg, c.K, T, C, P, problem); oi(problem); opa(P); P = c.P3; BSplCLib::Interpolate(c.deg, c.K, T, C, P, W, problem); opa(P); oa(W); }
    } catch (const Standard_Failure& f) { oi(problem); fail(f); }
    if (c.kind == 2) { auto P = c.P2; oi(BSplCLib::SolveBandedSystem(M, up, lo, P)); opa(P); }
    else { auto P = c.P3; oi(BSplCLib::SolveBandedSystem(M, up, lo, P)); opa(P); }
  }
  std::cout << '\n';
}

template <class P, class V>
void cache_points(const Curve& c, const NCollection_Array1<P>& poles, double u, double domain, int span, const std::vector<double>& params) {
  NCollection_Array1<P> CP(1, c.deg + 1); NCollection_Array1<double> CW(1, c.deg + 1); CW.Init(91.0);
  NCollection_Array1<double>* cw = c.rat ? &CW : nullptr;
  try {
    BSplCLib::BuildCache(u, domain, c.per, c.deg, c.K, poles, c.w(), CP, cw); opa(CP); if (c.rat) oa(CW);
    for (double t : params) {
      P p; V v1, v2, v3;
      BSplCLib::CacheD0(t, c.deg, u, domain, CP, cw, p); op(p);
      BSplCLib::CacheD1(t, c.deg, u, domain, CP, cw, p, v1); op(p); op(v1);
      BSplCLib::CacheD2(t, c.deg, u, domain, CP, cw, p, v1, v2); op(p); op(v1); op(v2);
      BSplCLib::CacheD3(t, c.deg, u, domain, CP, cw, p, v1, v2, v3); op(p); op(v1); op(v2); op(v3);
    }
  } catch (const Standard_Failure& f) { fail(f); }
  int dim = c.kind + (c.rat ? 1 : 0);
  NCollection_Array2<double> A(1, c.deg + 1, 1, dim); A.Init(91.0);
  try { BSplCLib::BuildCache(u, domain, c.per, c.deg, span, c.K, poles, c.w(), A); for (int r = 1; r <= c.deg + 1; ++r) for (int q = 1; q <= dim; ++q) o(A(r, q)); } catch (const Standard_Failure& f) { fail(f); }
  try {
    BSplCLib_Cache cache(c.deg, c.per, c.K, poles, c.w());
    cache.BuildCache(u, c.K, poles, c.w());
    for (double t : params) {
      oi(cache.IsCacheValid(t));
      P p; V v1, v2, v3;
      cache.D0(t, p); op(p); cache.D1(t, p, v1); op(p); op(v1);
      cache.D2(t, p, v1, v2); op(p); op(v1); op(v2);
      cache.D3(t, p, v1, v2, v3); op(p); op(v1); op(v2); op(v3);
    }
  } catch (const Standard_Failure& f) { fail(f); }
}

// cache: curve(knots = flat) u domain span params
void cache_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); double u = in.r(), domain = in.r(); int span = in.i(); auto params = in.buf();
  if (c.kind == 2) cache_points<gp_Pnt2d, gp_Vec2d>(c, c.P2, u, domain, span, params);
  else cache_points<gp_Pnt, gp_Vec>(c, c.P3, u, domain, span, params);
  std::cout << '\n';
}

template <class P, class V>
void bezier_points(const Curve& c, const NCollection_Array1<P>& poles, int nd, double u) {
  P p; V v1, v2, v3;
  try { BSplCLib::D0(u, poles, c.w(), p); op(p); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplCLib::D1(u, poles, c.w(), p, v1); op(p); op(v1); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplCLib::D2(u, poles, c.w(), p, v1, v2); op(p); op(v1); op(v2); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplCLib::D3(u, poles, c.w(), p, v1, v2, v3); op(p); op(v1); op(v2); op(v3); } catch (const Standard_Failure& f) { fail(f); }
  NCollection_Array1<P> NP(1, nd + 1); NCollection_Array1<double> NW(1, nd + 1); NW.Init(91.0);
  try { BSplCLib::IncreaseDegree(nd, poles, c.w(), NP, c.rat ? &NW : nullptr); opa(NP); if (c.rat) oa(NW); } catch (const Standard_Failure& f) { fail(f); }
  NCollection_Array1<P> CP(1, poles.Length()); NCollection_Array1<double> CW(1, poles.Length()); CW.Init(91.0);
  NCollection_Array1<double>* cw = c.rat ? &CW : nullptr;
  try {
    BSplCLib::PolesCoefficients(poles, c.w(), CP, cw); opa(CP); if (c.rat) oa(CW);
    BSplCLib::CoefsD0(u, CP, cw, p); op(p); BSplCLib::CoefsD1(u, CP, cw, p, v1); op(p); op(v1);
    BSplCLib::CoefsD2(u, CP, cw, p, v1, v2); op(p); op(v1); op(v2);
    BSplCLib::CoefsD3(u, CP, cw, p, v1, v2, v3); op(p); op(v1); op(v2); op(v3);
  } catch (const Standard_Failure& f) { fail(f); }
}

// bezier: curve(poles only) newdeg u
void bezier_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); int nd = in.i(); double u = in.r();
  if (c.kind == 2) bezier_points<gp_Pnt2d, gp_Vec2d>(c, c.P2, nd, u); else bezier_points<gp_Pnt, gp_Vec>(c, c.P3, nd, u);
  int deg = c.count() - 1;
  try { const double* k = &BSplCLib::FlatBezierKnots(deg); for (int i = 0; i < 2 * (deg + 1); ++i) o(k[i]); } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}
} // namespace bspl_probe
