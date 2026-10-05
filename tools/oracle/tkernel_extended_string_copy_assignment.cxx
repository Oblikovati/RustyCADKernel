#include <TCollection_ExtendedString.hxx>

#include <iostream>
#include <type_traits>
#include <utility>

static_assert(std::is_same_v<decltype(std::declval<TCollection_ExtendedString&>() =
                                      std::declval<const TCollection_ExtendedString&>()),
                             TCollection_ExtendedString&>);

int main()
{
  TCollection_ExtendedString target(u"old");
  TCollection_ExtendedString first(u"first");
  TCollection_ExtendedString second(u"second");
  TCollection_ExtendedString empty;
  TCollection_ExtendedString* const targetAddress = &target;
  TCollection_ExtendedString& assigned = (target = first);
  std::cout << "assignment_receiver_identity " << (&assigned == targetAddress) << '\n';
  std::cout << "assignment_value " << target.IsEqual(first) << '\n';
  TCollection_ExtendedString& chained = (target = first) = second;
  std::cout << "assignment_chain_identity " << (&chained == targetAddress) << '\n';
  std::cout << "assignment_chain_value " << target.IsEqual(second) << '\n';
  TCollection_ExtendedString& selfAssigned = (target = target);
  std::cout << "self_assignment_identity " << (&selfAssigned == targetAddress) << '\n';
  std::cout << "self_assignment_value " << target.IsEqual(second) << '\n';
  TCollection_ExtendedString& emptied = (target = empty);
  std::cout << "empty_assignment_identity " << (&emptied == targetAddress) << '\n';
  std::cout << "empty_assignment_value " << target.IsEmpty() << '\n';
}
