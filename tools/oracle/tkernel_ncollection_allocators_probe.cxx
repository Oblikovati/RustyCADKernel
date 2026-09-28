// NCollection allocators probe operations (twin of oracle_probe/ncollection/allocators.rs).
// Block identities are printed as positions and offsets, never raw addresses, so the native
// and Rust heaps may place blocks anywhere.
#include <NCollection_AccAllocator.hxx>
#include <NCollection_AlignedAllocator.hxx>
#include <NCollection_Allocator.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <NCollection_Buffer.hxx>
#include <NCollection_HeapAllocator.hxx>
#include <NCollection_IncAllocator.hxx>
#include <NCollection_OccAllocator.hxx>
#include <Standard_ProgramError.hxx>

#include <cstdint>
#include <istream>
#include <ostream>
#include <regex>
#include <sstream>
#include <string>
#include <vector>

namespace
{
//! Labels incremental blocks by the order they appear; an allocation continues the block whose
//! bump cursor it lands on, otherwise it starts a new block (payloads never touch).
struct IncBlockLabels
{
  std::vector<uintptr_t> Starts, Ends;

  std::string Label(uintptr_t theAddress, size_t theSize)
  {
    for (size_t i = 0; i < Ends.size(); ++i)
    {
      if (Ends[i] == theAddress)
      {
        Ends[i] = theAddress + theSize;
        return "b" + std::to_string(i) + "+" + std::to_string(theAddress - Starts[i]);
      }
    }
    Starts.push_back(theAddress);
    Ends.push_back(theAddress + theSize);
    return "b" + std::to_string(Starts.size() - 1) + "+0";
  }

  void Rewind() { Ends = Starts; }
};

void probeIncremental(std::istream& in, std::ostream& out)
{
  size_t aBlockSize = 0, aCount = 0;
  in >> aBlockSize >> aCount;
  occ::handle<NCollection_IncAllocator> anAlloc = new NCollection_IncAllocator(aBlockSize);
  IncBlockLabels                        aLabels;
  for (size_t i = 0; i < aCount; ++i)
  {
    std::string aStep;
    in >> aStep;
    if (aStep == "r0" || aStep == "r1")
    {
      anAlloc->Reset(aStep == "r1");
      if (aStep == "r1")
        aLabels = IncBlockLabels(); // released blocks: addresses may be reused
      else
        aLabels.Rewind();
      out << aStep << ' ';
      continue;
    }
    const size_t aSize = std::stoul(aStep.substr(1));
    out << aLabels.Label(reinterpret_cast<uintptr_t>(anAlloc->Allocate(aSize)), aSize) << ' ';
  }
}

//! Exposes the protected block bookkeeping of the accumulating allocator.
class AccProbe : public NCollection_AccAllocator
{
public:
  AccProbe(size_t theSize)
      : NCollection_AccAllocator(theSize)
  {
  }

  //! Chain position of the block holding theAddress and the offset from its start.
  std::string Where(void* theAddress)
  {
    Key    aKey;
    Block* aBlock = findBlock(theAddress, aKey);
    int    aPos   = 0;
    for (Block* anIter = mypLastBlock; anIter; anIter = anIter->prevBlock, ++aPos)
    {
      if (anIter == aBlock)
      {
        return std::to_string(aPos) + "+"
               + std::to_string((uint8_t*)theAddress - (uint8_t*)aBlock->address);
      }
    }
    return "none";
  }

