// Behavioral oracle for BOPAlgo_Algo at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BOPAlgo_Algo.hxx>
#include <BOPAlgo_Alerts.hxx>
#include <cassert>
#include <cmath>
#include <limits>
#include <vector>

class WeightedAlgorithm : public BOPAlgo_Algo
{
public:
  std::vector<std::pair<int, double>> constants, variable;
  using BOPAlgo_Algo::analyzeProgress;
  using BOPAlgo_Algo::CheckData;
  using BOPAlgo_Algo::CheckResult;
  void Perform(const Message_ProgressRange&) override {}
  void fillPIConstants(const double, BOPAlgo_PISteps& steps) const override
  {
    for (const auto& item : constants) { steps.SetStep(item.first, item.second); }
  }
  void fillPISteps(BOPAlgo_PISteps& steps) const override
  {
    for (const auto& item : variable) { steps.SetStep(item.first, item.second); }
  }
};

void checkWeights(double whole, std::vector<std::pair<int, double>> constants,
                  std::vector<std::pair<int, double>> variable, std::vector<double> expected)
{
  WeightedAlgorithm algorithm;
  algorithm.constants = constants;
  algorithm.variable = variable;
  BOPAlgo_PISteps steps(3);
  algorithm.analyzeProgress(whole, steps);
  for (int index = 0; index < 3; ++index)
  {
    const double result = steps.GetStep(index);
    assert(std::isnan(expected[index]) ? std::isnan(result) : result == expected[index]);
  }
}

void checkBounds()
{
  BOPAlgo_PISteps steps(3);
  assert(steps.Steps().Lower() == 0 && steps.Steps().Upper() == 2);
  assert(steps.GetStep(0) == 0 && steps.GetStep(-1) == 0);
  steps.SetStep(-1, 9);
  steps.SetStep(3, 9);
  assert(steps.GetStep(3) == 0);
  steps.ChangeSteps().Init(2);
  assert(steps.GetStep(1) == 2);
  BOPAlgo_PISteps empty(0);
  assert(empty.Steps().IsEmpty());
}

void checkValidation()
{
  WeightedAlgorithm algorithm;
  algorithm.AddWarning(new Message_Alert);
  algorithm.AddError(new BOPAlgo_AlertUserBreak);
  algorithm.CheckData();
  assert(!algorithm.HasErrors() && algorithm.HasWarnings());
  algorithm.AddError(new BOPAlgo_AlertUserBreak);
  algorithm.CheckResult();
  assert(!algorithm.HasErrors() && algorithm.HasWarnings());
}

int main()
{
  checkBounds();
  checkValidation();
  checkWeights(100, {{0, 20}}, {{1, 2}, {2, 6}}, {20, 20, 60});
  checkWeights(100, {{0, 20}}, {{0, 50}, {1, 1}}, {50, 80, 0});
  checkWeights(10, {{0, 20}}, {{1, 1}, {2, 1}}, {20, -5, -5});
  checkWeights(100, {}, {{0, -2}, {1, 2}}, {-2, 2, 0});
  checkWeights(100, {}, {{0, -2}}, {-2, 0, 0});
  const double nan = std::numeric_limits<double>::quiet_NaN();
  checkWeights(100, {}, {{0, nan}, {1, 2}}, {nan, 2, 0});
  checkWeights(100, {{0, -2}}, {{1, 6}}, {-50, 150, 0});
  checkWeights(100, {}, {}, {0, 0, 0});
}
