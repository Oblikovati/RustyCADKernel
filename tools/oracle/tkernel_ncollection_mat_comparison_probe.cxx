#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>
#include <iostream>

struct ComparisonProbe
{
  int value;
  int* equalCalls;
  int* notEqualCalls;

  constexpr ComparisonProbe(int theValue = 0, int* theEqualCalls = nullptr, int* theNotEqualCalls = nullptr)
      : value(theValue), equalCalls(theEqualCalls), notEqualCalls(theNotEqualCalls) {}
  bool operator==(const ComparisonProbe& theOther) const
  {
    ++*equalCalls;
    return value == theOther.value;
  }
  bool operator!=(const ComparisonProbe& theOther) const
  {
    ++*notEqualCalls;
    return value != theOther.value;
  }
};

template <class Matrix>
void FillPair(Matrix& theFirst, Matrix& theSecond, int theSize, int theMismatch, int* theEqualCalls, int* theNotEqualCalls)
{
  for (int anIndex = 0; anIndex < theSize * theSize; ++anIndex)
  {
    const size_t aRow = size_t(anIndex % theSize);
    const size_t aCol = size_t(anIndex / theSize);
    theFirst.SetValue(aRow, aCol, ComparisonProbe(1, theEqualCalls, theNotEqualCalls));
    theSecond.SetValue(aRow, aCol, ComparisonProbe(anIndex == theMismatch ? 2 : 1, theEqualCalls, theNotEqualCalls));
  }
}

template <class Matrix>
void ReportOperation(const char* theName, const char* theOperation, bool theResult, int theExpectedCalls, int theEqualCalls, int theNotEqualCalls)
{
  std::cout << theName << ' ' << theOperation << ' ' << theResult << " eq="
            << theEqualCalls << " ne=" << theNotEqualCalls
            << " expected-ne=" << theExpectedCalls << '\n';
}

template <class Matrix>
void RunMatrix(const char* theName, int theSize, int theMismatch)
{
  Matrix aFirst, aSecond;
  int anEqualCalls = 0;
  int aNotEqualCalls = 0;
  FillPair(aFirst, aSecond, theSize, theMismatch, &anEqualCalls, &aNotEqualCalls);
  const int anExpectedCalls = theMismatch < 0 ? theSize * theSize : theMismatch + 1;
  const bool anIsEqual = aFirst.IsEqual(aSecond);
  ReportOperation<Matrix>(theName, "IsEqual", anIsEqual, anExpectedCalls, anEqualCalls, aNotEqualCalls);
  anEqualCalls = 0;
  aNotEqualCalls = 0;
  const bool anOperatorEqual = aFirst == aSecond;
  ReportOperation<Matrix>(theName, "operator==", anOperatorEqual, anExpectedCalls, anEqualCalls, aNotEqualCalls);
  anEqualCalls = 0;
  aNotEqualCalls = 0;
  const bool anOperatorNotEqual = aFirst != aSecond;
  ReportOperation<Matrix>(theName, "operator!=", anOperatorNotEqual, anExpectedCalls, anEqualCalls, aNotEqualCalls);
}

int main()
{
  RunMatrix<NCollection_Mat3<ComparisonProbe>>("Mat3-equal", 3, -1);
  RunMatrix<NCollection_Mat3<ComparisonProbe>>("Mat3-mismatch", 3, 4);
  RunMatrix<NCollection_Mat4<ComparisonProbe>>("Mat4-equal", 4, -1);
  RunMatrix<NCollection_Mat4<ComparisonProbe>>("Mat4-mismatch", 4, 4);
}
