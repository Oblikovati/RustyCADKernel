#include <Standard_Type.hxx>
#include <Standard_Transient.hxx>

#include <algorithm>
#include <iostream>
#include <atomic>
#include <thread>
#include <vector>

struct OwnedRegistrationProbe
{
};
struct ConcurrentOwnedRegistrationProbe
{
};

void concurrent_registration_case(const occ::handle<Standard_Type>& theParent);
std::vector<std::thread> launch_registration_workers(
  const occ::handle<Standard_Type>& theParent,
  std::atomic<int>& theReady,
  std::atomic<bool>& theStart,
  std::vector<occ::handle<Standard_Type>>& theHandles);
void wait_for_registration_workers(std::atomic<int>& theReady, int theCount);
void join_registration_workers(std::vector<std::thread>& theWorkers);
bool all_handles_share_descriptor(
  const std::vector<occ::handle<Standard_Type>>& theHandles,
  const occ::handle<Standard_Type>& theAnchor);
void report_replacement(const occ::handle<Standard_Type>& theParent);
void single_registration_case(const occ::handle<Standard_Type>& theParent);

int main()
{
  occ::handle<Standard_Type> aParent = STANDARD_TYPE(Standard_Transient);
  single_registration_case(aParent);
  concurrent_registration_case(aParent);
}

void single_registration_case(const occ::handle<Standard_Type>& theParent)
{
  Standard_Type* aFirst =
    Standard_Type::Register(typeid(OwnedRegistrationProbe), "First", 17, theParent);
  std::cout << "raw " << aFirst->GetRefCount() << ' ' << aFirst->Name() << ' '
            << theParent->GetRefCount() << '\n';
  {
    occ::handle<Standard_Type> anOwner(aFirst);
    occ::handle<Standard_Type> aRepeated(
      Standard_Type::Register(typeid(OwnedRegistrationProbe), "Ignored", 99, theParent));
    std::cout << "live " << anOwner->GetRefCount() << ' ' << (anOwner == aRepeated) << ' '
              << anOwner->Name() << ' ' << anOwner->Size() << ' ' << theParent->GetRefCount() << '\n';
  }
  report_replacement(theParent);
}

void report_replacement(const occ::handle<Standard_Type>& theParent)
{
  Standard_Type* aReplacement = Standard_Type::Register(typeid(OwnedRegistrationProbe), "Replacement", 29, theParent);
  std::cout << "replacement " << aReplacement->GetRefCount() << ' ' << aReplacement->Name() << ' '
            << aReplacement->Size() << ' ' << theParent->GetRefCount() << '\n';
  {
    occ::handle<Standard_Type> anOwner(aReplacement);
    std::cout << "replacement_owned " << anOwner->GetRefCount() << '\n';
  }
  std::cout << "parent_released " << theParent->GetRefCount() << '\n';
}

void concurrent_registration_case(const occ::handle<Standard_Type>& theParent)
{
  Standard_Type* aRaw = Standard_Type::Register(
    typeid(ConcurrentOwnedRegistrationProbe), "ConcurrentOwned", 31, theParent);
  occ::handle<Standard_Type> anAnchor(aRaw);
  constexpr int aWorkerCount = 12;
  std::atomic<int> aReady{0};
  std::atomic<bool> aStart{false};
  std::vector<occ::handle<Standard_Type>> aHandles(aWorkerCount);
  auto aWorkers = launch_registration_workers(theParent, aReady, aStart, aHandles);
  wait_for_registration_workers(aReady, aWorkerCount);
  aStart.store(true);
  join_registration_workers(aWorkers);
  const bool isSame = all_handles_share_descriptor(aHandles, anAnchor);
  std::cout << "concurrent_owned " << anAnchor->GetRefCount() << ' ' << isSame << ' '
            << anAnchor->Name() << ' ' << anAnchor->Size() << ' ' << theParent->GetRefCount() << '\n';
  aHandles.clear();
  anAnchor.Nullify();
  std::cout << "concurrent_owned_released " << theParent->GetRefCount() << '\n';
}

std::vector<std::thread> launch_registration_workers(
  const occ::handle<Standard_Type>& theParent,
  std::atomic<int>& theReady,
  std::atomic<bool>& theStart,
  std::vector<occ::handle<Standard_Type>>& theHandles)
{
  std::vector<std::thread> aWorkers;
  for (int anIndex = 0; anIndex < static_cast<int>(theHandles.size()); ++anIndex)
    aWorkers.emplace_back([&, anIndex] {
      theReady.fetch_add(1);
      while (!theStart.load())
        std::this_thread::yield();
      theHandles[anIndex] = Standard_Type::Register(
        typeid(ConcurrentOwnedRegistrationProbe), "IgnoredConcurrent", 99, theParent);
    });
  return aWorkers;
}

void wait_for_registration_workers(std::atomic<int>& theReady, int theCount)
{
  while (theReady.load() != theCount)
    std::this_thread::yield();
}

void join_registration_workers(std::vector<std::thread>& theWorkers)
{
  for (std::thread& aWorker : theWorkers)
    aWorker.join();
}

bool all_handles_share_descriptor(
  const std::vector<occ::handle<Standard_Type>>& theHandles,
  const occ::handle<Standard_Type>& theAnchor)
{
  return std::all_of(theHandles.begin(), theHandles.end(), [&](const auto& theHandle) {
    return theHandle == theAnchor;
  });
}
