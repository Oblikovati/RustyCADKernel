// LU_Decompose and math_Gauss driven through a user progress indicator that can cancel.
// The Rust twin is crates/tkmath/examples/progress_probe/mod.rs.
#include <Message_ProgressIndicator.hxx>
#include <Message_ProgressScope.hxx>
#include <math_Gauss.hxx>
#include <math_Recipes.hxx>
#include <string>
#include <vector>
namespace progress_probe {
using vector_base_probe::cells;
using vector_base_probe::values;
// Records every Show and breaks once UserBreak has been polled more than `allowed` times
// (a negative `allowed` never breaks).
class CancellingIndicator : public Message_ProgressIndicator {
public:
  explicit CancellingIndicator(int allowed) : myAllowed(allowed), myPolls(0) {}
  int Polls() const { return myPolls; }
  const std::vector<std::string>& Shows() const { return myShows; }
protected:
  bool UserBreak() override { ++myPolls; return myAllowed >= 0 && myPolls > myAllowed; }
  void Show(const Message_ProgressScope& theScope, const bool isForce) override {
    std::string name = theScope.Name() == nullptr ? "~" : theScope.Name();
    for (char& ch : name) if (ch == ' ') ch = '_';
    myShows.push_back(std::to_string(bits(GetPosition())) + ' ' + name + ' ' + std::to_string(isForce));
  }
private:
  int myAllowed;
  int myPolls;
  std::vector<std::string> myShows;
};
void report(const CancellingIndicator& indicator) {
  std::cout << indicator.Polls() << ' ' << indicator.Shows().size() << ' ';
  for (const auto& show : indicator.Shows()) std::cout << show << ' ';
}
math_Matrix read_square(std::istream& in, int n) {
  math_Matrix m(1, n, 1, n);
  for (int r = 1; r <= n; ++r) for (int c = 1; c <= n; ++c) in >> m(r, c);
  return m;
}
// n allowed_polls with_scales tiny cells[n*n]
void lu_case(std::istream& in) {
  int n, allowed, with_scales; double tiny;
  in >> n >> allowed >> with_scales >> tiny;
  math_Matrix a = read_square(in, n);
  math_IntegerVector indx(1, n); indx.Init(0);
  double d = 91;
  int status;
  occ::handle<CancellingIndicator> indicator = new CancellingIndicator(allowed);
  {
    Message_ProgressRange range = indicator->Start();
    if (with_scales) { math_Vector vv(1, n); vv.Init(91); status = LU_Decompose(a, indx, d, vv, tiny, range); }
    else status = LU_Decompose(a, indx, d, tiny, range);
  }
  std::cout << status << ' ' << bits(d) << ' '; cells(a); values(indx); report(*indicator);
  std::cout << bits(indicator->GetPosition()) << '\n';
}
// n allowed_polls min_pivot cells[n*n]
void gauss_case(std::istream& in) {
  int n, allowed; double pivot;
  in >> n >> allowed >> pivot;
  math_Matrix a = read_square(in, n);
  occ::handle<CancellingIndicator> indicator = new CancellingIndicator(allowed);
  math_Gauss gauss(a, pivot, indicator->Start());
  std::cout << gauss.IsDone() << ' ';
  if (gauss.IsDone()) std::cout << bits(gauss.Determinant()) << ' ';
  report(*indicator);
  std::cout << bits(indicator->GetPosition()) << '\n';
}
} // namespace progress_probe
