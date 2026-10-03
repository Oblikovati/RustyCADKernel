#include <Standard_CLocaleSentry.hxx>
#include <Standard_Character.hxx>

#include <clocale>
#include <cstdio>

int main() {
  if (std::setlocale(LC_CTYPE, "de_DE.ISO-8859-1") == nullptr) {
    std::fputs("unable to activate de_DE.ISO-8859-1\n", stderr);
    return 2;
  }
  const auto isLatin1 = [] { return IsAlphabetic(static_cast<char>(0xe4)); };
  const auto isC = [] { return !IsAlphabetic(static_cast<char>(0xe4)); };
  const bool before = isLatin1();
  bool nested = false;
  bool restored = false;
  {
    Standard_CLocaleSentry outer;
    const bool outerC = isC();
    {
      Standard_CLocaleSentry inner;
      nested = outerC && isC();
    }
    restored = nested && isC();
  }
  const bool after = isLatin1();
  const bool checks[] = {before, nested, restored, after};
  for (bool check : checks) {
    std::printf("%d\n", check ? 1 : 0);
    if (!check) return 1;
  }
  return 0;
}
