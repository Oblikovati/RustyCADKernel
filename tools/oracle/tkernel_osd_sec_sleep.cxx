#include <OSD.hxx>

#include <cerrno>
#include <chrono>
#include <csignal>
#include <iostream>

volatile sig_atomic_t wasAlarmed = 0;

void markAlarm(int)
{
  wasAlarmed = 1;
}

void observeSleep(const char* theName, int theSeconds)
{
  wasAlarmed = 0;
  errno = 0;
  std::signal(SIGALRM, markAlarm);
  ualarm(100000, 0);
  const auto aStart = std::chrono::steady_clock::now();
  OSD::SecSleep(theSeconds);
  const auto aElapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
    std::chrono::steady_clock::now() - aStart);
  ualarm(0, 0);
  std::cout << theName << "_errno=" << errno << " alarm=" << wasAlarmed
            << " returned=" << (aElapsed.count() < 2000) << '\n';
}

void observeMilliSleep(const char* theName, int theMilliseconds)
{
  wasAlarmed = 0;
  errno = 0;
  std::signal(SIGALRM, markAlarm);
  ualarm(100000, 0);
  const auto aStart = std::chrono::steady_clock::now();
  OSD::MilliSecSleep(theMilliseconds);
  const auto aElapsed = std::chrono::duration_cast<std::chrono::milliseconds>(
    std::chrono::steady_clock::now() - aStart);
  ualarm(0, 0);
  std::cout << theName << "_errno=" << errno << " alarm=" << wasAlarmed
            << " returned=" << (aElapsed.count() < 2000) << '\n';
}

int main()
{
  observeSleep("positive", 30);
  observeMilliSleep("milliseconds", 30000);
  observeSleep("negative", -1);
}
