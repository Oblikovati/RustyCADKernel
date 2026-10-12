// Martensite Reset oracle: OCCT 3d097a0 must keep actual frame field addresses.
#include <Graphic3d_FrameStatsData.hxx>
#include <Standard_Real.hxx>
#include <algorithm>
#include <array>
#include <iostream>
#include <limits>

using ResetAddresses = std::array<const void *, 4>;
class ResetFrame : public Graphic3d_FrameStatsDataTmp {
public:
  ResetAddresses addresses() const {
    return {&myCounters[0], &myTimers[0], &myTimersMin[0], &myTimersMax[0]};
  }
  void seed() {
    std::fill(myCounters.begin(), myCounters.end(), 42);
    std::fill(myTimers.begin(), myTimers.end(),
              std::numeric_limits<double>::quiet_NaN());
    std::fill(myTimersMin.begin(), myTimersMin.end(), -3.0);
    std::fill(myTimersMax.begin(), myTimersMax.end(),
              std::numeric_limits<double>::infinity());
    ChangeFrameRate() = 60.0;
    ChangeFrameRateCpu() = 90.0;
    ChangeImmediateFrameRate() = 120.0;
    ChangeImmediateFrameRateCpu() = 180.0;
  }
  void record(int step, const ResetAddresses &original) const {
    const auto current = addresses();
    std::cout << step;
    for (size_t i = 0; i < current.size(); ++i)
      std::cout << ' ' << (current[i] == original[i]);
    std::cout << ' ' << resetValues() << '\n';
  }

private:
  bool resetValues() const {
    return FrameRate() == -1.0 && FrameRateCpu() == -1.0 &&
           ImmediateFrameRate() == -1.0 && ImmediateFrameRateCpu() == -1.0 &&
           std::all_of(myCounters.begin(), myCounters.end(),
                       [](size_t v) { return v == 0; }) &&
           std::all_of(myTimers.begin(), myTimers.end(),
                       [](double v) { return v == 0.0 && !std::signbit(v); }) &&
           std::all_of(myTimersMin.begin(), myTimersMin.end(),
                       [](double v) { return v == RealLast(); }) &&
           std::all_of(myTimersMax.begin(), myTimersMax.end(),
                       [](double v) { return v == 0.0 && !std::signbit(v); });
  }
};
int main() {
  ResetFrame frame;
  frame.seed();
  const auto original = frame.addresses();
  frame.Graphic3d_FrameStatsData::Reset();
  frame.record(1, original);
  frame.seed();
  frame.Reset();
  frame.record(2, original);
  frame.Reset();
  frame.record(3, original);
}
