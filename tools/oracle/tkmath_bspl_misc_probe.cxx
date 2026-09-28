// BSplCLib point moves, resolution, function products, extension, knot merging
// and continuity intervals.
#include <NCollection_HArray1.hxx>
namespace bspl_probe {
std::vector<double> flat_of(const Curve& c) {
  int dim = c.kind == 1 ? c.dim : c.kind;
  std::vector<double> flat(c.count() * dim);
  for (int k = 0; k < c.count(); ++k) for (int d = 0; d < dim; ++d)
    flat[k * dim + d] = c.kind == 1 ? c.F(c.F.Lower() + k * dim + d) : (c.kind == 2 ? c.P2(c.P2.Lower() + k).Coord(d + 1) : c.P3(c.P3.Lower() + k).Coord(d + 1));
  return flat;
}

template <class P, class V>
void move_points(const Curve& c, const NCollection_Array1<P>& poles, double u, const V& displ, int i1, int i2, const V& delta, const V& dd, double tol, int start, int end) {
  NCollection_Array1<P> NP(poles.Lower(), poles.Upper()); int first = -7, last = -7;
  try { BSplCLib::MovePoint(u, displ, i1, i2, c.deg, poles, c.w(), c.K, first, last, NP); oi(first); oi(last); opa(NP); } catch (const Standard_Failure& f) { fail(f); }
  NP = poles; int status = -7;
  try { BSplCLib::MovePointAndTangent(u, delta, dd, tol, c.deg, start, end, poles, c.w(), c.K, NP, status); oi(status); opa(NP); } catch (const Standard_Failure& f) { fail(f); }
}

// move: curve(knots = flat) u displ delta dd (3 each) i1 i2 tol start end
void move_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); double u = in.r(); double d[9]; for (double& x : d) x = in.r();
  int i1 = in.i(), i2 = in.i(); double tol = in.r(); int start = in.i(), end = in.i();
  if (c.kind == 2) move_points(c, c.P2, u, gp_Vec2d(d[0], d[1]), i1, i2, gp_Vec2d(d[3], d[4]), gp_Vec2d(d[6], d[7]), tol, start, end);
  else move_points(c, c.P3, u, gp_Vec(d[0], d[1], d[2]), i1, i2, gp_Vec(d[3], d[4], d[5]), gp_Vec(d[6], d[7], d[8]), tol, start, end);
  int dim = c.kind; auto flat = flat_of(c); std::vector<double> np(flat.size(), 91.0); int status = -7;
  try { BSplCLib::MovePointAndTangent(u, dim, d[3], d[6], tol, c.deg, start, end, flat[0], c.w(), c.K, np[0], status); oi(status); ob(np); } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// resolution: curve(knots = flat) numpoles tol
