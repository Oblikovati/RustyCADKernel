#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <iostream>

struct ComparisonProbe
{
  int value;
  static inline int equalCalls = 0;
  static inline int notEqualCalls = 0;

  constexpr ComparisonProbe(int theValue = 0) : value(theValue) {}
  bool operator==(const ComparisonProbe& theOther) const
  {
    ++equalCalls;
    return value == theOther.value;
  }
  bool operator!=(const ComparisonProbe& theOther) const
  {
    ++notEqualCalls;
    return value != theOther.value;
  }
  static void Reset()
  {
    equalCalls = 0;
    notEqualCalls = 0;
  }
};

template <class Matrix>
void FillPair(Matrix& theFirst, Matrix& theSecond, int theSize, int theMismatch)
{
  for (int anIndex = 0; anIndex < theSize * theSize; ++anIndex)
  {
    const size_t aRow = size_t(anIndex % theSize);
    const size_t aCol = size_t(anIndex / theSize);
    theFirst.SetValue(aRow, aCol, ComparisonProbe(1));
    theSecond.SetValue(aRow, aCol, ComparisonProbe(anIndex == theMismatch ? 2 : 1));
  }
}

template <class Matrix>
void ReportOperation(const char* theName, const char* theOperation, bool theResult, int theSize)
{
  std::cout << theName << ' ' << theOperation << ' ' << theResult << " eq="
            << ComparisonProbe::equalCalls << " ne=" << ComparisonProbe::notEqualCalls
            << " expected-ne=" << theSize << '\n';
}

template <class Matrix>
void RunMatrix(const char* theName, int theSize, int theMismatch)
{
  Matrix aFirst, aSecond;
  FillPair(aFirst, aSecond, theSize, theMismatch);
  ComparisonProbe::Reset();
  const bool anIsEqual = aFirst.IsEqual(aSecond);
  ReportOperation<Matrix>(theName, "IsEqual", anIsEqual, theMismatch < 0 ? theSize * theSize : theMismatch + 1);
  ComparisonProbe::Reset();
  const bool anOperatorEqual = aFirst == aSecond;
  ReportOperation<Matrix>(theName, "operator==", anOperatorEqual, theMismatch < 0 ? theSize * theSize : theMismatch + 1);
  ComparisonProbe::Reset();
  const bool anOperatorNotEqual = aFirst != aSecond;
  ReportOperation<Matrix>(theName, "operator!=", anOperatorNotEqual, theMismatch < 0 ? theSize * theSize : theMismatch + 1);
}

int main()
{
  RunMatrix<NCollection_Mat3<ComparisonProbe>>("Mat3-equal", 3, -1);
  RunMatrix<NCollection_Mat3<ComparisonProbe>>("Mat3-mismatch", 3, 4);
  RunMatrix<NCollection_Mat4<ComparisonProbe>>("Mat4-equal", 4, -1);
  RunMatrix<NCollection_Mat4<ComparisonProbe>>("Mat4-mismatch", 4, 4);
}
