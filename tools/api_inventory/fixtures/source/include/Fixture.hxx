#ifndef _Fixture_HeaderFile
#define _Fixture_HeaderFile

#define FIXTURE_DECLARE(name) int name();
#define FIXTURE_SCALE(value) ((value) * 2)

namespace fixture {
using Scalar = double;
typedef unsigned long Count;

struct PublicType {
  using Value = Scalar;
  int field;
  void overload(int value);
  void overload(Scalar value);
  FIXTURE_DECLARE(from_macro)

protected:
  struct ProtectedNested {};
  int protected_field;

private:
  int private_field;
};

class PrivateOuter {
private:
  struct SecretOuter {
  public:
    struct HiddenNested {};
  };
public:
  struct HiddenNested {};
};

template <typename T, unsigned N = 2> struct Box {
  T value;
  T scale(T input) const;
};

enum class Direction { Left, Right };
inline constexpr int scaled = FIXTURE_SCALE(3);
static_assert(sizeof(Count) > 0);
}

#if 0
namespace fixture { struct InactivePlatformType {}; }
#endif

#include "Fixture.lxx"
#include "Fixture.pxx"
#include "Fixture.gxx"
#endif
