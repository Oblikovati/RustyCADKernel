#include <Standard_Character.hxx>

#include <clocale>
#include <cstdio>

int main() {
  if (std::setlocale(LC_CTYPE, "de_DE.ISO-8859-1") == nullptr) {
    std::fputs("unable to activate de_DE.ISO-8859-1\n", stderr);
    return 2;
  }
  const bool checks[] = {
      IsAlphabetic(static_cast<char>(0xe4)),
      IsAlphanumeric(static_cast<char>(0xe4)),
      IsLowerCase(static_cast<char>(0xdf)),
      static_cast<unsigned char>(UpperCase(static_cast<char>(0xe4))) == 0xc4,
      IsPrintable(static_cast<char>(0xa0)),
  };
  for (bool check : checks) {
    std::printf("%d\n", check ? 1 : 0);
    if (!check) return 1;
  }
  return 0;
}
