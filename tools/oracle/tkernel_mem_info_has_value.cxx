// Supplemental protected hook probe at pinned OCCT 3d097a0.
#include <OSD_MemInfo.hxx>
#include <iostream>

struct MemoryAvailabilityProbe : OSD_MemInfo
{
  MemoryAvailabilityProbe() : OSD_MemInfo(false) {}
  using OSD_MemInfo::hasValue;
};

void printAvailability(const MemoryAvailabilityProbe& counters)
{
  for (int index = 0; index < OSD_MemInfo::MemCounter_NB; ++index)
    std::cout << counters.hasValue(static_cast<OSD_MemInfo::Counter>(index));
  std::cout << '\n';
}

int main()
{
  MemoryAvailabilityProbe counters;
  printAvailability(counters);
  counters.Update();
  printAvailability(counters);
  counters.SetActive(false);
  printAvailability(counters);
  counters.SetActive(true);
  printAvailability(counters);
  counters.Clear();
  printAvailability(counters);
}
