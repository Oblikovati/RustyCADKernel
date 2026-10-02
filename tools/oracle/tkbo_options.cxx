// Behavioral oracle for BOPAlgo_Options at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BOPAlgo_Options.hxx>
#include <BOPAlgo_Alerts.hxx>
#include <Message_ProgressIndicator.hxx>
#include <Message_ProgressScope.hxx>
#include <Message_MsgFile.hxx>
#include <cassert>
#include <cmath>
#include <limits>
#include <sstream>

class ExposedOptions : public BOPAlgo_Options
{
public:
  using BOPAlgo_Options::UserBreak;
};

class CancelledProgress : public Message_ProgressIndicator
{
public:
  bool UserBreak() override { return true; }
  void Show(const Message_ProgressScope&, const bool) override {}
};

void checkDefaults()
{
  BOPAlgo_Options options;
  assert(options.Allocator() == NCollection_BaseAllocator::CommonBaseAllocator());
  assert(options.FuzzyValue() == 1e-7 && !options.UseOBB());
  assert(!options.HasErrors() && !options.HasWarnings());
  assert(Message_MsgFile::HasMsg("BOPAlgo_LOAD_CHECKER"));
  BOPAlgo_Options nullAllocator{occ::handle<NCollection_BaseAllocator>()};
  assert(nullAllocator.Allocator().IsNull());
}

void checkParallel()
{
  BOPAlgo_Options::SetParallelMode(true);
  BOPAlgo_Options enabled;
  BOPAlgo_Options::SetParallelMode(false);
  BOPAlgo_Options disabled;
  assert(enabled.RunParallel() && !disabled.RunParallel());
  enabled.SetRunParallel(false);
  assert(!enabled.RunParallel() && !BOPAlgo_Options::GetParallelMode());
}

void checkFuzzy()
{
  BOPAlgo_Options options;
  for (double tolerance : {-std::numeric_limits<double>::infinity(), -1.0, -0.0, 0.0, 1e-8, 1e-7})
  {
    options.SetFuzzyValue(tolerance);
    assert(options.FuzzyValue() == 1e-7);
  }
  options.SetFuzzyValue(std::numeric_limits<double>::quiet_NaN());
  assert(std::isnan(options.FuzzyValue()));
  options.SetFuzzyValue(INFINITY);
  assert(std::isinf(options.FuzzyValue()));
}

void checkReporting()
{
  BOPAlgo_Options options;
  options.AddError(new BOPAlgo_AlertUserBreak);
  options.AddWarning(new Message_Alert);
  assert(options.HasError(STANDARD_TYPE(BOPAlgo_AlertUserBreak)));
  assert(!options.HasWarning(STANDARD_TYPE(BOPAlgo_AlertUserBreak)));
  assert(options.HasWarning(STANDARD_TYPE(Message_Alert)));
  std::ostringstream errors, warnings;
  options.DumpErrors(errors);
  options.DumpWarnings(warnings);
  assert(errors.str() == "BOPAlgo_AlertUserBreak\n");
  assert(warnings.str() == "Message_Alert\n");
  options.ClearWarnings();
  assert(options.HasErrors() && !options.HasWarnings());
}

void checkClearAndCopy()
{
  BOPAlgo_Options options;
  options.SetUseOBB(true);
  options.SetRunParallel(true);
  options.SetFuzzyValue(0.01);
  options.AddError(new BOPAlgo_AlertUserBreak);
  BOPAlgo_Options copy = options;
  assert(copy.GetReport() == options.GetReport());
  options.Clear();
  assert(!copy.HasErrors());
  assert(options.UseOBB() && options.RunParallel() && options.FuzzyValue() == 0.01);
}

void checkCancellation()
{
  ExposedOptions options;
  assert(!options.UserBreak(Message_ProgressScope()));
  occ::handle<CancelledProgress> progress = new CancelledProgress;
  Message_ProgressScope scope(progress->Start(), "", 1);
  assert(options.UserBreak(scope));
  assert(options.HasError(STANDARD_TYPE(BOPAlgo_AlertUserBreak)));
  // Native Message_Report removes the earlier alert after a merge at limit -1.
  assert(options.UserBreak(scope));
  assert(options.GetReport()->GetAlerts(Message_Fail).IsEmpty());
}

int main()
{
  checkDefaults();
  checkParallel();
  checkFuzzy();
  checkReporting();
  checkClearAndCopy();
  checkCancellation();
}
