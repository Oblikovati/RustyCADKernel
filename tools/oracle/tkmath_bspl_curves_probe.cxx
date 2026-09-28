// BSplCLib knot insertion/removal, degree elevation, unperiodization and trimming.
namespace bspl_probe {
// kind (1 flat, 2 gp_Pnt2d, 3 gp_Pnt) dim rational deg periodic knots mults poles weights
struct Curve {
  int kind, dim; bool rat; int deg; bool per;
  NCollection_Array1<double> K; NCollection_Array1<int> M; NCollection_Array1<double> F;
  NCollection_Array1<gp_Pnt2d> P2; NCollection_Array1<gp_Pnt> P3; NCollection_Array1<double> W;
  const NCollection_Array1<double>* w() const { return rat ? &W : BSplCLib::NoWeights(); }
  int count() const { return kind == 1 ? F.Length() / dim : (kind == 2 ? P2.Length() : P3.Length()); }
};
Curve curve(In& in) {
  Curve c; c.kind = in.i(); c.dim = in.i(); c.rat = in.i() != 0; c.deg = in.i(); c.per = in.i() != 0;
  c.K = in.reals(); c.M = in.ints();
  if (c.kind == 1) c.F = in.reals(); else if (c.kind == 2) c.P2 = in.p2(); else c.P3 = in.p3();
  c.W = in.reals();
  return c;
}
// Output arrays of a curve operation, sized as requested.
struct Out {
  NCollection_Array1<double> F, W, K; NCollection_Array1<int> M; NCollection_Array1<gp_Pnt2d> P2; NCollection_Array1<gp_Pnt> P3;
  Out(const Curve& c, int np, int nk) : F(1, c.kind == 1 ? np * c.dim : 0), W(1, np), K(1, nk), M(1, nk), P2(1, c.kind == 2 ? np : 0), P3(1, c.kind == 3 ? np : 0) {
    F.Init(91.0); W.Init(91.0); K.Init(91.0); M.Init(-7); P2.Init(gp_Pnt2d(91, 91)); P3.Init(gp_Pnt(91, 91, 91));
  }
  NCollection_Array1<double>* w(const Curve& c) { return c.rat ? &W : nullptr; }
  void print(const Curve& c, bool knots) { if (c.kind == 1) oa(F); else if (c.kind == 2) opa(P2); else opa(P3); if (c.rat) oa(W); if (knots) { oa(K); oia(M); } }
};

// insert: curve addknots useaddmults addmults tol add | single: u umult | raise: index mult
void insert_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); auto AK = in.reals(); bool useam = in.i() != 0; auto AM = in.ints(); double tol = in.r(); bool add = in.i() != 0;
  const NCollection_Array1<int>* am = useam ? &AM : nullptr;
  int np = -7, nk = -7;
  bool ok = BSplCLib::PrepareInsertKnots(c.deg, c.per, c.K, c.M, AK, am, np, nk, tol, add);
  oi(ok); oi(np); oi(nk);
  if (ok) {
    try {
      Out r(c, np, nk);
      if (c.kind == 1) BSplCLib::InsertKnots(c.deg, c.per, c.dim, c.F, c.K, c.M, AK, am, r.F, r.K, r.M, tol, add);
      else if (c.kind == 2) BSplCLib::InsertKnots(c.deg, c.per, c.P2, c.w(), c.K, c.M, AK, am, r.P2, r.w(c), r.K, r.M, tol, add);
      else BSplCLib::InsertKnots(c.deg, c.per, c.P3, c.w(), c.K, c.M, AK, am, r.P3, r.w(c), r.K, r.M, tol, add);
      r.print(c, true);
    } catch (const Standard_Failure& f) { fail(f); }
  }
  double u = in.r(); int umult = in.i(), index = in.i(), mult = in.i();
  if (c.kind == 1) { std::cout << '\n'; return; }
  NCollection_Array1<double> K1(1, 1); K1(1) = u; NCollection_Array1<int> M1(1, 1); M1(1) = umult;
  if (BSplCLib::PrepareInsertKnots(c.deg, c.per, c.K, c.M, K1, &M1, np, nk, Epsilon(u), true)) {
    Out r(c, np, 0); oi(np);
    try {
      if (c.kind == 2) BSplCLib::InsertKnot(0, u, umult, c.deg, c.per, c.P2, c.w(), c.K, c.M, r.P2, r.w(c));
      else BSplCLib::InsertKnot(0, u, umult, c.deg, c.per, c.P3, c.w(), c.K, c.M, r.P3, r.w(c));
      r.print(c, false);
    } catch (const Standard_Failure& f) { fail(f); }
  }
  K1(1) = c.K(index); M1(1) = mult - c.M(index);
  if (BSplCLib::PrepareInsertKnots(c.deg, c.per, c.K, c.M, K1, &M1, np, nk, Epsilon(c.K(index)), true)) {
    Out r(c, np, 0); oi(np);
    try {
      if (c.kind == 2) BSplCLib::RaiseMultiplicity(index, mult, c.deg, c.per, c.P2, c.w(), c.K, c.M, r.P2, r.w(c));
      else BSplCLib::RaiseMultiplicity(index, mult, c.deg, c.per, c.P3, c.w(), c.K, c.M, r.P3, r.w(c));
      r.print(c, false);
    } catch (const Standard_Failure& f) { fail(f); }
  }
  std::cout << '\n';
}

