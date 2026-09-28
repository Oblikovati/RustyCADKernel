// OSD threads probe operations (OSD_Thread, OSD_ThreadPool, OSD_Parallel); twin of
// oracle_probe/osd/threads.rs. Only deterministic observables are printed: sizes and index
// bounds, coverage checksums, failure texts and thread results.
//
// osd_pool <n> <init> <def>        Pool sizes; then SetNbDefaultThreadsToLaunch(<def>) unless
//                                  it is -99 and Init(<init>) unless it is 0, and sizes again.
// osd_launcher <n> <max1> <max2>   Two launchers on one pool (the second sees it busy), a
//                                  Perform on the second, Release of the first, IsInUse.
// osd_perform <n> <max> <b> <e>    One Perform: count, index sum, visited-once, index bounds.
// osd_perform_fail <n> <max> <b> <e> <at>  Perform raising Standard_Failure at index <at>;
//                                  a single-thread launcher performs again (failure kept).
// osd_for <b> <e> <force>          OSD_Parallel::For coverage.
// osd_foreach <count> <nb> <force> OSD_Parallel::ForEach coverage over a vector.
// osd_nproc                        NbLogicalProcessors, ToUseOcctThreads/SetUseOcctThreads.
// osd_default_pool                 DefaultPool sizes.
// osd_thread <v>                   Run/Wait/GetId of a doubling thread and of empty threads.
// osd_thread_timed <ms>            Timed Wait before and after releasing a spinning thread.
// osd_thread_copy <v>              Copy-constructed, assigned and re-functioned threads.
#include <OSD_Parallel.hxx>
#include <OSD_Thread.hxx>
#include <OSD_ThreadPool.hxx>
#include <Standard_Failure.hxx>

#include <atomic>
#include <cstdint>
#include <memory>
#include <vector>

// Visit counters of one loop: total calls, index sum, per-slot counts, thread bounds.
struct OsdThreadsCoverage
{
  explicit OsdThreadsCoverage(int theBegin, int theEnd)
      : myBegin(theBegin),
        mySlots(theEnd > theBegin ? theEnd - theBegin : 0)
  {
    for (auto& aSlot : mySlots)
      aSlot = 0;
  }

  void Visit(int theIndex) const
  {
    ++myCount;
    mySum += theIndex;
    const int aSlot = theIndex - myBegin;
    if (aSlot >= 0 && aSlot < (int)mySlots.size())
      ++mySlots[aSlot];
    else
      myOutside = true;
  }

  void Thread(int theThread, int theLower, int theUpper) const
  {
    if (theThread < theLower || theThread > theUpper)
      myBadThread = true;
  }

  void Print(std::ostream& out) const
  {
    bool isOnce = !myOutside;
    for (const auto& aSlot : mySlots)
      isOnce = isOnce && aSlot == 1;
    out << myCount << ' ' << mySum << ' ' << isOnce << ' ' << !myBadThread;
  }

  int                                   myBegin;
  mutable std::vector<std::atomic<int>> mySlots;
  mutable std::atomic<int>              myCount{0};
  mutable std::atomic<int64_t>          mySum{0};
  mutable std::atomic<bool>             myOutside{false};
  mutable std::atomic<bool>             myBadThread{false};
};

static void osd_pool_sizes(std::ostream& out, OSD_ThreadPool& thePool)
{
  out << thePool.NbThreads() << ' ' << thePool.HasThreads() << ' ' << thePool.LowerThreadIndex()
      << ' ' << thePool.UpperThreadIndex() << ' ' << thePool.NbDefaultThreadsToLaunch();
}

static void osd_launcher_sizes(std::ostream& out, const OSD_ThreadPool::Launcher& theLauncher)
{
  out << theLauncher.NbThreads() << ' ' << theLauncher.HasThreads() << ' '
      << theLauncher.LowerThreadIndex() << ' ' << theLauncher.UpperThreadIndex();
}

static void osd_pool_probe(std::istream& in, std::ostream& out)
{
  int aNb = 0, anInit = 0, aDef = 0;
  in >> aNb >> anInit >> aDef;
  OSD_ThreadPool aPool(aNb);
  osd_pool_sizes(out, aPool);
  if (aDef != -99)
    aPool.SetNbDefaultThreadsToLaunch(aDef);
  if (anInit != 0)
    aPool.Init(anInit);
  out << " | ";
  osd_pool_sizes(out, aPool);
  out << ' ' << aPool.IsInUse();
}

