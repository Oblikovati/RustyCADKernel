// Standard_Mutex (with Sentry) and Standard_Condition, driven by token scripts.
// Included by tkernel_standard_services_probe.cxx before its case table.
#include <Standard_Condition.hxx>
#include <Standard_Mutex.hxx>
#include <chrono>
#include <memory>
#include <thread>
#include <vector>

namespace standard_services_probe
{
Standard_DISABLE_DEPRECATION_WARNINGS

// "services_mutex <token>...": L/U/T lock, unlock, try-lock on the main thread; t try-lock
// (and release on success) and u unlock on a second thread; S/N push a sentry on the mutex
// or on NULL; s pops the newest sentry. T and t answer 0/1, the others "-".
static std::string mutex_script(std::istream& theInput)
{
  Standard_Mutex                                       aMutex;
  std::vector<std::unique_ptr<Standard_Mutex::Sentry>> aSentries;
  std::string                                          aResult, aToken;
  while (theInput >> aToken)
  {
    std::string anAnswer = "-";
    if (aToken == "L")
      aMutex.Lock();
    else if (aToken == "U")
      aMutex.Unlock();
    else if (aToken == "T")
      anAnswer = aMutex.TryLock() ? "1" : "0";
    else if (aToken == "t")
      std::thread([&] {
        const bool isLocked = aMutex.TryLock();
        if (isLocked)
          aMutex.Unlock();
        anAnswer = isLocked ? "1" : "0";
      }).join();
    else if (aToken == "u")
      std::thread([&] { aMutex.Unlock(); }).join();
    else if (aToken == "S")
      aSentries.push_back(std::make_unique<Standard_Mutex::Sentry>(aMutex));
    else if (aToken == "N")
      aSentries.push_back(std::make_unique<Standard_Mutex::Sentry>(static_cast<Standard_Mutex*>(nullptr)));
    else
      aSentries.pop_back();
    aResult += (aResult.empty() ? "" : " ") + anAnswer;
  }
  // Scripts are balanced (see the corpus), so the mutex is destroyed unlocked.
  aSentries.clear();
  return aResult;
}

Standard_ENABLE_DEPRECATION_WARNINGS

// "services_condition <initial 0|1> <token>...": S set, R reset, C check, X check-and-reset,
// W<ms> timed wait, D<delay>,<ms> timed wait while a thread sets after <delay> ms,
// I<delay> infinite wait while a thread sets after <delay> ms. Answers 0/1 or "-".
static std::string condition_script(std::istream& theInput)
{
  int aState = 0;
  theInput >> aState;
  Standard_Condition aCondition(aState != 0);
  std::string        aResult, aToken;
  while (theInput >> aToken)
  {
    std::string anAnswer = "-";
    const char  anOp     = aToken[0];
    if (anOp == 'S')
      aCondition.Set();
    else if (anOp == 'R')
      aCondition.Reset();
    else if (anOp == 'C')
      anAnswer = aCondition.Check() ? "1" : "0";
    else if (anOp == 'X')
      anAnswer = aCondition.CheckReset() ? "1" : "0";
    else if (anOp == 'W')
      anAnswer = aCondition.Wait(std::stoi(aToken.substr(1))) ? "1" : "0";
    else
    {
      const std::size_t aComma = aToken.find(',');
      const int         aDelay = std::stoi(aToken.substr(1, aComma));
      std::thread       aSetter([&] {
        std::this_thread::sleep_for(std::chrono::milliseconds(aDelay));
        aCondition.Set();
      });
      if (anOp == 'D')
        anAnswer = aCondition.Wait(std::stoi(aToken.substr(aComma + 1))) ? "1" : "0";
      else
        aCondition.Wait();
      aSetter.join();
    }
    aResult += (aResult.empty() ? "" : " ") + anAnswer;
  }
  return aResult;
}
} // namespace standard_services_probe
