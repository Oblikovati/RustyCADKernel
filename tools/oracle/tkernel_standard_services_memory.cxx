// Standard.hxx global memory services and Standard_MMgrOpt, one request per line.
// Included by tkernel_standard_services_probe.cxx before its case table.
#include <Standard.hxx>
#include <Standard_MMgrOpt.hxx>
#include <algorithm>
#include <cstdint>
#include <vector>

namespace standard_services_probe
{
// "services_standard_allocate <size>": Standard::Allocate zero-fills (calloc).
static std::string standard_allocate(std::istream& theInput)
{
  std::size_t aSize = 0;
  theInput >> aSize;
  return tkernel_probe::guarded([&] {
    char*       aBlock = static_cast<char*>(Standard::Allocate(aSize));
    const bool  isZero = std::all_of(aBlock, aBlock + std::min<std::size_t>(aSize, 4096), [](char c) {
      return c == 0;
    });
    Standard::Free(aBlock);
    return std::string("zero=") + (isZero ? "1" : "0");
  });
}

// "services_standard_allocate_optimal <size>": malloc, null instead of an exception.
static std::string standard_allocate_optimal(std::istream& theInput)
{
  std::size_t aSize = 0;
  theInput >> aSize;
  void* aBlock = Standard::AllocateOptimal(aSize);
  Standard::Free(aBlock);
  return aBlock == nullptr ? "null" : "live";
}

// "services_standard_reallocate <initial size|-> <new size>": prefix kept by realloc.
static std::string standard_reallocate(std::istream& theInput)
{
  std::string aFirst;
  std::size_t aNewSize = 0;
  theInput >> aFirst >> aNewSize;
  const std::size_t anOld   = aFirst == "-" ? 0 : std::stoull(aFirst);
  char*             aBlock  = aFirst == "-" ? nullptr : static_cast<char*>(Standard::Allocate(anOld));
  for (std::size_t i = 0; i < anOld; ++i)
    aBlock[i] = static_cast<char>(i + 1);
  std::string aResult = tkernel_probe::guarded([&] {
    char* aMoved = static_cast<char*>(Standard::Reallocate(aBlock, aNewSize));
    aBlock       = aMoved;
    return tkernel_probe::hex(aMoved, std::min<std::size_t>({anOld, aNewSize, 32}));
  });
  // glibc's realloc(ptr, 0) has already released the block before the exception.
  if (aNewSize != 0 || aResult.rfind("OK", 0) == 0)
    Standard::Free(aBlock);
  return aResult;
}

// "services_standard_aligned <size> <alignment>": null or the block's alignment check.
static std::string standard_aligned(std::istream& theInput)
{
  std::size_t aSize = 0, anAlign = 0;
  theInput >> aSize >> anAlign;
  void* aBlock = Standard::AllocateAligned(aSize, anAlign);
  if (aBlock == nullptr)
    return "null";
  const bool isAligned = reinterpret_cast<std::uintptr_t>(aBlock) % anAlign == 0;
  Standard::FreeAligned(aBlock);
  return std::string("aligned=") + (isAligned ? "1" : "0");
}

// "services_standard_misc": allocator type, Purge and the nulling template Free overloads.
static std::string standard_misc(std::istream&)
{
  char* aBlock   = static_cast<char*>(Standard::Allocate(8));
  char* anAligned = static_cast<char*>(Standard::AllocateAligned(8, 16));
  Standard::Free(aBlock);
  Standard::FreeAligned(anAligned);
  return "type=" + std::to_string(static_cast<int>(Standard::GetAllocatorType()))
         + " purge=" + std::to_string(Standard::Purge()) + " nulled="
         + std::to_string(aBlock == nullptr && anAligned == nullptr);
}

namespace mmgr
{
struct Event
{
  bool        IsAlloc;
  std::size_t RoundSize;
  std::size_t Size;
};

static std::vector<Event>& events()
{
  static std::vector<Event> THE_EVENTS;
  return THE_EVENTS;
}

static void record(const bool theIsAlloc, void* const, const std::size_t theRound, const std::size_t theSize)
{
  events().push_back({theIsAlloc, theRound, theSize});
}

enum class State
{
  Live,
  Cached,
  Gone
};

struct Block
{
  char*       Address;
  std::size_t Round;
  std::size_t Filled;
  State       Status;
  std::size_t Owner; // index whose pattern the bytes hold
};

// Replays one script against a fresh manager; see tools/tkernel_standard_services_corpus.py.
class Script
{
public:
  Script(bool theClear, bool theMap, std::size_t theCell, std::size_t theThreshold)
      : myClear(theClear),
        myMap(theMap),
        myCell((theCell + 15) & ~std::size_t(15)),
        myMax((((theThreshold - 1) + 7) & ~std::size_t(7)) >> 3)
  {
  }