static void osd_launcher_probe(std::istream& in, std::ostream& out)
{
  int aNb = 0, aMax1 = 0, aMax2 = 0;
  in >> aNb >> aMax1 >> aMax2;
  OSD_ThreadPool           aPool(aNb);
  OSD_ThreadPool::Launcher aFirst(aPool, aMax1);
  osd_launcher_sizes(out, aFirst);
  out << ' ' << aPool.IsInUse() << " | ";
  OSD_ThreadPool::Launcher aSecond(aPool, aMax2);
  osd_launcher_sizes(out, aSecond);
  const OsdThreadsCoverage aCoverage(0, 100);
  const int                aLower = aSecond.LowerThreadIndex(), anUpper = aSecond.UpperThreadIndex();
  aSecond.Perform(0, 100, [&](int theThread, int theIndex) {
    aCoverage.Thread(theThread, aLower, anUpper);
    aCoverage.Visit(theIndex);
  });
  out << ' ';
  aCoverage.Print(out);
  aFirst.Release();
  out << " | " << aFirst.NbThreads() << ' ' << aPool.IsInUse();
  const OsdThreadsCoverage anEmpty(0, 10);
  aFirst.Perform(0, 10, [&](int, int theIndex) { anEmpty.Visit(theIndex); });
  out << ' ' << anEmpty.myCount;
  aSecond.Release();
  out << ' ' << aPool.IsInUse();
}

static void osd_perform_probe(std::istream& in, std::ostream& out)
{
  int aNb = 0, aMax = 0, aBegin = 0, anEnd = 0;
  in >> aNb >> aMax >> aBegin >> anEnd;
  OSD_ThreadPool           aPool(aNb);
  OSD_ThreadPool::Launcher aLauncher(aPool, aMax);
  const OsdThreadsCoverage aCoverage(aBegin, anEnd);
  const int aLower = aLauncher.LowerThreadIndex(), anUpper = aLauncher.UpperThreadIndex();
  aLauncher.Perform(aBegin, anEnd, [&](int theThread, int theIndex) {
    aCoverage.Thread(theThread, aLower, anUpper);
    aCoverage.Visit(theIndex);
  });
  osd_launcher_sizes(out, aLauncher);
  out << ' ';
  aCoverage.Print(out);
}

static void osd_perform_once(std::ostream& out, OSD_ThreadPool::Launcher& theLauncher, int theBegin,
                             int theEnd, int theAt)
{
  try
  {
    theLauncher.Perform(theBegin, theEnd, [&](int, int theIndex) {
      if (theIndex == theAt)
        throw Standard_Failure("boom");
    });
    out << "ok";
  }
  catch (const Standard_Failure& theFailure)
  {
    out << theFailure.ExceptionType() << osd_escape(theFailure.what());
  }
}

static void osd_perform_fail_probe(std::istream& in, std::ostream& out)
{
  int aNb = 0, aMax = 0, aBegin = 0, anEnd = 0, anAt = 0;
  in >> aNb >> aMax >> aBegin >> anEnd >> anAt;
  OSD_ThreadPool           aPool(aNb);
  OSD_ThreadPool::Launcher aLauncher(aPool, aMax);
  osd_perform_once(out, aLauncher, aBegin, anEnd, anAt);
  out << ' ';
  if (aLauncher.NbThreads() != 1)
  {
    out << '-';
    return;
  }
  osd_perform_once(out, aLauncher, aBegin, anEnd, anEnd);
}

static void osd_for_probe(std::istream& in, std::ostream& out)
{
  int aBegin = 0, anEnd = 0, aForce = 0;
  in >> aBegin >> anEnd >> aForce;
  const OsdThreadsCoverage aCoverage(aBegin, anEnd);
  OSD_Parallel::For(aBegin, anEnd, [&](int theIndex) { aCoverage.Visit(theIndex); }, aForce != 0);
  aCoverage.Print(out);
}

static void osd_foreach_probe(std::istream& in, std::ostream& out)
{
  int aCount = 0, aNbItems = 0, aForce = 0;
  in >> aCount >> aNbItems >> aForce;
  std::vector<int> aValues(aCount);
  for (int anIndex = 0; anIndex < aCount; ++anIndex)
    aValues[anIndex] = anIndex;
  const OsdThreadsCoverage aCoverage(0, aCount);
  OSD_Parallel::ForEach(aValues.begin(), aValues.end(),
                        [&](int theValue) { aCoverage.Visit(theValue); }, aForce != 0,
                        aNbItems);
  aCoverage.Print(out);
}

static void osd_nproc_probe(std::ostream& out)
{
  out << OSD_Parallel::NbLogicalProcessors() << ' ' << OSD_Parallel::ToUseOcctThreads();
  OSD_Parallel::SetUseOcctThreads(false);
  out << ' ' << OSD_Parallel::ToUseOcctThreads();
  OSD_Parallel::SetUseOcctThreads(true);
}