// remove: curve index mult tol newpoles newknots
void remove_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); int index = in.i(), mult = in.i(); double tol = in.r(); int np = in.i(), nk = in.i();
  Out r(c, np, nk); bool ok = false;
  try {
    if (c.kind == 1) ok = BSplCLib::RemoveKnot(index, mult, c.deg, c.per, c.dim, c.F, c.K, c.M, r.F, r.K, r.M, tol);
    else if (c.kind == 2) ok = BSplCLib::RemoveKnot(index, mult, c.deg, c.per, c.P2, c.w(), c.K, c.M, r.P2, r.w(c), r.K, r.M, tol);
    else ok = BSplCLib::RemoveKnot(index, mult, c.deg, c.per, c.P3, c.w(), c.K, c.M, r.P3, r.w(c), r.K, r.M, tol);
    oi(ok); r.print(c, true);
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// degree: curve newdeg newpoles
void degree_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); int nd = in.i(), np = in.i();
  try {
    int nk = BSplCLib::IncreaseDegreeCountKnots(c.deg, nd, c.per, c.M); oi(nk);
    Out r(c, np, nk);
    if (c.kind == 1) BSplCLib::IncreaseDegree(c.deg, nd, c.per, c.dim, c.F, c.K, c.M, r.F, r.K, r.M);
    else if (c.kind == 2) BSplCLib::IncreaseDegree(c.deg, nd, c.per, c.P2, c.w(), c.K, c.M, r.P2, r.w(c), r.K, r.M);
    else BSplCLib::IncreaseDegree(c.deg, nd, c.per, c.P3, c.w(), c.K, c.M, r.P3, r.w(c), r.K, r.M);
    r.print(c, true);
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// unperiodize: curve
void unperiodize_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); int nk = -7, np = -7;
  try {
    BSplCLib::PrepareUnperiodize(c.deg, c.M, nk, np); oi(nk); oi(np);
    Out r(c, np, nk);
    if (c.kind == 1) BSplCLib::Unperiodize(c.deg, c.dim, c.M, c.K, c.F, r.M, r.K, r.F);
    else if (c.kind == 2) BSplCLib::Unperiodize(c.deg, c.M, c.K, c.P2, c.w(), r.M, r.K, r.P2, r.w(c));
    else BSplCLib::Unperiodize(c.deg, c.M, c.K, c.P3, c.w(), r.M, r.K, r.P3, r.w(c));
    r.print(c, true);
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}

// trim: curve u1 u2
void trim_case(std::istream& s) {
  In in{s};
  Curve c = curve(in); double u1 = in.r(), u2 = in.r(); int nk = -7, np = -7;
  try {
    BSplCLib::PrepareTrimming(c.deg, c.per, c.K, c.M, u1, u2, nk, np); oi(nk); oi(np);
    Out r(c, np, nk);
    if (c.kind == 1) BSplCLib::Trimming(c.deg, c.per, c.dim, c.K, c.M, c.F, u1, u2, r.K, r.M, r.F);
    else if (c.kind == 2) BSplCLib::Trimming(c.deg, c.per, c.K, c.M, c.P2, c.w(), u1, u2, r.K, r.M, r.P2, r.w(c));
    else BSplCLib::Trimming(c.deg, c.per, c.K, c.M, c.P3, c.w(), u1, u2, r.K, r.M, r.P3, r.w(c));
    r.print(c, true);
  } catch (const Standard_Failure& f) { fail(f); }
  std::cout << '\n';
}
} // namespace bspl_probe
