#ifndef _Fixture_HeaderFile
#define _Fixture_HeaderFile

#define FIXTURE_DECLARE(name) int name();
#define FIXTURE_SCALE(value) ((value) * 2)

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

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

template <typename T> T identity(T value);

struct ProviderValue { static constexpr int inherited_value = 1; };
template <typename T> struct DependentValueBase : T {
  using T::inherited_value;
};

struct SFINAEOverloads {
  template <typename Provider>
  auto IsValidIn(const Provider& provider) const
      -> decltype(provider.Nb(), bool()) { return true; }
  template <typename Provider>
  auto IsValidIn(const Provider& provider) const
      -> decltype(provider.Size(), bool()) { return true; }
};

inline void function_with_locals() {
  using LocalAlias = int;
  struct LocalRecord {};
  int local_value = 0;
  (void)sizeof(LocalAlias);
  (void)sizeof(LocalRecord);
  (void)local_value;
}

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
