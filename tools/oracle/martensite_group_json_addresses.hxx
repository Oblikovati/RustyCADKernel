// Martensite local diagnostic normalization removes addresses and preserves every other byte.
#ifndef MARTENSITE_GROUP_JSON_ADDRESSES
#define MARTENSITE_GROUP_JSON_ADDRESSES
#include <regex>
static std::string GroupJsonPointers(const std::string& text) {
  // Pointer addresses vary by process; no numeric, NaN, field-order or failure fragment is normalized.
  return std::regex_replace(text,std::regex("0xx[0-9a-fA-F]+"),"@pointer");
}
#endif