  std::string Run(Standard_MMgrOpt& theManager, const std::string& theToken)
  {
    events().clear();
    std::string aResult;
    try
    {
      aResult = step(theManager, theToken);
    }
    catch (const Standard_Failure& anError)
    {
      // A failed allocation may have purged cached blocks; forget them.
      forgetCached();
      aResult = std::string("E") + anError.ExceptionType() + ":" + anError.what();
    }
    return eventText() + "/" + aResult;
  }

private:
  std::string step(Standard_MMgrOpt& theManager, const std::string& theToken)
  {
    const char        anOp = theToken[0];
    const std::string anArgs = theToken.substr(1);
    if (anOp == 'a')
      return added(theManager.Allocate(std::stoull(anArgs)), std::stoull(anArgs));
    if (anOp == 'R')
      return added(theManager.Reallocate(nullptr, std::stoull(anArgs)), std::stoull(anArgs));
    if (anOp == 'm')
      return many(theManager, anArgs);
    if (anOp == 'f')
      return freed(theManager, std::stoul(anArgs));
    if (anOp == 'F')
      return freedRange(theManager, anArgs);
    if (anOp == 'r')
      return reallocated(theManager, anArgs);
    if (anOp == 'n')
    {
      theManager.Free(nullptr);
      return "-";
    }
    const int aCount = theManager.Purge(false);
    forgetCached();
    return "P" + std::to_string(aCount);
  }

  std::string many(Standard_MMgrOpt& theManager, const std::string& theArgs)
  {
    const std::size_t aComma = theArgs.find(',');
    const std::size_t aCount = std::stoull(theArgs.substr(0, aComma));
    const std::size_t aSize  = std::stoull(theArgs.substr(aComma + 1));
    int               aPools = 0;
    std::string       aLast;
    for (std::size_t i = 0; i < aCount; ++i)
    {
      aLast = added(theManager.Allocate(aSize), aSize);
      aPools += aLast.rfind("@new", 0) == 0 ? 1 : 0;
    }
    events().erase(events().begin(), events().end() - std::min<std::size_t>(1, events().size()));
    return "pools=" + std::to_string(aPools) + ";" + aLast;
  }

  std::string freed(Standard_MMgrOpt& theManager, std::size_t theIndex)
  {
    theManager.Free(myBlocks[theIndex].Address);
    release(theIndex);
    return "-";
  }

  std::string freedRange(Standard_MMgrOpt& theManager, const std::string& theArgs)
  {
    const std::size_t aComma = theArgs.find(',');
    const std::size_t aFirst = std::stoull(theArgs.substr(0, aComma));
    const std::size_t aCount = std::stoull(theArgs.substr(aComma + 1));
    for (std::size_t i = aFirst; i < aFirst + aCount; ++i)
      freed(theManager, i);
    events().erase(events().begin(), events().end() - std::min<std::size_t>(1, events().size()));
    return "-";
  }

  std::string reallocated(Standard_MMgrOpt& theManager, const std::string& theArgs)
  {
    const std::size_t aComma = theArgs.find(',');
    const std::size_t anIndex = std::stoull(theArgs.substr(0, aComma));
    const std::size_t aSize   = std::stoull(theArgs.substr(aComma + 1));
    Block             anOld   = myBlocks[anIndex];
    char*             aMoved  = static_cast<char*>(theManager.Reallocate(anOld.Address, aSize));
    const std::size_t aKept   = std::min(anOld.Filled, aSize);
    bool              isCopy  = true;
    for (std::size_t i = 0; i < aKept; ++i)
      isCopy = isCopy && aMoved[i] == pattern(anOld.Owner, i);
    std::string aRelation;
    if (aMoved == anOld.Address)
    {
      myBlocks[anIndex].Status = State::Gone;
      aRelation = "same" + std::string(isCopy ? "c1" : "c0");
      myBlocks.push_back({aMoved, anOld.Round, anOld.Filled, State::Live, anOld.Owner});
      return aRelation;
    }
    release(anIndex);
    const std::size_t aRound = events().empty() ? 0 : events().front().RoundSize;
    return relation(aMoved, aRound, aSize) + (isCopy ? "c1" : "c0");
  }

  std::string added(void* theAddress, std::size_t theSize)
  {
    const std::size_t aRound = events().empty() ? 0 : events().back().RoundSize;
    return relation(static_cast<char*>(theAddress), aRound, theSize);
  }

