// Captures the first-multibyte-length allocation quirk in NCollection_UtfString.cxx.
#include <NCollection_UtfString.hxx>
#include <clocale>
#include <cstdio>

static void inspect(const char* label, const char* input)
{
  NCollection_UtfString<char> text;
  text.FromLocale(input);
  unsigned char output[12];
  for (auto& byte : output) byte = 0xff;
  const bool success = text.ToLocale(reinterpret_cast<char*>(output), 12);
  std::printf("%s:%d:%d:", label, text.Length(), success);
  for (auto byte : output) std::printf("%02x", byte);
  std::puts("");
}

int main(int argc, char** argv)
{
  if (argc != 2 || !std::setlocale(LC_CTYPE, argv[1])) return 2;
  inspect("ascii", "abc");
  inspect("two", "\xc3\xa4" "bc");
  inspect("three", "\xe2\x82\xac" "bcd");
  inspect("four", "\xf0\x9f\x98\x80" "abcd");
  inspect("latin", "\xe4" "bc");
  inspect("empty", "");
}