  std::string Chain()
  {
    std::string aText = "[";
    for (Block* anIter = mypLastBlock; anIter; anIter = anIter->prevBlock)
    {
      aText += std::to_string(anIter->FreeSize()) + ":" + std::to_string(anIter->allocCount) + ";";
    }
    return aText + "]";
  }
};

void probeAccumulating(std::istream& in, std::ostream& out)
{
  size_t aBlockSize = 0, aCount = 0;
  in >> aBlockSize >> aCount;
  AccProbe           anAlloc(aBlockSize);
  std::vector<void*> anAddresses;
  out << anAlloc.Chain();
  for (size_t i = 0; i < aCount; ++i)
  {
    std::string aStep;
    in >> aStep;
    out << ' ';
    if (aStep[0] == 'a')
    {
      anAddresses.push_back(anAlloc.Allocate(std::stoul(aStep.substr(1))));
      out << anAlloc.Where(anAddresses.back());
    }
    else
    {
      int   aLocal    = 0;
      void* anAddress = aStep == "x" ? (void*)&aLocal : anAddresses[std::stoul(aStep.substr(1))];
      try
      {
        anAlloc.Free(anAddress);
        out << "freed";
      }
      catch (const Standard_ProgramError&)
      {
        out << "throw Standard_ProgramError";
      }
    }
    out << anAlloc.Chain();
  }
}

void probeAligned(std::istream& in, std::ostream& out)
{
  size_t anAlignment = 0, aSize = 0;
  in >> anAlignment >> aSize;
  occ::handle<NCollection_AlignedAllocator> anAlloc = new NCollection_AlignedAllocator(anAlignment);
  void*                                     aPtr    = anAlloc->AllocateOptimal(aSize);
  if (aPtr == nullptr)
  {
    out << "null";
    return;
  }
  out << "aligned " << (anAlignment == 0 ? 1 : reinterpret_cast<uintptr_t>(aPtr) % anAlignment == 0);
  anAlloc->Free(aPtr);
}

occ::handle<NCollection_BaseAllocator> allocatorNamed(const std::string& theName)
{
  if (theName == "common")
    return NCollection_BaseAllocator::CommonBaseAllocator();
  if (theName == "heap")
    return occ::handle<NCollection_BaseAllocator>(NCollection_HeapAllocator::GlobalHeapAllocator());
  if (theName == "inc")
    return new NCollection_IncAllocator();
  if (theName == "acc")
    return new NCollection_AccAllocator();
  if (theName[0] == 'a')
    return new NCollection_AlignedAllocator(std::stoul(theName.substr(1)));
  return nullptr;
}

//! Buffer state, with dumped addresses replaced by the token PTR.
std::string bufferState(const NCollection_Buffer& theBuffer)
{
  std::ostringstream aDump;
  theBuffer.DumpJson(aDump);
  const std::string aText = std::regex_replace(aDump.str(), std::regex("0xx[0-9a-f]+"), "PTR");
  return std::to_string(theBuffer.IsEmpty()) + " " + std::to_string(theBuffer.Size()) + " {"
         + aText + "}";
}

void probeBuffer(std::istream& in, std::ostream& out)
{
  std::string anAllocName;
  size_t      aSize = 0, aResize = 0;
  in >> anAllocName >> aSize >> aResize;
  // Every step starts with Free(), which NCollection_AccAllocator refuses for a null address.
  try
  {
    NCollection_Buffer aBuffer(allocatorNamed(anAllocName), aSize);
    out << bufferState(aBuffer) << " | ";
    const bool isAllocated = aBuffer.Allocate(aResize);
    out << isAllocated << ' ' << bufferState(aBuffer) << " | ";
    aBuffer.Free();
    out << bufferState(aBuffer);
  }
  catch (const Standard_ProgramError&)
  {
    out << "throw Standard_ProgramError";
  }
}

void probeOccEquality(std::istream& in, std::ostream& out)
{
  (void)in;
  const occ::handle<NCollection_BaseAllocator> anInc = new NCollection_IncAllocator();
  const NCollection_OccAllocator<int>          anAllocs[] = {
    NCollection_OccAllocator<int>(),
    NCollection_OccAllocator<int>(NCollection_BaseAllocator::CommonBaseAllocator()),
    NCollection_OccAllocator<int>(
      occ::handle<NCollection_BaseAllocator>(NCollection_HeapAllocator::GlobalHeapAllocator())),
    NCollection_OccAllocator<int>(anInc)};
  for (const auto& aLeft : anAllocs)
  {
    for (const auto& aRight : anAllocs)
    {
      const NCollection_OccAllocator<double> aRebound(aRight);
      // Mixed element types are ambiguous between the member and free templates in C++17,
      // so the member template is named explicitly.
      out << (aLeft == aRight) << (aLeft != aRight) << aLeft.operator==(aRebound);
    }
    out << ' ';
  }
  out << NCollection_OccAllocator<char>().max_size() << ' '
      << NCollection_OccAllocator<int16_t>().max_size() << ' '
      << NCollection_OccAllocator<int>().max_size() << ' '
      << NCollection_OccAllocator<double>().max_size() << ' '
      << NCollection_OccAllocator<char[24]>().max_size() << ' '
      << NCollection_Allocator<int>().operator==(NCollection_Allocator<double>()) << ' '
      << (NCollection_HeapAllocator::GlobalHeapAllocator()
          == NCollection_HeapAllocator::GlobalHeapAllocator())
      << (NCollection_HeapAllocator::GlobalHeapAllocator().get()
          == NCollection_BaseAllocator::CommonBaseAllocator().get());
}
} // namespace

static bool ncollection_allocators_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "ncoll_allocators_inc")
    probeIncremental(in, out);
  else if (op == "ncoll_allocators_acc")
    probeAccumulating(in, out);
  else if (op == "ncoll_allocators_aligned")
    probeAligned(in, out);
  else if (op == "ncoll_allocators_buffer")
    probeBuffer(in, out);
  else if (op == "ncoll_allocators_occ_eq")
    probeOccEquality(in, out);
  else
    return false;
  return true;
}
