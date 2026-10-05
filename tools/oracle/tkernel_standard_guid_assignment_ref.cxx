#include <Standard_GUID.hxx>
#include <iostream>

static void printGuid(const char* name, const Standard_GUID& guid)
{
  char text[Standard_GUID_SIZE_ALLOC];
  guid.ToCString(text);
  std::cout << name << ' ' << text << '\n';
}

int main()
{
  Standard_GUID target("12345678-1234-5678-9abc-def012345678");
  Standard_GUID source("ffffffff-0001-0002-0304-05060708090a");
  Standard_GUID& assignedGuid = target = source;
  std::cout << "guid_returns_target " << (&assignedGuid == &target) << '\n';
  printGuid("guid_assignment", target);

  const Standard_UUID uuid{0x80000000, 1, 2, {3, 4, 5, 6, 7, 8, 9, 10}};
  Standard_GUID& assignedUuid = target = uuid;
  std::cout << "uuid_returns_target " << (&assignedUuid == &target) << '\n';
  printGuid("uuid_assignment", target);

  Standard_GUID left;
  Standard_GUID right;
  Standard_GUID& chained = left = (right = uuid);
  std::cout << "chain_returns_left " << (&chained == &left) << '\n';
  printGuid("chain_left", left);
  printGuid("chain_right", right);
}
