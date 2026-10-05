#include <Standard_Persistent.hxx>
#include <Standard_Type.hxx>

#include <iostream>
#include <type_traits>
#include <utility>

using StandardTypeHandle = occ::handle<Standard_Type>;
static_assert(std::is_same_v<decltype(Standard_Type::get_type_name()), const char*>);
static_assert(std::is_same_v<decltype(Standard_Type::get_type_descriptor()), const StandardTypeHandle&>);
static_assert(std::is_same_v<decltype(std::declval<const Standard_Type&>().DynamicType()), const StandardTypeHandle&>);
static_assert(std::is_same_v<decltype(Standard_Persistent::get_type_name()), const char*>);
static_assert(std::is_same_v<decltype(Standard_Persistent::get_type_descriptor()), const StandardTypeHandle&>);
static_assert(std::is_same_v<decltype(std::declval<const Standard_Persistent&>().DynamicType()), const StandardTypeHandle&>);

int main()
{
  const StandardTypeHandle& aType = Standard_Type::get_type_descriptor();
  const StandardTypeHandle& aPersistent = Standard_Persistent::get_type_descriptor();
  const StandardTypeHandle& aTypeDynamic = aType->DynamicType();
  Standard_Persistent anObject;
  const StandardTypeHandle& aDynamic = anObject.DynamicType();
  std::cout << "type_name " << Standard_Type::get_type_name() << '\n';
  std::cout << "type_descriptor " << aType->Name() << ' ' << aType->Parent()->Name() << ' '
            << aType->GetRefCount() << '\n';
  std::cout << "type_dynamic " << aTypeDynamic->Name() << ' '
            << aTypeDynamic->GetRefCount() << '\n';
  std::cout << "persistent_name " << Standard_Persistent::get_type_name() << '\n';
  std::cout << "persistent_descriptor " << aPersistent->Name() << ' '
            << aPersistent->Parent()->Name() << ' ' << aPersistent->GetRefCount() << '\n';
  std::cout << "dynamic " << aDynamic->Name() << ' ' << aDynamic->GetRefCount() << '\n';
  {
    StandardTypeHandle aCopy = aPersistent;
    std::cout << "owning_copy " << aCopy->GetRefCount() << '\n';
  }
  std::cout << "after_copy " << aPersistent->GetRefCount() << '\n';
}
