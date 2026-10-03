#include <Standard_ErrorHandler.hxx>
#include <Standard_Mutex.hxx>

#include <cstdio>
#include <thread>

static bool otherThreadCanLock(Standard_Mutex& theMutex) {
  bool isLocked = false;
  std::thread aThread([&] {
    isLocked = theMutex.TryLock();
    if (isLocked) {
      theMutex.Unlock();
    }
  });
  aThread.join();
  return isLocked;
}

int main() {
  Standard_Mutex aMutex;
  Standard_ErrorHandler anErrorHandler;
  Standard_Mutex::Sentry aSentry(aMutex);
  const bool lockedDuringScope = !otherThreadCanLock(aMutex);
  anErrorHandler.Destroy();
  const bool releasedByHandler = otherThreadCanLock(aMutex);
  std::printf("%d\n%d\n", lockedDuringScope ? 1 : 0, releasedByHandler ? 1 : 0);
  return lockedDuringScope && releasedByHandler ? 0 : 1;
}
