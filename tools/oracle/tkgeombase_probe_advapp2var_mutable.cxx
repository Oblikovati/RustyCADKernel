// Defined mutation of the evaluator's derivative-order pointer in mma1fdi_.
#include <array>
#include <vector>
#include <AdvApp2Var_ApproxF2var.hxx>

class GbAaMutatingOrder : public AdvApp2Var_EvaluatorFunc2Var {
public:
  mutable std::vector<std::array<int, 3>> calls;

  void Evaluate(int* dim, double*, double*, int*, double* constant, int* count, double* params,
                int* du, int* dv, double* result, int* error) const override {
    calls.push_back({*count, *du, *dv});
    for (int i = 0; i < *count; ++i)
      result[i * *dim] = *constant + params[i] + 10. * *du + 100. * *dv;
    if (*count == 1 && *du == 0 && *dv == 1)
      *dv = 2;
    *error = 0;
  }
};

void gb_aa_mma2fnc_mut_order_case(std::istream&) {
  int dim = 1, spaces = 1, subdim = 1, iso = 1, nbroot = 8;
  int order = 2, derivative = 0, degree = 12, pieces = 2, coeffs = 10, done = -1, error = -9;
  double domain[4] = {0., 1., 0., 1.}, constant = .3, tolerance = 1e-6;
  GbAaMutatingOrder function;
  auto roots = gb_aa_fill(nbroot), other_roots = gb_aa_fill(nbroot);
  auto curve = gb_aa_fill(coeffs * dim * pieces);
  auto sums = gb_aa_fill((nbroot / 2 + 1) * dim * pieces), differences = sums;
  auto first = gb_aa_fill(dim * (order + 2) * pieces), second = first;
  auto cuts = gb_aa_fill(pieces + 1), max_errors = gb_aa_fill(pieces), mean_errors = max_errors;
  std::vector<int> coefficients(pieces, -1);
  AdvApp2Var_ApproxF2var::mma2roo_(&nbroot, &nbroot, roots.data(), other_roots.data());
  AdvApp2Var_ApproxF2var::mma2fnc_(
    &dim, &spaces, &subdim, domain, function, &constant, &iso, &nbroot, roots.data(), &order,
    &derivative, &degree, &pieces, &coeffs, &tolerance, coefficients.data(), curve.data(), &done,
    sums.data(), differences.data(), first.data(), second.data(), cuts.data(), max_errors.data(),
    mean_errors.data(), &error);
  gb_int(error); gb_int(done); gb_int(static_cast<int>(function.calls.size()));
  for (const auto& call : function.calls) { gb_int(call[0]); gb_int(call[1]); gb_int(call[2]); }
  gb_end();
}
