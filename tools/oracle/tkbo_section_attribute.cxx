#include <BOPAlgo_SectionAttribute.hxx>
#include <BOPAlgo_PaveFiller.hxx>
#include <iostream>

int Mask(const BOPAlgo_SectionAttribute& flags) {
  return int(flags.Approximation()) | (int(flags.PCurveOnS1()) << 1)
       | (int(flags.PCurveOnS2()) << 2);
}

void Mutations(int mask, const BOPAlgo_SectionAttribute& initial) {
  for (int axis = 0; axis < 3; ++axis) {
    for (int enabled = 0; enabled < 2; ++enabled) {
      auto flags = initial;
      if (axis == 0) flags.Approximation(enabled != 0);
      if (axis == 1) flags.PCurveOnS1(enabled != 0);
      if (axis == 2) flags.PCurveOnS2(enabled != 0);
      std::cout << "set " << mask << ' ' << axis << ' ' << enabled << ' ' << Mask(flags) << '\n';
    }
  }
}

class Probe final : public BOPAlgo_PaveFiller {
public:
  int Flags() const { return Mask(mySectionAttribute); }
  void Reset() { Clear(); }
};

int main() {
  std::cout << "default " << Mask(BOPAlgo_SectionAttribute()) << '\n';
  for (int mask = 0; mask < 8; ++mask) {
    BOPAlgo_SectionAttribute flags((mask & 1) != 0, (mask & 2) != 0, (mask & 4) != 0);
    std::cout << "new " << mask << ' ' << Mask(flags) << '\n';
    Mutations(mask, flags);
  }
  Probe filler;
  std::cout << "filler-default " << filler.Flags() << '\n';
  BOPAlgo_SectionAttribute flags(true, false, true);
  filler.SetSectionAttribute(flags); flags.Approximation(false);
  std::cout << "filler-copy " << filler.Flags() << '\n';
  filler.Reset();
  std::cout << "filler-clear " << filler.Flags() << '\n';
}
