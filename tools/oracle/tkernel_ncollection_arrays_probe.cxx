// NCollection arrays probe operations (twin of oracle_probe/ncollection/arrays.rs).
// Bound arithmetic, the Array2 layout, and the observable growth rules of the
// LinearVector, SparseArray and DynamicArray storage.
#include <NCollection_AliasedArray.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <NCollection_DynamicArray.hxx>
#include <NCollection_LinearVector.hxx>
#include <NCollection_SparseArray.hxx>
#include <Standard_Failure.hxx>
#include <istream>
#include <ostream>
#include <string>

// "ncoll_arrays_array1 lower upper new_upper": bounds before and after UpdateUpperBound.
static void ncoll_arrays_array1(std::istream& in, std::ostream& out)
{
  int aLower, anUpper, aNewUpper;
  in >> aLower >> anUpper >> aNewUpper;
  NCollection_Array1<int> anArray(aLower, anUpper);
  out << anArray.Length() << ' ' << anArray.Lower() << ' ' << anArray.Upper() << ' '
      << NCollection_Array1<int>::RangeSize(aLower, anUpper);
  anArray.UpdateUpperBound(aNewUpper);
  out << ' ' << anArray.Lower() << ' ' << anArray.Upper();
}

// "ncoll_arrays_range lower upper size": static bound helpers, any sign.
static void ncoll_arrays_range(std::istream& in, std::ostream& out)
{
  int    aLower, anUpper;
  size_t aSize;
  in >> aLower >> anUpper >> aSize;
  out << NCollection_Array1<int>::RangeSize(aLower, anUpper) << ' '
      << NCollection_Array2<int>::RangeSize(aLower, anUpper) << ' '
      << NCollection_Array2<int>::UpperForSize(aLower, aSize) << ' '
      << NCollection_Array2<int>::ArraySize(aSize, 3);
}

// "ncoll_arrays_layout rl ru cl cu": positions, then the constructed rectangle.
static void ncoll_arrays_layout(std::istream& in, std::ostream& out)
{
  int aRL, aRU, aCL, aCU;
  in >> aRL >> aRU >> aCL >> aCU;
  out << NCollection_Array2<int>::BeginPosition(aRL, aRU, aCL, aCU) << ' '
      << NCollection_Array2<int>::LastPosition(aRL, aRU, aCL, aCU);
  NCollection_Array2<int> aGrid(aRL, aRU, aCL, aCU);
  out << ' ' << aGrid.Length() << ' ' << aGrid.NbRows() << ' ' << aGrid.NbColumns() << ' '
      << aGrid.RowLength() << ' ' << aGrid.ColLength() << ' ' << aGrid.IsEmpty() << ' '
      << aGrid.UpperRow() << ' ' << aGrid.UpperCol() << ' ' << aGrid.Array1().Lower() << ' '
      << aGrid.Array1().Upper();
  aGrid.UpdateUpperRow(aRU + 7);
  aGrid.UpdateUpperCol(aCU - 3);
  out << ' ' << aGrid.LowerRow() << ' ' << aGrid.LowerCol();
}

// "ncoll_arrays_aliased stride count": shape queries; count 0 uses the stride-only constructor.
static void ncoll_arrays_aliased(std::istream& in, std::ostream& out)
{
  size_t aStride, aCount;
  in >> aStride >> aCount;
  NCollection_AliasedArray<> anArray =
    aCount == 0 ? NCollection_AliasedArray<>(aStride) : NCollection_AliasedArray<>(aStride, aCount);
  out << anArray.Lower() << ' ' << anArray.Upper() << ' ' << anArray.IsAllocated() << ' '
      << anArray.SizeBytes() << ' ' << anArray.Length();
}

// "ncoll_arrays_linear op...": a=append r<n>=Reserve R<n>=Resize c=Clear(false)
// C=Clear(true) e<i>=Erase i<i>=InsertBefore(i); prints Size/Capacity/HasData after each op.
static void ncoll_arrays_linear(std::istream& in, std::ostream& out)
{
  NCollection_LinearVector<int> aVector;
  std::string                   anOp;
  int                           aCounter = 0;
  while (in >> anOp)
  {
    const size_t anArg = anOp.size() > 1 ? std::stoul(anOp.substr(1)) : 0;
    switch (anOp[0])
    {
      case 'a': aVector.Append(++aCounter); break;
      case 'r': aVector.Reserve(anArg); break;
      case 'R': aVector.Resize(anArg); break;
      case 'c': aVector.Clear(false); break;
      case 'C': aVector.Clear(true); break;
      case 'e': aVector.Erase(anArg); break;
      case 'i': aVector.InsertBefore(anArg, ++aCounter); break;
    }
    out << aVector.Size() << ':' << aVector.Capacity() << ':' << aVector.HasData() << ' ';
  }
  for (const int aValue : aVector)
  {
    out << aValue << ',';
  }
}

