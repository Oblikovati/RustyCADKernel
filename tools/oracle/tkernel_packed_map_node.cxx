// Access control is disabled only for this probe of the unmodified native private node.
#include <NCollection_PackedMap.hxx>
#include <cstddef>
#include <cstdint>
#include <iostream>
#include <limits>

template<class T> using Map = NCollection_PackedMap<T>;
template<class T> using Node = typename Map<T>::PackedMapNode;
template<class T> using Word = typename Map<T>::IndexType;

template<class T> void snapshot(const char* name, const char* kind, unsigned step,
                                const Node<T>& node, bool changed) {
  std::cout << name << ':' << kind << ':' << step << ' ' << std::uint64_t(node.Mask())
            << ' ' << std::uint64_t(node.Data()) << ' ' << std::uint64_t(Word<T>(node.Key()))
            << ' ' << node.NbValues() << ' ' << node.HasValues() << ' ' << changed
            << ' ' << node.HashCode(101) << ' ' << node.IsEqual(Map<T>::packedKeyIndex(node.Key())) << '\n';
}

template<class T> void constructors(const char* name) {
  Node<T> tail;
  Node<T>* head = &tail;
  Node<T> empty(head);
  snapshot<T>(name, "constructor", 0, empty, empty.Next() == head);
  const T keys[] = {T(0), T(1), T(-1), T(31), T(32), std::numeric_limits<T>::lowest(), std::numeric_limits<T>::max()};
  for (unsigned index = 0; index < 7; ++index) {
    Node<T> node(keys[index], head);
    snapshot<T>(name, "constructor", index + 1, node, node.Next() == &tail && head == &tail);
  }
  Node<T> raw(Word<T>(~Word<T>(0)), Word<T>(0), head);
  snapshot<T>(name, "constructor", 8, raw, raw.Next() == head);
  raw.ChangeMask() = Word<T>(32); raw.ChangeData() = Word<T>(3); raw.SetNext(nullptr);
  snapshot<T>(name, "constructor", 9, raw, raw.Next() == nullptr);
  std::cout << name << ":layout  " << sizeof(Node<T>) << ' ' << alignof(Node<T>) << ' '
            << offsetof(Node<T>, myNext) << ' ' << offsetof(Node<T>, myMask) << ' ' << offsetof(Node<T>, myData) << '\n';
}

template<class T> void mutations(const char* name) {
  Node<T> node;
  for (unsigned index = 0; index < Map<T>::BitsPerBlock; ++index) {
    const bool changed = node.AddValue(T(index));
    const bool duplicate = node.AddValue(T(index));
    snapshot<T>(name, "add", index, node, changed && !duplicate && node.HasValue(T(index)));
  }
  for (unsigned index = 0; index < Map<T>::BitsPerBlock; ++index) {
    const bool changed = node.DelValue(T(index));
    const bool absent = node.DelValue(T(index));
    snapshot<T>(name, "remove", index, node, changed && !absent && !node.HasValue(T(index)));
  }
  Node<T> wrapped(Word<T>(~Word<T>(0)), Word<T>(0), nullptr);
  snapshot<T>(name, "wrap", 0, wrapped, wrapped.AddValue(T(0)));
  snapshot<T>(name, "wrap", 1, wrapped, wrapped.DelValue(T(0)));
}

template<class T> void cursor_row(const char* name, const char* kind, unsigned index,
                                 Word<T> base, Word<T> bits, Word<T> search) {
  Node<T> node(base, bits, nullptr);
  Word<T> next_mask = search, prev_mask = search;
  const T next = Map<T>::findNext(&node, next_mask);
  const T prev = Map<T>::findPrev(&node, prev_mask);
  std::cout << name << ':' << kind << ':' << index << ' ' << std::uint64_t(Word<T>(next)) << ' '
            << std::uint64_t(next_mask) << ' ' << std::uint64_t(Word<T>(prev)) << ' ' << std::uint64_t(prev_mask) << '\n';
}

template<class T> void cursors(const char* name) {
  const Word<T> full = ~Word<T>(0), high = Word<T>(1) << (Map<T>::BitsPerBlock - 1);
  const Word<T> bases[] = {Word<T>(0), Word<T>(std::numeric_limits<T>::lowest()), Word<T>(std::numeric_limits<T>::max())};
  const Word<T> bits[] = {Word<T>(0), Word<T>(1), high, full, Word<T>(0xaaaaaaaaaaaaaaaaULL), Word<T>(0x5555555555555555ULL), Word<T>(0x80000001)};
  const Word<T> masks[] = {Word<T>(0), full, Word<T>(1), high, Word<T>(full ^ 1)};
  for (unsigned index = 0; index < 105; ++index)
    cursor_row<T>(name, "cursor", index, bases[index / 35], bits[index / 5 % 7], masks[index % 5]);
  for (unsigned index = 0; index < Map<T>::BitsPerBlock; ++index)
    cursor_row<T>(name, "one_bit", index, Word<T>(32), Word<T>(1) << index, full);
}

template<class T> void width(const char* name) {
  constructors<T>(name);
  mutations<T>(name);
  cursors<T>(name);
}
int main() {
  width<std::int32_t>("i32"); width<std::uint32_t>("u32");
  width<std::int64_t>("i64"); width<std::uint64_t>("u64");
  width<std::int8_t>("i8"); width<bool>("bool");
}