void resolution_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); int num = in.i(); double tol = in.r(); double ut = 91.0;
  int dim = c.kind == 1 ? c.dim : c.kind; auto flat = flat_of(c);
  try { BSplCLib::Resolution(flat[0], dim, num, c.w(), c.K, c.deg, tol, ut); o(ut); } catch (const Standard_Failure& f) { fail(f); }
  ut = 91.0;
  try {
    if (c.kind == 2) { BSplCLib::Resolution(c.P2, c.w(), num, c.K, c.deg, tol, ut); o(ut); }
    else if (c.kind == 3) { BSplCLib::Resolution(c.P3, c.w(), num, c.K, c.deg, tol, ut); o(ut); }
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

class ProbeFunction : public BSplCLib_EvaluatorFunction {
public:
  double c0, c1, c2, limit;
  void Evaluate(const int, const double*, const double t, double& r, int& e) const override {
    if (t > limit) { e = 1; return; }
    r = c0 + t * (c1 + t * c2); e = 0;
  }
};

// function: curve(knots = bspline flat) newdeg newflat c0 c1 c2 limit reparam
void function_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); int nd = in.i(); auto NF = in.reals();
  ProbeFunction fn; fn.c0 = in.r(); fn.c1 = in.r(); fn.c2 = in.r(); fn.limit = in.r(); bool reparam = in.i() != 0;
  int dim = c.kind == 1 ? c.dim : c.kind; auto flat = flat_of(c);
  int nnew = NF.Length() - nd - 1; std::vector<double> np(nnew * dim > 0 ? nnew * dim : 1, 91.0); int status = -7;
  try {
    if (reparam) BSplCLib::FunctionReparameterise(fn, c.deg, c.K, dim, flat[0], NF, nd, np[0], status);
    else BSplCLib::FunctionMultiply(fn, c.deg, c.K, dim, flat[0], NF, nd, np[0], status);
    oi(status); ob(np);
  } catch (const Standard_Failure& f) { fail(f); }
  status = -7;
  try {
    if (c.kind == 1 && c.dim == 1) {
      NCollection_Array1<double> NP(1, nnew); NP.Init(91.0);
      if (reparam) BSplCLib::FunctionReparameterise(fn, c.deg, c.K, c.F, NF, nd, NP, status);
      else BSplCLib::FunctionMultiply(fn, c.deg, c.K, c.F, NF, nd, NP, status);
      oi(status); oa(NP);
    } else if (c.kind == 2) {
      NCollection_Array1<gp_Pnt2d> NP(1, nnew); NP.Init(gp_Pnt2d(91, 91));
      if (reparam) BSplCLib::FunctionReparameterise(fn, c.deg, c.K, c.P2, NF, nd, NP, status);
      else BSplCLib::FunctionMultiply(fn, c.deg, c.K, c.P2, NF, nd, NP, status);
      oi(status); opa(NP);
    } else if (c.kind == 3) {
      NCollection_Array1<gp_Pnt> NP(1, nnew); NP.Init(gp_Pnt(91, 91, 91));
      if (reparam) BSplCLib::FunctionReparameterise(fn, c.deg, c.K, c.P3, NF, nd, NP, status);
      else BSplCLib::FunctionMultiply(fn, c.deg, c.K, c.P3, NF, nd, NP, status);
      oi(status); opa(NP);
    }
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// tang: curve(kind 1, knots = flat) point continuity after c1
void tang_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); auto P = in.reals(); int cont = in.i(); bool after = in.i() != 0; double c1 = in.r();
  int num = c.F.Length() / c.dim, nbp = -7, nbk = -7;
  std::vector<double> kr(c.K.Length() + 2 * (c.deg + 1), 91.0), pr((num + c.deg) * c.dim, 91.0);
  std::vector<double> poles(c.F.begin(), c.F.end());
  try {
    BSplCLib::TangExtendToConstraint(c.K, c1, num, poles[0], c.dim, c.deg, P, cont, after, nbp, nbk, kr[0], pr[0]);
    oi(nbp); oi(nbk); for (int k = 0; k < nbk; ++k) o(kr[k]); for (int k = 0; k < nbp * c.dim; ++k) o(pr[k]);
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// merge: tol start end d1 k1 m1 d2 k2 m2
void merge_case(std::istream& s) {
  In in{s};
  double tol = in.r(), a = in.r(), b = in.r(); int d1 = in.i(); auto K1 = in.reals(); auto M1 = in.ints();
  int d2 = in.i(); auto K2 = in.reals(); auto M2 = in.ints();
  occ::handle<NCollection_HArray1<double>> NK; occ::handle<NCollection_HArray1<int>> NM; int num = -7;
  try { BSplCLib::MergeBSplineKnots(tol, a, b, d1, K1, M1, d2, K2, M2, num, NK, NM); oi(num); oa(NK->Array1()); oia(NM->Array1()); } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// intervals: deg periodic continuity first last tol knots mults
void intervals_case(std::istream& s) {
  In in{s};
  int deg = in.i(); bool per = in.i() != 0; int cont = in.i(); double first = in.r(), last = in.r(), tol = in.r();
  auto K = in.reals(); auto M = in.ints();
  NCollection_Array1<double> I(1, 1); I.Init(91.0);
  try { oi(BSplCLib::Intervals(K, M, deg, per, cont, first, last, tol, nullptr)); oi(BSplCLib::Intervals(K, M, deg, per, cont, first, last, tol, &I)); oa(I); } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}
} // namespace bspl_probe
