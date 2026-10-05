#include <NCollection_DynamicArray.hxx>

#include <iostream>
#include <stdexcept>
#include <utility>

struct CopyBomb
{
  int value;
  static int copyCount;
  static int throwOnCopy;

  explicit CopyBomb(int theValue) : value(theValue) {}
  CopyBomb(const CopyBomb& theOther) : value(theOther.value)
  {
    if (++copyCount == throwOnCopy)
    {
      throw std::runtime_error("selected copy");
    }
  }
  CopyBomb(CopyBomb&& theOther) noexcept : value(theOther.value) {}
};

int CopyBomb::copyCount   = 0;
int CopyBomb::throwOnCopy = 0;

static void print_values(const char* label, const NCollection_DynamicArray<int>& array)
{
  std::cout << label << ' ' << array.Size();
  for (size_t anIndex = 0; anIndex < array.Size(); ++anIndex)
  {
    std::cout << ' ' << array.Value(anIndex);
  }
  std::cout << '\n';
}

static void print_assignment_results()
{
  NCollection_DynamicArray<int> aSource(2, NCollection_BaseAllocator::CommonBaseAllocator());
  aSource.Append(10);
  aSource.Append(20);
  NCollection_DynamicArray<int> aOther(2);
  aOther.Append(30);
  NCollection_DynamicArray<int> aTarget(4);
  const bool isAssignReceiver = &aTarget.Assign(aSource) == &aTarget;
  std::cout << "assign_keep_null_allocator " << aTarget.Allocator().IsNull() << '\n';
  aTarget.Assign(aSource, false);
  std::cout << "assign_adopt_allocator " << aTarget.Allocator().IsNull() << '\n';
  const bool isCopyReceiver = &(aTarget = aOther) == &aTarget;
  (aTarget = aSource) = aOther;
  std::cout << "assign_reference " << isAssignReceiver << '\n';
  std::cout << "copy_operator_reference " << isCopyReceiver << '\n';
  print_values("copy_operator_chain", aTarget);
}

static void print_move_results()
{
  NCollection_DynamicArray<int> aMoveSource(2, NCollection_BaseAllocator::CommonBaseAllocator());
  NCollection_DynamicArray<int> aTarget(4);
  aMoveSource.Append(40);
  aMoveSource.Append(50);
  const bool isAssignMoveReceiver = &aTarget.Assign(std::move(aMoveSource)) == &aTarget;
  std::cout << "move_assign_reference " << isAssignMoveReceiver << '\n';
  std::cout << "move_target_allocator_null " << aTarget.Allocator().IsNull() << '\n';
  print_values("move_target", aTarget);
  std::cout << "move_source_allocator_null " << aMoveSource.Allocator().IsNull() << '\n';
  print_values("move_source", aMoveSource);
  NCollection_DynamicArray<int> anOperatorSource(1);
  anOperatorSource.Append(60);
  const bool isMoveOperatorReceiver = &(aTarget = std::move(anOperatorSource)) == &aTarget;
  std::cout << "move_operator_reference " << isMoveOperatorReceiver << '\n';
  print_values("move_operator_target", aTarget);
}

static void print_failed_copy_values(const char*                                theLabel,
                                     const NCollection_DynamicArray<CopyBomb>& theTarget)
{
  std::cout << theLabel << ' ' << theTarget.Size();
  for (size_t anIndex = 0; anIndex < theTarget.Size(); ++anIndex)
  {
    std::cout << ' ' << theTarget.Value(anIndex).value;
  }
  std::cout << '\n';
}

static void print_copy_failure_state(const NCollection_DynamicArray<CopyBomb>& theSource,
                                     NCollection_DynamicArray<CopyBomb>&       theTarget,
                                     const int                                  theThrowOnCopy,
                                     const char*                                theLabel)
{
  CopyBomb::copyCount   = 0;
  CopyBomb::throwOnCopy = theThrowOnCopy;
  try
  {
    theTarget.Assign(theSource);
  }
  catch (const std::runtime_error&)
  {
    print_failed_copy_values(theLabel, theTarget);
  }
}

static void print_copy_failure_results()
{
  NCollection_DynamicArray<CopyBomb> aBombSource(2);
  aBombSource.Append(CopyBomb(1));
  aBombSource.Append(CopyBomb(2));
  aBombSource.Append(CopyBomb(3));
  NCollection_DynamicArray<CopyBomb> aBombTarget(2);
  aBombTarget.Append(CopyBomb(99));
  print_copy_failure_state(aBombSource, aBombTarget, 2, "first_block_copy_throw");
  print_copy_failure_state(aBombSource, aBombTarget, 3, "later_block_copy_throw");
}

int main()
{
  print_assignment_results();
  print_move_results();
  print_copy_failure_results();
}
