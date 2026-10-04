#include <Standard_MMgrOpt.hxx>

#include <cstring>
#include <iostream>

class ExposedMMgrOpt : public Standard_MMgrOpt
{
public:
  using Standard_MMgrOpt::Standard_MMgrOpt;
  void ReleasePools() { FreePools(); }
};

int main()
{
  ExposedMMgrOpt aManager(true, false, 16, 100, 40000);
  void* aSmall = aManager.Allocate(16);
  void* aMedium = aManager.Allocate(300);
  std::memcpy(aMedium, "keep", 4);
  aManager.ReleasePools();
  std::cout << "small=" << (aSmall != nullptr) << " medium="
            << (std::memcmp(aMedium, "keep", 4) == 0) << " released=1\n";
  aManager.Free(aMedium);
}
