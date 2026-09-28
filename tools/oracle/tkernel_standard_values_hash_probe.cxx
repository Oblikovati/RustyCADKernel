// Standard_HashUtils, Standard_CString and Standard_CStringHasher cases of the
// standard_values probe (included by tkernel_standard_values_probe.cxx).
#include "tkernel_standard_values_scalar_probe.cxx"
#include <Standard_CString.hxx>
#include <Standard_CStringHasher.hxx>
#include <Standard_HashUtils.hxx>
#include <array>

namespace standard_values_hash_probe
{
using standard_values_scalar_probe::hex32;
using standard_values_scalar_probe::hex64;

// "hash_bytes <seed> <hex>": both Murmur and FNV engines, hashBytes and Standard_CStringHasher.
static std::string hash_bytes_case(std::istream& theInput)
{
  std::string aSeedText;
  theInput >> aSeedText;
  const std::uint64_t aSeed  = std::stoull(aSeedText, nullptr, 16);
  const std::string   aBytes = tkernel_probe::next_bytes(theInput);
  const int           aLen   = static_cast<int>(aBytes.size());
  const char*         aData  = aBytes.data();
  const auto          aSeed32 = static_cast<std::uint32_t>(aSeed);
  std::ostringstream  anOut;
  anOut << hex64(opencascade::MurmurHash::MurmurHash64A(aData, aLen, aSeed)) << ' '
        << hex32(opencascade::MurmurHash::MurmurHash2A(aData, aLen, aSeed32)) << ' '
        << hex32(opencascade::FNVHash::FNVHash1A(aData, aLen, aSeed32)) << ' '
        << hex64(opencascade::FNVHash::FNVHash64A(aData, aLen, aSeed)) << ' '
        << hex64(opencascade::hashBytes(aData, aLen)) << ' '
        << hex64(Standard_CStringHasher{}(aBytes.c_str())) << ' '
        << (Standard_CStringHasher{}(aBytes.c_str(), "abcd") ? '1' : '0');
  return anOut.str();
}

// "hash_combine <seed> <len> <hex64>": the templated hash_combine of both engines and the
// top-level one, over the first `len` bytes of a 64-byte object.
static std::string hash_combine_case(std::istream& theInput)
{
  std::string aSeedText;
  int         aLen = 0;
  theInput >> aSeedText >> aLen;
  const std::uint64_t          aSeed  = std::stoull(aSeedText, nullptr, 16);
  const std::string            aBytes = tkernel_probe::next_bytes(theInput);
  std::array<unsigned char, 64> aValue{};
  std::memcpy(aValue.data(), aBytes.data(), aValue.size());
  using Value         = std::array<unsigned char, 64>;
  const auto aSeed32 = static_cast<std::uint32_t>(aSeed);
  std::ostringstream anOut;
  anOut << hex64(opencascade::MurmurHash::hash_combine<Value, std::uint64_t>(aValue, aLen, aSeed))
        << ' ' << hex32(opencascade::MurmurHash::hash_combine<Value, std::uint32_t>(aValue, aLen, aSeed32))
        << ' ' << hex64(opencascade::FNVHash::hash_combine<Value, std::uint64_t>(aValue, aLen, aSeed))
        << ' ' << hex32(opencascade::FNVHash::hash_combine<Value, std::uint32_t>(aValue, aLen, aSeed32))
        << ' ' << hex64(opencascade::hash_combine<Value, size_t>(aValue, aLen, aSeed));
  return anOut.str();
}

// Every default-argument hash of one typed value read from its object bytes.
template <class T>
static std::string typed_hashes(const std::string& theBytes)
{
  T aValue;
  std::memcpy(&aValue, theBytes.data(), sizeof(T));
  std::ostringstream anOut;
  anOut << hex64(opencascade::hash(aValue)) << ' '
        << hex32(opencascade::hash<T, std::uint32_t>(aValue)) << ' '
        << hex64(opencascade::MurmurHash::hash_combine(aValue)) << ' '
        << hex32(opencascade::MurmurHash::hash_combine<T, std::uint32_t>(aValue)) << ' '
        << hex64(opencascade::FNVHash::hash_combine(aValue)) << ' '
        << hex32(opencascade::FNVHash::hash_combine<T, std::uint32_t>(aValue));
  return anOut.str();
}

// "hash_value <type> <hex>": opencascade::hash and the defaulted hash_combine overloads.
static std::string hash_value_case(std::istream& theInput)
{
  std::string aType;
  theInput >> aType;
  const std::string aBytes = tkernel_probe::next_bytes(theInput);
  if (aType == "char")
    return typed_hashes<char>(aBytes);
  if (aType == "uint16")
    return typed_hashes<std::uint16_t>(aBytes);
  if (aType == "int")
    return typed_hashes<int>(aBytes);
  if (aType == "int64")
    return typed_hashes<std::int64_t>(aBytes);
  return typed_hashes<double>(aBytes);
}

// "hash_seeds": optimalSeed of both engines for size_t, uint32_t and uint64_t.
static std::string hash_seeds_case(std::istream&)
{
  std::ostringstream anOut;
  anOut << hex64(opencascade::MurmurHash::optimalSeed()) << ' '
        << hex32(opencascade::MurmurHash::optimalSeed<std::uint32_t>()) << ' '
        << hex64(opencascade::MurmurHash::optimalSeed<std::uint64_t>()) << ' '
        << hex64(opencascade::FNVHash::optimalSeed()) << ' '
        << hex32(opencascade::FNVHash::optimalSeed<std::uint32_t>()) << ' '
        << hex64(opencascade::FNVHash::optimalSeed<std::uint64_t>());
  return anOut.str();
}

// "c_string_equal <hex> <hex>": IsEqual(const char*, const char*) on C strings.
static std::string c_string_equal_case(std::istream& theInput)
{
  const std::string aFirst  = tkernel_probe::next_bytes(theInput);
  const std::string aSecond = tkernel_probe::next_bytes(theInput);
  return IsEqual(aFirst.c_str(), aSecond.c_str()) ? "1" : "0";
}

// "atof <hex>": Atof on a C string.
static std::string atof_case(std::istream& theInput)
{
  const std::string aText = tkernel_probe::next_bytes(theInput);
  return hex64(tkernel_probe::bits(Atof(aText.c_str())));
}

static const tkernel_probe::Table cases = {{"standard_values_hash_bytes", hash_bytes_case},
                                           {"standard_values_hash_combine", hash_combine_case},
                                           {"standard_values_hash_value", hash_value_case},
                                           {"standard_values_hash_seeds", hash_seeds_case},
                                           {"standard_values_c_string_equal", c_string_equal_case},
                                           {"standard_values_atof", atof_case}};
} // namespace standard_values_hash_probe
