// NCollection flat maps probe operations (twin of oracle_probe/ncollection/flat_maps.rs).
// Each request is a script of one-letter commands with integer arguments applied to one
// map; every command prints its observable result, then the map's final state is dumped.
#include <NCollection_FlatDataMap.hxx>
#include <NCollection_FlatMap.hxx>
#include <NCollection_OrderedDataMap.hxx>
#include <NCollection_OrderedMap.hxx>
#include <NCollection_PackedMap.hxx>
#include <NCollection_PackedMapAlgo.hxx>
#include <Standard_Failure.hxx>

#include <cstdint>
#include <istream>
#include <ostream>
#include <string>
#include <vector>

// A native exception becomes its class name, the text the Rust twin prints for its error.
template <class Action>
static void ncoll_flat_guarded(std::ostream& out, Action theAction)
{
  try
  {
    theAction();
  }
  catch (const Standard_Failure& theFailure)
  {
    out << " throw " << theFailure.ExceptionType();
  }
}

static void ncoll_flat_set(std::istream& in, std::ostream& out)
{
  NCollection_FlatMap<int> aMap;
  char                     aCmd;
  int                      aKey;
  while (in >> aCmd >> aKey)
  {
    switch (aCmd)
    {
      case 'a': out << ' ' << aMap.Add(aKey); break;
      case 'd': out << ' ' << aMap.Added(aKey); break;
      case 'e': out << ' ' << aMap.Emplaced(aKey); break;
      case 'E': out << ' ' << aMap.Emplace(aKey); break;
      case 't': out << ' ' << aMap.TryEmplaced(aKey); break;
      case 'T': out << ' ' << aMap.TryEmplace(aKey); break;
      case 'r': out << ' ' << aMap.Remove(aKey); break;
      case 'q': out << ' ' << (aMap.Contained(aKey) ? aMap.Contained(aKey)->get() : -1); break;
      case 'R': aMap.Reserve(static_cast<size_t>(aKey)); break;
      case 'c': aMap.Clear(aKey != 0); break;
      case 'm': {
        NCollection_FlatMap<int> aTarget(size_t(40));
        aTarget.Add(1000);
        aTarget = std::move(aMap);
        out << " moved " << aMap.Size() << ' ' << aMap.Capacity() << ' ' << aTarget.Size();
        aMap = aTarget;
        break;
      }
    }
  }
  out << " | " << aMap.Size() << ' ' << aMap.Capacity() << " :";
  for (NCollection_FlatMap<int>::Iterator anIter(aMap); anIter.More(); anIter.Next())
    out << ' ' << anIter.Key();
  NCollection_FlatMap<int>::Iterator anEnd;
  out << " end " << anEnd.More() << ' ' << anEnd.IsEqual(NCollection_FlatMap<int>::Iterator(aMap));
}

static void ncoll_flat_data(std::istream& in, std::ostream& out)
{
  NCollection_FlatDataMap<int, int> aMap;
  char                              aCmd;
  int                               aKey;
  while (in >> aCmd >> aKey)
  {
    int aValue = 0;
    if (aCmd == 'b' || aCmd == 'B' || aCmd == 't' || aCmd == 'T' || aCmd == 'e' || aCmd == 'E')
      in >> aValue;
    switch (aCmd)
    {
      case 'b': out << ' ' << aMap.Bind(aKey, aValue); break;
      case 'B': out << ' ' << aMap.Bound(aKey, aValue); break;
      case 't': out << ' ' << aMap.TryBind(aKey, aValue); break;
      case 'T': out << ' ' << aMap.TryBound(aKey, aValue); break;
      case 'e': out << ' ' << aMap.Emplaced(aKey, aValue); break;
      case 'E': out << ' ' << aMap.TryEmplaced(aKey, aValue); break;
      case 'u': out << ' ' << aMap.UnBind(aKey); break;
      case 'f': ncoll_flat_guarded(out, [&] { out << ' ' << aMap.Find(aKey); }); break;
      case 'q': {
        auto aPair = aMap.Contained(aKey);
        if (aPair)
          out << ' ' << aPair->first.get() << ':' << aPair->second.get();
        else
          out << " none";
        break;
      }
      case 'R': aMap.Reserve(static_cast<size_t>(aKey)); break;
      case 'c': aMap.Clear(aKey != 0); break;
    }
  }
  out << " | " << aMap.Size() << ' ' << aMap.Extent() << ' ' << aMap.Capacity() << " :";
  for (NCollection_FlatDataMap<int, int>::Iterator anIter(aMap); anIter.More(); anIter.Next())
    out << ' ' << anIter.Key() << ':' << anIter.Value();
}

