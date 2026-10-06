// This advertised arrow overload cannot address an ordinary by-value result.
#include "tkernel_base_map_indexed_proxy_items.hxx"
int main()
{
  IndexedProxyMap map;
#ifdef TEST_MUTABLE
  IndexedProxyWrite cursor(map);
#else
  IndexedProxyRead cursor(map);
#endif
  cursor.operator->();
}
