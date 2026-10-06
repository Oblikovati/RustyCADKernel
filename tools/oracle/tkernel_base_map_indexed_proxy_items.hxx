// Native proxy/value accessor results intentionally delete copy and move constructors.
#include <NCollection_BaseMap.hxx>
#include <Standard_OutOfRange.hxx>
#include <array>
#include <string>

inline std::string indexed_proxy_calls;
struct IndexedOwnedProbe {
  int number;
  explicit IndexedOwnedProbe(int value) : number(value)
  {
    indexed_proxy_calls += 'c';
  }
  ~IndexedOwnedProbe()
  {
    indexed_proxy_calls += 'd';
  }
  IndexedOwnedProbe(const IndexedOwnedProbe&) = delete;
  IndexedOwnedProbe(IndexedOwnedProbe&&) = delete;
};
struct IndexedWritableProbe {
  int* slot;
  explicit IndexedWritableProbe(int* value) : slot(value)
  {
    indexed_proxy_calls += 'p';
  }
  ~IndexedWritableProbe()
  {
    indexed_proxy_calls += 'q';
  }
  IndexedWritableProbe(const IndexedWritableProbe&) = delete;
  IndexedWritableProbe(IndexedWritableProbe&&) = delete;
  void Set(int value)
  {
    indexed_proxy_calls += '=';
    *slot = value;
  }
};
struct IndexedProxyMap {
  std::array<int, 2> values{10, 20};
  size_t Size() const
  {
    indexed_proxy_calls += 's';
    return values.size();
  }
  void Check(size_t index) const
  {
    if (index >= 1 && index <= values.size()) return;
    indexed_proxy_calls += 'e';
    throw Standard_OutOfRange(("index=" + std::to_string(index) + "; expected 1..=2").c_str());
  }
  IndexedOwnedProbe operator()(size_t index) const
  {
    indexed_proxy_calls += "r" + std::to_string(index);
    Check(index);
    return IndexedOwnedProbe(values[index - 1]);
  }
  IndexedWritableProbe operator()(size_t index)
  {
    indexed_proxy_calls += "w" + std::to_string(index);
    Check(index);
    return IndexedWritableProbe(&values[index - 1]);
  }
  int FindKey(size_t index)
  {
    indexed_proxy_calls += "K" + std::to_string(index);
    Check(index);
    return static_cast<int>(index * 111);
  }
  int FindKey(size_t index) const
  {
    indexed_proxy_calls += "k" + std::to_string(index);
    Check(index);
    return static_cast<int>(index * 11);
  }
};
using IndexedProxyRead = NCollection_BaseMap::IndexedIterator<IndexedProxyMap, int, true>;
using IndexedProxyWrite = NCollection_BaseMap::IndexedIterator<IndexedProxyMap, int, false>;
