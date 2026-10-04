#include <Standard_Transient.hxx>
#include <Standard_Type.hxx>

#include <atomic>
#include <iostream>
#include <thread>
#include <vector>

class OwnerProbeParent : public Standard_Transient
{
  DEFINE_STANDARD_RTTIEXT(OwnerProbeParent, Standard_Transient)
};
class OwnerProbeChild : public OwnerProbeParent
{
  DEFINE_STANDARD_RTTIEXT(OwnerProbeChild, OwnerProbeParent)
};
class ConcurrentProbeParent : public Standard_Transient
{
  DEFINE_STANDARD_RTTIEXT(ConcurrentProbeParent, Standard_Transient)
};
class ConcurrentProbeChild : public ConcurrentProbeParent
{
  DEFINE_STANDARD_RTTIEXT(ConcurrentProbeChild, ConcurrentProbeParent)
};

IMPLEMENT_STANDARD_RTTIEXT(OwnerProbeParent, Standard_Transient)
IMPLEMENT_STANDARD_RTTIEXT(OwnerProbeChild, OwnerProbeParent)
IMPLEMENT_STANDARD_RTTIEXT(ConcurrentProbeParent, Standard_Transient)
IMPLEMENT_STANDARD_RTTIEXT(ConcurrentProbeChild, ConcurrentProbeParent)

void concurrent_case();
void raw_register_case();

int main()
{
  const auto& aParent = STANDARD_TYPE(OwnerProbeParent);
  std::cout << "static_parent " << aParent->GetRefCount() << '\n';
  const auto& aChild = STANDARD_TYPE(OwnerProbeChild);
  std::cout << "static_child " << aChild->GetRefCount() << ' ' << aParent->GetRefCount() << '\n';
  {
    occ::handle<Standard_Type> aCopy = aChild;
    std::cout << "copied_child " << aCopy->GetRefCount() << '\n';
  }
  std::cout << "released_child " << aChild->GetRefCount() << '\n';
  concurrent_case();
  raw_register_case();
}

void concurrent_case()
{
  const auto& aParent = STANDARD_TYPE(ConcurrentProbeParent);
  constexpr int aWorkerCount = 12;
  std::atomic<int> aReady{0};
  std::atomic<bool> aStart{false};
  std::vector<std::thread> aWorkers;
  for (int anIndex = 0; anIndex < aWorkerCount; ++anIndex)
  {
    aWorkers.emplace_back([&] {
      aReady.fetch_add(1);
      while (!aStart.load())
        std::this_thread::yield();
      (void)STANDARD_TYPE(ConcurrentProbeChild);
    });
  }
  while (aReady.load() != aWorkerCount)
    std::this_thread::yield();
  aStart.store(true);
  for (std::thread& aWorker : aWorkers)
    aWorker.join();
  const auto& aChild = STANDARD_TYPE(ConcurrentProbeChild);
  std::cout << "concurrent " << aChild->GetRefCount() << ' ' << aParent->GetRefCount() << '\n';
}

void raw_register_case()
{
  const auto& aParent = STANDARD_TYPE(OwnerProbeParent);
  struct RawDescriptor
  {
  };
  Standard_Type* aRaw = Standard_Type::Register(typeid(RawDescriptor), "RawDescriptor", 0, aParent);
  std::cout << "raw_registration " << aRaw->GetRefCount() << ' ' << aParent->GetRefCount() << '\n';
}