// "ncoll_arrays_sparse increment op...": s<i>=SetValue u<i>=UnsetValue c=Clear;
// prints Capacity/Size after each op, then the defined keys in iteration order.
static void ncoll_arrays_sparse(std::istream& in, std::ostream& out)
{
  size_t anIncrement;
  in >> anIncrement;
  NCollection_SparseArray<int> anArray(anIncrement);
  std::string                  anOp;
  while (in >> anOp)
  {
    const size_t anIndex = anOp.size() > 1 ? std::stoul(anOp.substr(1)) : 0;
    if (anOp[0] == 's')
      anArray.SetValue(anIndex, static_cast<int>(anIndex));
    else if (anOp[0] == 'u')
      out << anArray.UnsetValue(anIndex) << '/';
    else
      anArray.Clear();
    out << anArray.Capacity() << ':' << anArray.Size() << ' ';
  }
  for (NCollection_SparseArray<int>::ConstIterator anIter(anArray); anIter.More(); anIter.Next())
  {
    out << anIter.Key() << ',';
  }
}

// "ncoll_arrays_sparse_assign src_increment n keys... dst_increment n keys...":
// Assign, then Exchange with an empty increment-5 array.
static void ncoll_arrays_sparse_assign(std::istream& in, std::ostream& out)
{
  size_t anIncrements[2], aCount;
  NCollection_SparseArray<int>* anArrays[2];
  for (int aSide = 0; aSide < 2; ++aSide)
  {
    in >> anIncrements[aSide] >> aCount;
    anArrays[aSide] = new NCollection_SparseArray<int>(anIncrements[aSide]);
    for (size_t anIter = 0, aKey; anIter < aCount && in >> aKey; ++anIter)
      anArrays[aSide]->SetValue(aKey, static_cast<int>(aKey) * 2 + aSide);
  }
  anArrays[1]->Assign(*anArrays[0]);
  out << anArrays[1]->Capacity() << ' ' << anArrays[1]->Size();
  for (NCollection_SparseArray<int>::ConstIterator anIter(*anArrays[1]); anIter.More(); anIter.Next())
    out << ' ' << anIter.Key() << '=' << anIter.Value();
  NCollection_SparseArray<int> anEmpty(5);
  anEmpty.Exchange(*anArrays[1]);
  out << ' ' << anEmpty.Capacity() << ' ' << anArrays[1]->Capacity();
  delete anArrays[0];
  delete anArrays[1];
}

// "ncoll_arrays_dynamic increment op...": a<v>=Append s<i>,<v>=SetValue b<i>,<v>=InsertBefore
// f<i>,<v>=InsertAfter e=EraseLast c=Clear(false); prints Length/Upper and the values.
static void ncoll_arrays_dynamic(std::istream& in, std::ostream& out)
{
  int anIncrement;
  in >> anIncrement;
  NCollection_DynamicArray<int> anArray(anIncrement);
  std::string                   anOp;
  while (in >> anOp)
  {
    const size_t aComma = anOp.find(',');
    const int    anA    = anOp.size() > 1 ? std::stoi(anOp.substr(1, aComma - 1)) : 0;
    const int    aB     = aComma == std::string::npos ? 0 : std::stoi(anOp.substr(aComma + 1));
    switch (anOp[0])
    {
      case 'a': anArray.Append(anA); break;
      case 's': anArray.SetValue(anA, aB); break;
      case 'b': anArray.InsertBefore(anA, aB); break;
      case 'f': anArray.InsertAfter(anA, aB); break;
      case 'e': anArray.EraseLast(); break;
      case 'c': anArray.Clear(false); break;
    }
  }
  out << anArray.Length() << ' ' << anArray.Upper();
  for (NCollection_DynamicArray<int>::ConstIterator anIter(anArray); anIter.More(); anIter.Next())
    out << ' ' << anIter.Value();
}

using NCollectionArraysProbe = void (*)(std::istream&, std::ostream&);

static bool ncollection_arrays_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  static const struct
  {
    const char*            name;
    NCollectionArraysProbe probe;
  } THE_PROBES[] = {
    {"ncoll_arrays_array1", ncoll_arrays_array1},
    {"ncoll_arrays_range", ncoll_arrays_range},
    {"ncoll_arrays_layout", ncoll_arrays_layout},
    {"ncoll_arrays_aliased", ncoll_arrays_aliased},
    {"ncoll_arrays_linear", ncoll_arrays_linear},
    {"ncoll_arrays_sparse", ncoll_arrays_sparse},
    {"ncoll_arrays_sparse_assign", ncoll_arrays_sparse_assign},
    {"ncoll_arrays_dynamic", ncoll_arrays_dynamic},
  };
  for (const auto& anEntry : THE_PROBES)
  {
    if (op == anEntry.name)
    {
      try
      {
        anEntry.probe(in, out);
      }
      catch (const Standard_Failure& theFailure)
      {
        out << "throw " << theFailure.ExceptionType();
      }
      return true;
    }
  }
  return false;
}
