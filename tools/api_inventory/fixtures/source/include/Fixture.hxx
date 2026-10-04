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

struct TemplateAccessRoot {
public:
  template <typename T> struct PublicNestedTemplate {
    void public_member();
    template <typename U> U public_member_template(U value);
  protected:
    void protected_member();
    template <typename U> U protected_member_template(U value);
  private:
    void private_member();
    template <typename U> U private_member_template(U value);
  };
protected:
  template <typename T> struct ProtectedNestedTemplate {
    void public_member();
    template <typename U> U public_member_template(U value);
  };
private:
  template <typename T> struct PrivateNestedTemplate {
  public:
    void public_member();
  };
};

class DefaultAccessRoot {
  template <typename T> struct PrivateByDefaultTemplate {
  public:
    void public_member();
  };
public:
  template <typename T> struct PublicNestedTemplate {
    void public_member();
  };
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
template <typename... Elements> struct Pack {};
template <template <typename> class Container> struct TemplateTemplateBox {};
template <typename Element, typename = typename Element::value_type>
struct AssociatedTypeBox {};
template <typename Element> using ElementAlias = Box<Element>;

template <typename T> T identity(T value);
template <typename Defaulted = int> void default_template_parameter(Defaulted value);
template <typename Renamed> void default_template_parameter(Renamed other) {}

using CountAlias = unsigned long;
void redeclared_value(const CountAlias original_count);
void redeclared_value(unsigned long renamed_count) {}
void redeclared_default(const int original_value = 7);
void redeclared_default(const int renamed_value) {}

struct BaseDefaults { void inherited_default(int value = 11); };
struct DerivedDefaults : BaseDefaults {
  using BaseDefaults::inherited_default;
};

template <typename FirstProvider>
auto named_template(FirstProvider first) -> decltype(first.Size());
template <typename SecondProvider>
auto named_template(SecondProvider second) -> decltype(second.Size()) {
  return second.Size();
}

template <class FirstProvider>
auto class_template_parameter(FirstProvider first) -> decltype(first.Size());
template <typename SecondProvider>
auto class_template_parameter(SecondProvider second) -> decltype(second.Size()) {
  return second.Size();
}

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
