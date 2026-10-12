// Actual frame assignment must preserve the temporary frame's timer history; OCCT 3d097a0.
#include <Graphic3d_FrameStatsData.hxx>
#include <iomanip>
#include <iostream>

class AssignmentFrame : public Graphic3d_FrameStatsDataTmp {
public:
  using Graphic3d_FrameStatsDataTmp::operator=;
  void record(int step, const OSD_Timer* original) {
    std::cout << std::setprecision(17) << step << ' ' << FrameRate() << ' '
      << FrameRateCpu() << ' ' << ImmediateFrameRate() << ' '
      << ImmediateFrameRateCpu() << ' ' << CounterValue(Graphic3d_FrameStatsCounter_NbLayers)
      << ' ' << TimerValue(Graphic3d_FrameStatsTimer_CpuFrame) << ' '
      << myTimersMin[Graphic3d_FrameStatsTimer_CpuFrame] << ' '
      << myTimersMax[Graphic3d_FrameStatsTimer_CpuFrame] << ' '
      << ChangeTimer(Graphic3d_FrameStatsTimer_CpuFrame).IsStarted() << ' '
      << (&ChangeTimer(Graphic3d_FrameStatsTimer_CpuFrame) == original) << '\n';
  }
};

static void seedSource(AssignmentFrame& source) {
  source.ChangeFrameRate() = 60; source.ChangeFrameRateCpu() = 90;
  source.ChangeImmediateFrameRate() = 120; source.ChangeImmediateFrameRateCpu() = 180;
  source.ChangeCounterValue(Graphic3d_FrameStatsCounter_NbLayers) = 42;
  source.ChangeTimerValue(Graphic3d_FrameStatsTimer_CpuFrame) = 3;
  source.FlushTimers(1, false);
}

int main() {
  AssignmentFrame target, source; seedSource(source);
  OSD_Timer* timer = &target.ChangeTimer(Graphic3d_FrameStatsTimer_CpuFrame);
  timer->Start(); target.ChangeTimerValue(Graphic3d_FrameStatsTimer_CpuFrame) = 8;
  target.FlushTimers(1, false); target.record(0, timer);
  target = static_cast<const Graphic3d_FrameStatsData&>(source); target.record(1, timer);
  target.ChangeTimerValue(Graphic3d_FrameStatsTimer_CpuFrame) = 11;
  target.FlushTimers(1, false); target.record(2, timer);
  source.ChangeTimerValue(Graphic3d_FrameStatsTimer_CpuFrame) = 4;
  source.FlushTimers(1, false);
  target = static_cast<const Graphic3d_FrameStatsData&>(source); target.record(3, timer);
  target.ChangeTimerValue(Graphic3d_FrameStatsTimer_CpuFrame) = 16;
  target.FlushTimers(2, true); target.record(4, timer);
  timer->Stop(); target.Reset(); target.record(5, timer);
}
