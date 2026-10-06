#include <NCollection_DataMap.hxx>
#include <cstdint>
#include <iostream>

enum class SignedKey : int { Negative = -7, MinusOne = -1, Positive = 9 };
enum class WideKey : uint64_t { Zero = 0, High = 0x8000000000000021ull, Max = UINT64_MAX };
enum PlainKey { PlainNegative = -3, PlainPositive = 5 };

// The native enum branch bypasses both standard-library customization points.
namespace std {
template <> struct hash<SignedKey> {
  size_t operator()(SignedKey) const { return 17; }
};
template <> struct equal_to<SignedKey> {
  bool operator()(SignedKey, SignedKey) const { return false; }
};
}

template <class Key>
static void PrintKey(const char* label, Key key)
{
  NCollection_DefaultHasher<Key> policy;
  std::cout << label << ' ' << policy(key) << ' ' << policy(key, key) << '\n';
}

static void PrintEnumHashes()
{
  PrintKey("signed_negative", SignedKey::Negative);
  PrintKey("signed_minus_one", SignedKey::MinusOne);
  PrintKey("signed_positive", SignedKey::Positive);
  PrintKey("wide_zero", WideKey::Zero);
  PrintKey("wide_high", WideKey::High);
  PrintKey("wide_max", WideKey::Max);
  PrintKey("plain_negative", PlainNegative);
  PrintKey("plain_positive", PlainPositive);
  std::cout << "bypassed " << std::hash<SignedKey>{}(SignedKey::Negative) << ' '
            << std::equal_to<SignedKey>{}(SignedKey::Negative, SignedKey::Negative) << '\n';
}

static void PrintEnumMaps()
{
  NCollection_DataMap<SignedKey, int> signedMap;
  std::cout << "signed_map " << signedMap.Bind(SignedKey::Negative, 1) << ' '
            << signedMap.Bind(SignedKey::Negative, 2) << ' '
            << signedMap.Find(SignedKey::Negative) << ' ' << signedMap.Size() << '\n';
  NCollection_DataMap<WideKey, int> wideMap;
  wideMap.Bind(WideKey::Zero, 1);
  wideMap.Bind(WideKey::High, 2);
  wideMap.Bind(WideKey::Max, 3);
  wideMap.ReSize(101);
  std::cout << "wide_map " << wideMap.Size() << ' ' << wideMap.Find(WideKey::Zero)
            << ' ' << wideMap.Find(WideKey::High) << ' ' << wideMap.Find(WideKey::Max) << '\n';
}

int main()
{
  PrintEnumHashes();
  PrintEnumMaps();
}
