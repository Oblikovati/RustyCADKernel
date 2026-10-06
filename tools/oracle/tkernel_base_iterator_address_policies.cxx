// OCCT uses overloaded unary & after its Value/ChangeValue getter dispatch.
#include <NCollection_BaseList.hxx>
#include <NCollection_BaseSequence.hxx>
#include <iostream>
#include <memory>
#include <string>
#include <utility>

static std::string address_calls;
struct AddressItem {
  int number;
  AddressItem* alternate;
  const AddressItem* operator&() const
  {
    address_calls += 'a';
    return alternate;
  }
  AddressItem* operator&()
  {
    address_calls += 'A';
    ++number;
    return alternate;
  }
};
struct ListLink : NCollection_ListNode {
  ListLink() : NCollection_ListNode(nullptr) {}
};
template<class Link>
struct AddressNode : Link {
  AddressItem item;
  AddressNode(int number, AddressItem* alternate) : item{number, alternate} {}
  const AddressItem& Value() const
  {
    address_calls += 'v';
    return item;
  }
  AddressItem& ChangeValue()
  {
    address_calls += 'V';
    return item;
  }
};
using ListAddressNode = AddressNode<ListLink>;
using SequenceAddressNode = AddressNode<NCollection_SeqNode>;
struct AddressList : NCollection_BaseList {
  template<bool Constant> using Cursor = BasicIterator<ListAddressNode, AddressItem, Constant>;
  explicit AddressList(ListAddressNode& node) { PAppend(std::addressof(node)); }
};
struct AddressSequence : NCollection_BaseSequence {
  template<bool Constant> using Cursor = BasicIterator<SequenceAddressNode, AddressItem, Constant>;
  explicit AddressSequence(SequenceAddressNode& node) : NCollection_BaseSequence(nullptr)
  {
    PAppend(std::addressof(node));
  }
};
static std::string take_address_calls()
{
  return std::exchange(address_calls, {});
}
template<class Node, class Read, class Write>
static void builtin_rows(const char* label, Node& node, const Read& readonly, Write& writable)
{
  const AddressItem* direct = std::addressof(*readonly);
  std::cout << label << "_builtin_const " << (direct == std::addressof(node.item)) << ' '
            << direct->number << ' ' << take_address_calls() << '\n';
  AddressItem* mutable_direct = std::addressof(*writable);
  std::cout << label << "_builtin_mut " << (mutable_direct == std::addressof(node.item)) << ' '
            << mutable_direct->number << ' ' << take_address_calls() << '\n';
}
template<class Container, class Node>
static void address_rows(const char* label, Node& node, AddressItem& target)
{
  Container container(node);
  typename Container::template Cursor<true> readonly(container);
  typename Container::template Cursor<false> writable(container);
  const AddressItem* read = readonly.operator->();
  std::cout << label << "_const " << (read == std::addressof(target)) << ' '
            << read->number << ' ' << node.item.number << ' ' << take_address_calls() << '\n';
  AddressItem* write = writable.operator->();
  std::cout << label << "_mut " << (write == std::addressof(target)) << ' '
            << write->number << ' ' << node.item.number << ' ' << take_address_calls() << '\n';
  builtin_rows(label, node, readonly, writable);
}
int main()
{
  AddressItem list_target{30, nullptr}, sequence_target{70, nullptr};
  ListAddressNode list_node(10, std::addressof(list_target));
  SequenceAddressNode sequence_node(50, std::addressof(sequence_target));
  address_rows<AddressList>("list", list_node, list_target);
  address_rows<AddressSequence>("sequence", sequence_node, sequence_target);
}
