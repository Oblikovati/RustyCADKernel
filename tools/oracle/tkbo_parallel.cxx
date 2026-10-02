// Oracle for BOPAlgo_ParallelAlgo in pinned BOPAlgo_Algo.hxx.
#include <BOPAlgo_Algo.hxx>
#include <Message_ProgressIndicator.hxx>
#include <Message_ProgressScope.hxx>
#include <cassert>

class QuietProgress : public Message_ProgressIndicator
{
public:
  void Show(const Message_ProgressScope&, const bool) override {}
};

class ParallelTask : public BOPAlgo_ParallelAlgo
{
public:
  int runs = 0;
  bool active() const { return myProgressRange.IsActive(); }
  void Perform() override
  {
    ++runs;
    Message_ProgressScope scope(myProgressRange, "", 1);
  }
};

void checkDispatchAndTransfer()
{
  occ::handle<QuietProgress> indicator = new QuietProgress;
  Message_ProgressRange range = indicator->Start();
  ParallelTask task;
  task.SetProgressRange(range);
  assert(!range.IsActive() && task.active());
  BOPAlgo_Algo& base = task;
  base.Perform();
  assert(task.runs == 0 && indicator->GetPosition() == 0);
  task.Perform();
  assert(task.runs == 1 && indicator->GetPosition() == 1);
  assert(!task.active());
}

void checkAssignment()
{
  occ::handle<QuietProgress> indicator = new QuietProgress;
  Message_ProgressScope scope(indicator->Start(), "", 4);
  Message_ProgressRange first = scope.Next();
  Message_ProgressRange second = scope.Next();
  ParallelTask task;
  task.SetProgressRange(first);
  task.SetProgressRange(second);
  assert(!first.IsActive() && !second.IsActive());
  assert(indicator->GetPosition() == 0);
  task.Perform();
  assert(indicator->GetPosition() == 0.25);
}

int main()
{
  checkDispatchAndTransfer();
  checkAssignment();
}