  // How a new block relates to earlier ones: reuse of a cached block, offset in the current
  // small-block pool, or a fresh system block; then its address modulo 16 and zero check.
  std::string relation(char* theAddress, std::size_t theRound, std::size_t theSize)
  {
    std::string aRelation = reuse(theAddress);
    if (aRelation.empty() && theRound <= myCell && (theRound >> 3) <= myMax)
      aRelation = poolOffset(theAddress, theRound);
    if (aRelation.empty())
      aRelation = "new";
    aRelation += "%" + std::to_string(reinterpret_cast<std::uintptr_t>(theAddress) % 16);
    const std::size_t aFill = std::min(theSize, theRound);
    if (myClear)
    {
      const bool isZero = std::all_of(theAddress, theAddress + aFill, [](char c) { return c == 0; });
      aRelation += isZero ? "z1" : "z0";
    }
    const std::size_t anIndex = myBlocks.size();
    for (std::size_t i = 0; i < aFill; ++i)
      theAddress[i] = pattern(anIndex, i);
    myBlocks.push_back({theAddress, theRound, aFill, State::Live, anIndex});
    return aRelation;
  }

  std::string reuse(char* theAddress)
  {
    for (std::size_t i = myBlocks.size(); i-- > 0;)
    {
      if (myBlocks[i].Status == State::Cached && myBlocks[i].Address == theAddress)
      {
        myBlocks[i].Status = State::Gone;
        return "=" + std::to_string(i);
      }
    }
    return "";
  }

  std::string poolOffset(char* theAddress, std::size_t theRound)
  {
    std::string aRelation;
    if (myPoolBase == nullptr || theAddress != myNextSmall)
    {
      myPoolBase = theAddress;
      // A mapped pool starts on a page; a malloc'ed pool's page offset is not reproducible.
      const std::uintptr_t anOffset = reinterpret_cast<std::uintptr_t>(theAddress) % 4096;
      aRelation = "@new" + (myMap ? std::to_string(anOffset) : std::string());
    }
    else
    {
      aRelation = "@" + std::to_string(theAddress - myPoolBase);
    }
    myNextSmall = theAddress + theRound + sizeof(std::size_t);
    return aRelation;
  }

  void release(std::size_t theIndex)
  {
    const bool isLarge          = (myBlocks[theIndex].Round >> 3) > myMax;
    myBlocks[theIndex].Status = isLarge ? State::Gone : State::Cached;
  }

  void forgetCached()
  {
    for (Block& aBlock : myBlocks)
      aBlock.Status = aBlock.Status == State::Cached ? State::Gone : aBlock.Status;
  }

  static char pattern(std::size_t theIndex, std::size_t theByte)
  {
    return static_cast<char>((theIndex * 31 + theByte + 1) & 0xff);
  }

  static std::string eventText()
  {
    std::string aText;
    for (const Event& anEvent : events())
    {
      aText += aText.empty() ? "" : ",";
      aText += anEvent.IsAlloc ? "A" + std::to_string(anEvent.RoundSize) + ":" + std::to_string(anEvent.Size)
                               : "F" + std::to_string(anEvent.RoundSize);
    }
    return aText.empty() ? "-" : aText;
  }

  bool               myClear;
  bool               myMap;
  std::size_t        myCell;
  std::size_t        myMax;
  std::vector<Block> myBlocks;
  char*              myPoolBase  = nullptr;
  char*              myNextSmall = nullptr;
};
} // namespace mmgr

// "services_mmgr <clear> <mmap> <cell> <pages> <threshold> <token>...": one result per token.
static std::string mmgr_script(std::istream& theInput)
{
  int         aClear = 0, aMap = 0, aPages = 0;
  std::size_t aCell = 0, aThreshold = 0;
  theInput >> aClear >> aMap >> aCell >> aPages >> aThreshold;
  Standard_MMgrOpt::SetCallBackFunction(mmgr::record);
  std::string aResult;
  {
    Standard_MMgrOpt aManager(aClear != 0, aMap != 0, aCell, aPages, aThreshold);
    mmgr::Script     aScript(aClear != 0, aMap != 0, aCell, aThreshold);
    std::string      aToken;
    while (theInput >> aToken)
      aResult += (aResult.empty() ? "" : " ") + aScript.Run(aManager, aToken);
  }
  Standard_MMgrOpt::SetCallBackFunction(nullptr);
  return "OK " + aResult;
}
} // namespace standard_services_probe