template <class TheMap>
static void ncoll_flat_ordered_edges(const TheMap& theMap, std::ostream& out)
{
  ncoll_flat_guarded(out, [&] { out << " first " << theMap.First(); });
  ncoll_flat_guarded(out, [&] { out << " last " << theMap.Last(); });
}

static void ncoll_flat_ordered(std::istream& in, std::ostream& out)
{
  int aBuckets = 0;
  in >> aBuckets;
  ncoll_flat_guarded(out, [&] {
    NCollection_OrderedMap<int> aMap(aBuckets);
    out << " nb " << aMap.NbBuckets();
    char aCmd;
    int  aKey;
    while (in >> aCmd >> aKey)
    {
      switch (aCmd)
      {
        case 'a': out << ' ' << aMap.Add(aKey); break;
        case 'd': out << ' ' << aMap.Added(aKey); break;
        case 'e': out << ' ' << aMap.Emplaced(aKey); break;
        case 't': out << ' ' << aMap.TryEmplaced(aKey); break;
        case 'r': out << ' ' << aMap.Remove(aKey); break;
        case 'q': out << ' ' << (aMap.Contained(aKey) ? aMap.Contained(aKey)->get() : -1); break;
        case 'R': ncoll_flat_guarded(out, [&] { aMap.ReSize(aKey); }); break;
        case 'c': aMap.Clear(aKey != 0); break;
        case 'C': aMap.Clear(occ::handle<NCollection_BaseAllocator>()); break;
        case 'N': out << " nb " << aMap.NbBuckets(); break;
        case 'm': {
          NCollection_OrderedMap<int> aTarget(size_t(3));
          aTarget.Add(1000);
          aTarget = std::move(aMap);
          out << " moved " << aTarget.Extent() << " source";
          for (NCollection_OrderedMap<int>::Iterator anIter(aMap); anIter.More(); anIter.Next())
            out << ' ' << anIter.Key();
          aMap = aTarget;
          break;
        }
      }
    }
    out << " | " << aMap.Extent() << " nb " << aMap.NbBuckets() << " :";
    for (auto aNode = aMap.FirstNode(); aNode != nullptr; aNode = aNode->OrderNext())
      out << ' ' << aNode->Key();
    NCollection_OrderedMap<int>::Iterator anIter(aMap);
    if (anIter.More())
      anIter.Next();
    anIter.Reset();
    out << " reset " << (anIter.More() ? anIter.Key() : -1);
    ncoll_flat_ordered_edges(aMap, out);
  });
}

