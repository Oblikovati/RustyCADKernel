// BSplSLib caches (static BuildCache/CacheD*, Bezier coefficients, BSplSLib_Cache),
// MovePoint, Resolution, Interpolate and FunctionMultiply at pinned OCCT 3d097a0.
namespace bsps_probe {
using bspl_probe::fail;
using bspl_probe::In;
using bspl_probe::o;
using bspl_probe::oi;
using bspl_probe::op;

// CacheD0/D1/D2 (or CoefsD0/D1/D2 when `coefs`) at (u, v) on cached coefficients.
void cache_evals(double u, double v, const double* span, const NCollection_Array2<gp_Pnt>& CP, const NCollection_Array2<double>* CW, int ud, int vd, bool coefs) {
  gp_Pnt P; gp_Vec Vu, Vv, Vuu, Vuv, Vvv;
  try {
    if (coefs) BSplSLib::CoefsD0(u, v, CP, CW, P); else BSplSLib::CacheD0(u, v, ud, vd, span[0], span[1], span[2], span[3], CP, CW, P);
    op(P);
  } catch (const Standard_Failure& f) { fail(f); }
  try {
    if (coefs) BSplSLib::CoefsD1(u, v, CP, CW, P, Vu, Vv); else BSplSLib::CacheD1(u, v, ud, vd, span[0], span[1], span[2], span[3], CP, CW, P, Vu, Vv);
    op(P); op(Vu); op(Vv);
  } catch (const Standard_Failure& f) { fail(f); }
  try {
    if (coefs) BSplSLib::CoefsD2(u, v, CP, CW, P, Vu, Vv, Vuu, Vuv, Vvv); else BSplSLib::CacheD2(u, v, ud, vd, span[0], span[1], span[2], span[3], CP, CW, P, Vu, Vv, Vuu, Vuv, Vvv);
    op(P); op(Vu); op(Vv); op(Vuu); op(Vuv); op(Vvv);
  } catch (const Standard_Failure& f) { fail(f); }
}

// Cache class: IsCacheValid, BuildCache when invalid, D0..D2, then D0Local..D2Local.
void cache_class(const Surf& s, BSplSLib_Cache& cache, double u, double v, double lu, double lv) {
  gp_Pnt P; gp_Vec Vu, Vv, Vuu, Vvv, Vuv;
  bool valid = cache.IsCacheValid(u, v); oi(valid);
  try {
    if (!valid) cache.BuildCache(u, v, s.UK, s.VK, s.P, s.w());
    cache.D0(u, v, P); op(P); cache.D1(u, v, P, Vu, Vv); op(P); op(Vu); op(Vv);
    cache.D2(u, v, P, Vu, Vv, Vuu, Vvv, Vuv); op(P); op(Vu); op(Vv); op(Vuu); op(Vvv); op(Vuv);
    cache.D0Local(lu, lv, P); op(P); cache.D1Local(lu, lv, P, Vu, Vv); op(P); op(Vu); op(Vv);
    cache.D2Local(lu, lv, P, Vu, Vv, Vuu, Vvv, Vuv); op(P); op(Vu); op(Vv); op(Vuu); op(Vvv); op(Vuv);
  } catch (const Standard_Failure& f) { fail(f); }
}

// cache: surface(flat knots) u v udomain vdomain uindex vindex ucache vcache ulength vlength
//        count (u v lu lv)...
void cache_case(std::istream& is) {
  In in{is};
  Surf s = surf(in); double u = in.r(), v = in.r(), ud = in.r(), vd = in.r(); int ui = in.i(), vi = in.i();
  double span[4] = {in.r(), in.r(), in.r(), in.r()};
  int mx = std::max(s.ud, s.vd) + 1, mn = std::min(s.ud, s.vd) + 1, dim = s.hw ? 4 : 3;
  NCollection_Array2<gp_Pnt> CP = pgrid(mx, mn); NCollection_Array2<double> CW = wgrid(mx, mn);
  NCollection_Array2<double>* cw = s.hw ? &CW : nullptr;
  try { BSplSLib::BuildCache(u, v, ud, vd, s.up, s.vp, s.ud, s.vd, ui, vi, s.UK, s.VK, s.P, s.w(), CP, cw); og(CP); if (cw) og(CW); } catch (const Standard_Failure& f) { fail(f); }
  NCollection_Array2<double> CA = wgrid(mx, dim * mn);
  try { BSplSLib::BuildCache(u, v, ud, vd, s.up, s.vp, s.ud, s.vd, ui, vi, s.UK, s.VK, s.P, s.w(), CA); og(CA); } catch (const Standard_Failure& f) { fail(f); }
  int n = in.i(); std::vector<double> pts(4 * n); for (double& x : pts) x = in.r();
  for (int k = 0; k < n; ++k) cache_evals(pts[4 * k], pts[4 * k + 1], span, CP, cw, s.ud, s.vd, false);
  try {
    BSplSLib_Cache cache(s.ud, s.up, s.UK, s.vd, s.vp, s.VK, s.w());
    for (int k = 0; k < n; ++k) cache_class(s, cache, pts[4 * k], pts[4 * k + 1], pts[4 * k + 2], pts[4 * k + 3]);
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// bezier: poles hasweights weights count (u v)...
void bezier_case(std::istream& is) {
  In in{is};
  auto P = grid3(in); bool hw = in.i() != 0; auto W = grid1(in);
  // The coefficients take the shape of the poles (see the PolesCoefficients quirk).
  NCollection_Array2<gp_Pnt> CP = pgrid(P.ColLength(), P.RowLength()); NCollection_Array2<double> CW = wgrid(P.ColLength(), P.RowLength());
  try { BSplSLib::PolesCoefficients(P, CP); og(CP); } catch (const Standard_Failure& f) { fail(f); }
  try { BSplSLib::PolesCoefficients(P, hw ? &W : nullptr, CP, hw ? &CW : nullptr); og(CP); if (hw) og(CW); } catch (const Standard_Failure& f) { fail(f); }
  int n = in.i();
  for (int k = 0; k < n; ++k) { double u = in.r(), v = in.r(); cache_evals(u, v, nullptr, CP, hw ? &CW : nullptr, 0, 0, true); }
  std::cout << '\n';
}

// move: u v dx dy dz ui1 ui2 vi1 vi2 udeg vdeg rational poles weights uflat vflat
void move_case(std::istream& is) {
  In in{is};
  double u = in.r(), v = in.r(), dx = in.r(), dy = in.r(), dz = in.r();
  int ui1 = in.i(), ui2 = in.i(), vi1 = in.i(), vi2 = in.i(), ud = in.i(), vd = in.i(); bool rat = in.i() != 0;
  auto P = grid3(in); auto W = grid1(in); auto UF = in.reals(); auto VF = in.reals();
  NCollection_Array2<gp_Pnt> NP(P.LowerRow(), P.UpperRow(), P.LowerCol(), P.UpperCol()); NP.Init(gp_Pnt(91, 91, 91));
  int uf = -7, ul = -7, vf = -7, vl = -7;
  try {
    BSplSLib::MovePoint(u, v, gp_Vec(dx, dy, dz), ui1, ui2, vi1, vi2, ud, vd, rat, P, W, UF, VF, uf, ul, vf, vl, NP);
    oi(uf); oi(ul); oi(vf); oi(vl); og(NP);
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// resolution: surface tolerance
void resolution_case(std::istream& is) {
  In in{is};
  Surf s = surf(in); double tol = in.r(), ut = 91, vt = 91;
  try { BSplSLib::Resolution(s.P, s.w(), s.UK, s.VK, s.UM, s.VM, s.ud, s.vd, s.ur, s.vr, s.up, s.vp, tol, ut, vt); o(ut); o(vt); } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// interpolate: udeg vdeg uflat vflat upar vpar rational values weights
void interpolate_case(std::istream& is) {
  In in{is};
  int ud = in.i(), vd = in.i(); auto UF = in.reals(); auto VF = in.reals(); auto UP = in.reals(); auto VP = in.reals();
  bool rat = in.i() != 0; auto P = grid3(in); auto W = grid1(in); int problem = -7;
  try {
    if (rat) BSplSLib::Interpolate(ud, vd, UF, VF, UP, VP, P, W, problem); else BSplSLib::Interpolate(ud, vd, UF, VF, UP, VP, P, problem);
    oi(problem); og(P); if (rat) og(W);
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

class ProbeSurfaceFunction : public BSplSLib_EvaluatorFunction {
public:
  double c[4], limit;
  void Evaluate(const int, const double u, const double v, double& r, int& e) const override {
    if (u > limit) { e = 1; return; }
    r = c[0] + u * c[1] + v * (c[2] + u * c[3]); e = 0;
  }
};

// function: surface unewflat vnewflat unewdeg vnewdeg c0 c1 c2 c3 limit
void function_case(std::istream& is) {
  In in{is};
  Surf s = surf(in); auto UF = in.reals(); auto VF = in.reals(); int und = in.i(), vnd = in.i();
  ProbeSurfaceFunction fn; for (double& x : fn.c) x = in.r(); fn.limit = in.r();
  int nu = UF.Length() - und - 1, nv = VF.Length() - vnd - 1, status = -7;
  NCollection_Array2<gp_Pnt> NN = pgrid(nu, nv); NCollection_Array2<double> ND = wgrid(nu, nv);
  try {
    BSplSLib::FunctionMultiply(fn, s.ud, s.vd, s.UK, s.VK, s.umults(), s.vmults(), s.P, s.w(), UF, VF, und, vnd, NN, ND, status);
    oi(status); og(NN); og(ND);
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}
} // namespace bsps_probe