static void* osd_thread_double(void* theData)
{
  return reinterpret_cast<void*>(reinterpret_cast<intptr_t>(theData) * 2);
}

static std::atomic<bool>              THE_OSD_SPIN_RELEASED{false};
static std::atomic<Standard_ThreadId> THE_OSD_SPIN_CURRENT{0};

static void* osd_thread_spin(void* theData)
{
  THE_OSD_SPIN_CURRENT = OSD_Thread::Current();
  while (!THE_OSD_SPIN_RELEASED)
  {
  }
  return reinterpret_cast<void*>(reinterpret_cast<intptr_t>(theData) + 1);
}

static void osd_wait_out(std::ostream& out, OSD_Thread& theThread)
{
  void*      aResult = reinterpret_cast<void*>(7);
  const bool isDone  = theThread.Wait(aResult);
  out << ' ' << isDone << ' ' << reinterpret_cast<intptr_t>(aResult);
}

static void osd_thread_probe(std::istream& in, std::ostream& out)
{
  intptr_t aValue = 0;
  in >> aValue;
  OSD_Thread anEmpty;
  out << anEmpty.Run(reinterpret_cast<void*>(aValue)) << ' ' << anEmpty.Wait();
  void* aTimed = reinterpret_cast<void*>(7);
  out << ' ' << anEmpty.Wait(5, aTimed) << ' ' << reinterpret_cast<intptr_t>(aTimed);
  OSD_Thread aThread(osd_thread_double);
  out << ' ' << (aThread.GetId() == 0) << ' ' << aThread.Run(reinterpret_cast<void*>(aValue));
  out << ' ' << (aThread.GetId() != 0);
  osd_wait_out(out, aThread);
  out << ' ' << (aThread.GetId() == 0);
  osd_wait_out(out, aThread);
  out << ' ' << (OSD_Thread::Current() != 0);
}

static void osd_thread_timed_probe(std::istream& in, std::ostream& out)
{
  int aTime = 0;
  in >> aTime;
  THE_OSD_SPIN_RELEASED = false;
  OSD_Thread aThread(osd_thread_spin);
  out << aThread.Run(reinterpret_cast<void*>(41));
  const Standard_ThreadId anId    = aThread.GetId();
  void*                   aResult = reinterpret_cast<void*>(7);
  out << ' ' << aThread.Wait(aTime, aResult) << ' ' << reinterpret_cast<intptr_t>(aResult);
  THE_OSD_SPIN_RELEASED = true;
  out << ' ' << aThread.Wait(10000, aResult) << ' ' << reinterpret_cast<intptr_t>(aResult);
  out << ' ' << (THE_OSD_SPIN_CURRENT == anId) << ' ' << (aThread.GetId() == 0);
}

static void osd_thread_copy_probe(std::istream& in, std::ostream& out)
{
  intptr_t aValue = 0;
  in >> aValue;
  OSD_Thread anOriginal(osd_thread_double);
  OSD_Thread aCopy(anOriginal);
  out << aCopy.Run(reinterpret_cast<void*>(aValue));
  osd_wait_out(out, aCopy);
  OSD_Thread anAssigned;
  anAssigned = anOriginal;
  out << ' ' << anAssigned.Run(reinterpret_cast<void*>(aValue + 1));
  osd_wait_out(out, anAssigned);
  anAssigned.SetFunction(osd_thread_double);
  anAssigned.SetPriority(0);
  out << ' ' << anAssigned.Run(reinterpret_cast<void*>(aValue + 2), 65536);
  osd_wait_out(out, anAssigned);
}

static bool osd_threads_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "osd_pool")
    osd_pool_probe(in, out);
  else if (op == "osd_launcher")
    osd_launcher_probe(in, out);
  else if (op == "osd_perform")
    osd_perform_probe(in, out);
  else if (op == "osd_perform_fail")
    osd_perform_fail_probe(in, out);
  else if (op == "osd_for")
    osd_for_probe(in, out);
  else if (op == "osd_foreach")
    osd_foreach_probe(in, out);
  else if (op == "osd_nproc")
    osd_nproc_probe(out);
  else if (op == "osd_default_pool")
    osd_pool_sizes(out, *OSD_ThreadPool::DefaultPool());
  else if (op == "osd_thread")
    osd_thread_probe(in, out);
  else if (op == "osd_thread_timed")
    osd_thread_timed_probe(in, out);
  else if (op == "osd_thread_copy")
    osd_thread_copy_probe(in, out);
  else
    return false;
  return true;
}