static void ncoll_flat_ordered_data(std::istream& in, std::ostream& out)
{
  int aBuckets = 0;
  in >> aBuckets;
  ncoll_flat_guarded(out, [&] {
    NCollection_OrderedDataMap<int, int> aMap(aBuckets);
    out << " nb " << aMap.NbBuckets();
    char aCmd;
    int  aKey;
    while (in >> aCmd >> aKey)
    {
      int aValue = 0;
      if (aCmd == 'b' || aCmd == 'B' || aCmd == 't' || aCmd == 'T' || aCmd == 'e' || aCmd == 'E')
        in >> aValue;
      switch (aCmd)
      {
        case 'b': out << ' ' << aMap.Bind(aKey, aValue); break;
        case 'B': out << ' ' << *aMap.Bound(aKey, aValue); break;
        case 't': out << ' ' << aMap.TryBind(aKey, aValue); break;
        case 'T': out << ' ' << aMap.TryBound(aKey, aValue); break;
        case 'e': out << ' ' << aMap.Emplaced(aKey, aValue); break;
        case 'E': out << ' ' << aMap.TryEmplaced(aKey, aValue); break;
        case 'u': out << ' ' << aMap.UnBind(aKey); break;
        case 'f': ncoll_flat_guarded(out, [&] { out << ' ' << aMap.Find(aKey); }); break;
        case 'F': {
          int aCopy = -1;
          out << ' ' << aMap.Find(aKey, aCopy) << ':' << aCopy;
          break;
        }
        case 'R': ncoll_flat_guarded(out, [&] { aMap.ReSize(aKey); }); break;
        case 'c': aMap.Clear(aKey != 0); break;
        case 'C': aMap.Clear(occ::handle<NCollection_BaseAllocator>()); break;
        case 'n': {
          auto aNode = aMap.FirstNode();
          if (aNode != nullptr)
            aNode->ChangeValue() = aKey;
          break;
        }
        case 'm': {
          NCollection_OrderedDataMap<int, int> aTarget(size_t(3));
          aTarget.Bind(1000, 1);
          aTarget = std::move(aMap);
          out << " moved " << aTarget.Extent() << " source";
          for (NCollection_OrderedDataMap<int, int>::Iterator anIter(aMap); anIter.More();
               anIter.Next())
            out << ' ' << anIter.Key() << ':' << anIter.Value();
          aMap = aTarget;
          break;
        }
      }
    }
    out << " | " << aMap.Extent() << " nb " << aMap.NbBuckets() << " :";
    for (auto aNode = aMap.FirstNode(); aNode != nullptr; aNode = aNode->OrderNext())
      out << ' ' << aNode->Key() << ':' << aNode->Value();
    NCollection_OrderedDataMap<int, int>::Iterator anIter(aMap);
    if (anIter.More())
      anIter.Next();
    anIter.Reset();
    out << " reset " << (anIter.More() ? anIter.Key() : -1);
    ncoll_flat_ordered_edges(aMap, out);
    ncoll_flat_guarded(out, [&] { out << " values " << aMap.FirstValue() << ' ' << aMap.LastValue(); });
  });
}

template <class IntType>
static void ncoll_flat_packed_dump(const NCollection_PackedMap<IntType>& theMap, std::ostream& out)
{
  out << " [" << theMap.Extent() << ' ' << theMap.Length() << ' ' << theMap.Size() << ' '
      << theMap.NbBuckets() << ' ' << theMap.IsEmpty() << ' ' << theMap.GetMinimalMapped() << ' '
      << theMap.GetMaximalMapped() << " :";
  for (typename NCollection_PackedMap<IntType>::Iterator anIter(theMap); anIter.More();
       anIter.Next())
    out << ' ' << anIter.Key();
  out << ']';
}

template <class IntType>
static std::vector<IntType> ncoll_flat_read_keys(std::istream& in)
{
  size_t aCount = 0;
  in >> aCount;
  std::vector<IntType> aKeys(aCount);
  for (IntType& aKey : aKeys)
    in >> aKey;
  return aKeys;
}

template <class IntType>
static NCollection_PackedMap<IntType> ncoll_flat_packed_of(const std::vector<IntType>& theKeys)
{
  NCollection_PackedMap<IntType> aMap;
  for (const IntType aKey : theKeys)
    aMap.Add(aKey);
  return aMap;
}

