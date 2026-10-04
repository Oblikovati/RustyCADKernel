#include <Standard_CString.hxx>
#include <climits>
#include <cstdarg>
#include <cstdio>
#include <cstring>
#include <iostream>
#include <string>
#include <unistd.h>
#include <vector>

static void record_bytes(const char* theName,
                        int theCount,
                        const char* theBuffer,
                        size_t theLength)
{
  std::cout << theName << ' ' << theCount << ' ';
  const unsigned char* aBytes = reinterpret_cast<const unsigned char*>(theBuffer);
  for (size_t anIndex = 0; anIndex < theLength; ++anIndex)
  {
    static const char* aHex = "0123456789abcdef";
    std::cout << aHex[aBytes[anIndex] >> 4] << aHex[aBytes[anIndex] & 15];
  }
  std::cout << '\n';
}

static void record(const char* theName, int theCount, const char* theBuffer)
{
  record_bytes(theName, theCount, theBuffer, static_cast<size_t>(theCount));
}

static int format_variadic(char* theBuffer, const char* theFormat, ...)
{
  va_list anArguments;
  va_start(anArguments, theFormat);
  const int aCount = Vsprintf(theBuffer, theFormat, anArguments);
  va_end(anArguments);
  return aCount;
}

static void format_buffer_cases(char* theBuffer)
{
  const int anIntegerCount = Sprintf(theBuffer, "[%+06d|%#o|%#08X]", -42, 8U, 42U);
  record("sprintf_integer", anIntegerCount, theBuffer);
  const int aFloatCount = Sprintf(theBuffer, "float=%+08.2f;exp=%.3e;gen=%.4g", -1.25, 0.0000125, 12345.67);
  record("sprintf_float", aFloatCount, theBuffer);
  const int aTextCount = Sprintf(theBuffer, "%8.3s|%-5s|%%", "kernel", "CAD");
  record("sprintf_text", aTextCount, theBuffer);
  const int aCTextCount = Sprintf(theBuffer, "%s", "left\0ignored");
  record("sprintf_c_text", aCTextCount, theBuffer);
  const int anUnsignedCount = Sprintf(theBuffer, "%u|%#x", 4294967295U, 0xdeadbeefU);
  record("sprintf_unsigned", anUnsignedCount, theBuffer);
  const int anUppercaseCount = Sprintf(theBuffer, "%i|%.1F|%.2E|%.4G", -12, 1.25, 0.0000125, 12345.67);
  record("sprintf_uppercase", anUppercaseCount, theBuffer);
  const int aVariadicCount = format_variadic(theBuffer, "%04d:%s", 7, "edge");
  record("vsprintf", aVariadicCount, theBuffer);
}

static void capture_printf(char* theBuffer)
{
  FILE* aStdoutCapture = tmpfile();
  fflush(stdout);
  const int aSavedStdout = dup(STDOUT_FILENO);
  dup2(fileno(aStdoutCapture), STDOUT_FILENO);
  const int aPrintfCount = Printf("printf:%04d\n", 7);
  fflush(stdout);
  dup2(aSavedStdout, STDOUT_FILENO);
  close(aSavedStdout);
  std::rewind(aStdoutCapture);
  const size_t aPrintfBytes = fread(theBuffer, 1, 256, aStdoutCapture);
  theBuffer[aPrintfBytes] = '\0';
  record("printf", aPrintfCount, theBuffer);
  fclose(aStdoutCapture);
}

static void capture_fprintf(char* theBuffer)
{
  FILE* aFile = tmpfile();
  const int aFprintfCount = Fprintf(aFile, "fprintf:%#x\n", 42U);
  fflush(aFile);
  std::rewind(aFile);
  const size_t aRead = fread(theBuffer, 1, 256, aFile);
  theBuffer[aRead] = '\0';
  record("fprintf", aFprintfCount, theBuffer);
  fclose(aFile);
}

static void format_writer_cases(char* theBuffer)
{
  capture_printf(theBuffer);
  capture_fprintf(theBuffer);
}

static void format_star_char_cases(char* theBuffer)
{
  const int aStarCount = Sprintf(theBuffer, "%*.*f|%*d|%.*s", -8, 2, 1.25, 5, -7, -2, "native");
  record("sprintf_star", aStarCount, theBuffer);
  const int aCharCount = Sprintf(theBuffer, "%cX%3c|%-4c", 0, 65, -1);
  record("sprintf_char", aCharCount, theBuffer);
}

static void format_fixed_buffer_cases()
{
  char aSprintfBuffer[16];
  std::memset(aSprintfBuffer, 0xa5, sizeof(aSprintfBuffer));
  const int aSprintfCount = Sprintf(aSprintfBuffer, "%c:%04d", 0, 7);
  record_bytes("sprintf_slice_tail", aSprintfCount, aSprintfBuffer, sizeof(aSprintfBuffer));
  char aVsprintfBuffer[16];
  std::memset(aVsprintfBuffer, 0xa5, sizeof(aVsprintfBuffer));
  const int aVsprintfCount = format_variadic(aVsprintfBuffer, "%c:%04d", 0, 7);
  record_bytes("vsprintf_slice_tail", aVsprintfCount, aVsprintfBuffer, sizeof(aVsprintfBuffer));
}

static void format_long_integer_cases(char* theBuffer)
{
  const int aCount = Sprintf(theBuffer,
                             "%ld|%ld|%lu|%#018lx",
                             LONG_MIN,
                             LONG_MAX,
                             ULONG_MAX,
                             0xdeadbeefUL);
  record("sprintf_long_lengths", aCount, theBuffer);
}

static void format_long_long_cases(char* theBuffer)
{
  const int aCount = Sprintf(theBuffer, "%lld|%lld|%.20llu|%#020llX",
                             LLONG_MIN, LLONG_MAX, ULLONG_MAX, 42ULL);
  record("sprintf_long_long_lengths", aCount, theBuffer);
}

int main()
{
  char aBuffer[256] = {};
  format_buffer_cases(aBuffer);
  format_star_char_cases(aBuffer);
  format_fixed_buffer_cases();
  format_long_integer_cases(aBuffer);
  format_long_long_cases(aBuffer);
  format_writer_cases(aBuffer);
  return 0;
}
