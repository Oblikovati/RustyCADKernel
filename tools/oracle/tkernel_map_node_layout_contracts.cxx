// Measure the pinned compiler's actual native object layout; base tail padding can be reused.
#include <NCollection_DataMap.hxx>
#include <NCollection_Map.hxx>
#include <iostream>
#include <cstdint>
template<class K,class V> void data_layout(const char* label) {
  using N=typename NCollection_DataMap<K,V>::DataMapNode;
  N node(K{},V{},nullptr);const auto start=reinterpret_cast<std::uintptr_t>(&node);
  std::cout<<label<<' '<<sizeof(N)<<' '<<alignof(N)<<' '
    <<(reinterpret_cast<std::uintptr_t>(&node.Key())-start)<<' '
    <<(reinterpret_cast<std::uintptr_t>(&node.Value())-start)<<' '
    <<sizeof(NCollection_TListNode<V>)<<'\n';
}
template<class K> void map_layout(const char* label) {
  using N=typename NCollection_Map<K>::MapNode;
  N node(K{},nullptr);const auto start=reinterpret_cast<std::uintptr_t>(&node);
  std::cout<<label<<' '<<sizeof(N)<<' '<<alignof(N)<<' '
    <<(reinterpret_cast<std::uintptr_t>(&node.Key())-start)<<' '
    <<sizeof(NCollection_TListNode<K>)<<'\n';
}
int main() {
  data_layout<int,int>("data_int_int");data_layout<char,int>("data_char_int");
  data_layout<int,char>("data_int_char");data_layout<char,char>("data_char_char");
  data_layout<double,int>("data_double_int");data_layout<int,double>("data_int_double");
  data_layout<long long,char>("data_long_char");data_layout<char,long long>("data_char_long");
  map_layout<int>("map_int");map_layout<char>("map_char");map_layout<double>("map_double");
}