// The deprecated members forward to NCollection_PackedMapAlgo; both spellings are dumped.
// They are probed on purpose, so their deprecation warnings are silenced here only.
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wdeprecated-declarations"
template <class IntType>
static void ncoll_flat_packed_algebra(std::istream& in, std::ostream& out)
{
  const std::vector<IntType>           aLeftKeys = ncoll_flat_read_keys<IntType>(in);
  const std::vector<IntType>           aRightKeys = ncoll_flat_read_keys<IntType>(in);
  const NCollection_PackedMap<IntType> aLeft      = ncoll_flat_packed_of(aLeftKeys);
  const NCollection_PackedMap<IntType> aRight     = ncoll_flat_packed_of(aRightKeys);
  NCollection_PackedMap<IntType>       aResult;
  aResult.Union(aLeft, aRight);
  ncoll_flat_packed_dump(aResult, out);
  aResult.Intersection(aLeft, aRight);
  ncoll_flat_packed_dump(aResult, out);
  aResult.Subtraction(aLeft, aRight);
  ncoll_flat_packed_dump(aResult, out);
  aResult.Difference(aLeft, aRight);
  ncoll_flat_packed_dump(aResult, out);
  NCollection_PackedMap<IntType> aWork = aLeft;
  out << ' ' << aWork.Unite(aRight);
  ncoll_flat_packed_dump(aWork, out);
  aWork = aLeft;
  out << ' ' << aWork.Intersect(aRight);
  ncoll_flat_packed_dump(aWork, out);
  aWork = aLeft;
  out << ' ' << aWork.Subtract(aRight);
  ncoll_flat_packed_dump(aWork, out);
  aWork = aLeft;
  out << ' ' << aWork.Differ(aRight);
  ncoll_flat_packed_dump(aWork, out);
  out << ' ' << aLeft.IsEqual(aRight) << aLeft.IsSubset(aRight) << aLeft.HasIntersection(aRight)
      << aLeft.Contains(aRight) << NCollection_PackedMapAlgo::IsEqual(aLeft, aLeft);
}
#pragma GCC diagnostic pop

static void ncoll_flat_packed(std::istream& in, std::ostream& out)
{
  int aBuckets = 0;
  in >> aBuckets;
  NCollection_PackedMap<int> aMap(aBuckets);
  out << " nb " << aMap.NbBuckets();
  char aCmd;
  int  aKey;
  while (in >> aCmd >> aKey)
  {
    switch (aCmd)
    {
      case 'a': out << ' ' << aMap.Add(aKey); break;
      case 'r': out << ' ' << aMap.Remove(aKey); break;
      case 'h': out << ' ' << aMap.Contains(aKey); break;
      case 'R': aMap.ReSize(aKey); break;
      case 'c': aMap.Clear(); break;
      case 'm': {
        NCollection_PackedMap<int> aTarget(size_t(7));
        aTarget.Add(1000);
        aTarget = std::move(aMap);
        out << " moved " << aMap.NbBuckets() << ' ' << aMap.Extent();
        aMap = aTarget;
        break;
      }
    }
  }
  ncoll_flat_packed_dump(aMap, out);
  NCollection_PackedMap<int>::Iterator anIter(aMap);
  if (anIter.More())
    anIter.Next();
  anIter.Reset();
  NCollection_PackedMap<int>::Iterator anEmpty;
  out << " reset " << (anIter.More() ? anIter.Key() : -1) << ' ' << anEmpty.More();
}

static bool ncollection_flat_maps_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "ncoll_flat_maps_set")
    ncoll_flat_set(in, out);
  else if (op == "ncoll_flat_maps_data")
    ncoll_flat_data(in, out);
  else if (op == "ncoll_flat_maps_ordered")
    ncoll_flat_ordered(in, out);
  else if (op == "ncoll_flat_maps_ordered_data")
    ncoll_flat_ordered_data(in, out);
  else if (op == "ncoll_flat_maps_packed")
    ncoll_flat_packed(in, out);
  else if (op == "ncoll_flat_maps_algebra32")
    ncoll_flat_packed_algebra<int>(in, out);
  else if (op == "ncoll_flat_maps_algebra64")
    ncoll_flat_packed_algebra<int64_t>(in, out);
  else
    return false;
  return true;
}
