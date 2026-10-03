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

static void inspectTool(const char* label, const char* input)
{
  NCollection_UtfStringTool aTool;
  wchar_t*                  aWideString = aTool.FromLocale(input);
  unsigned char             anOutput[12];
  if (aWideString != nullptr)
  {
    aWideString[0] = L'x';
  }
  for (auto& aByte : anOutput) aByte = 0xff;
  const bool isDone = aWideString != nullptr
                      && NCollection_UtfStringTool::ToLocale(
                        aWideString, reinterpret_cast<char*>(anOutput), 12);
  std::printf("%s:%d:", label, isDone);
  for (auto aByte : anOutput) std::printf("%02x", aByte);
  std::puts("");
  aWideString = aTool.FromLocale("z");
  for (auto& aByte : anOutput) aByte = 0xff;
  const bool isReplaced = NCollection_UtfStringTool::ToLocale(
    aWideString, reinterpret_cast<char*>(anOutput), 12);
  std::printf("%s_replaced:%d:", label, isReplaced);
  for (auto aByte : anOutput) std::printf("%02x", aByte);
  std::puts("");
}

static void inspectToolOutputEdges()
{
  const wchar_t aUmlaut[] = {L'\u00e4', L'\0'};
  const wchar_t anEmpty[] = {L'\0'};
  unsigned char anOutput[4];
  for (auto& aByte : anOutput) aByte = 0xff;
  const bool isShort = NCollection_UtfStringTool::ToLocale(
    aUmlaut, reinterpret_cast<char*>(anOutput), 1);
  std::printf("tool_short:%d:", isShort);
  for (auto aByte : anOutput) std::printf("%02x", aByte);
  std::puts("");

  for (auto& aByte : anOutput) aByte = 0xff;
  const bool isEmpty = NCollection_UtfStringTool::ToLocale(
    anEmpty, reinterpret_cast<char*>(anOutput), 4);
  std::printf("tool_empty_output:%d:", isEmpty);
  for (auto aByte : anOutput) std::printf("%02x", aByte);
  std::puts("");

  std::setlocale(LC_CTYPE, "C");
  for (auto& aByte : anOutput) aByte = 0xff;
  const bool isUnrepresentable = NCollection_UtfStringTool::ToLocale(
    aUmlaut, reinterpret_cast<char*>(anOutput), 4);
  std::printf("tool_error:%d:", isUnrepresentable);
  for (auto aByte : anOutput) std::printf("%02x", aByte);
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
  inspectTool("tool_utf8", "\xc3\xa4" "bc");
  inspectTool("tool_empty", "");
  inspectToolOutputEdges();
}
