#pragma once

namespace returned_fixture {
class Owner {
  struct Leaf { int leaf() const { return 9; } };
  class Hidden {
    int secret() const { return 99; }
  protected:
    int protectedValue() const { return 7; }
  public:
    int value = 5;
    int read() const { return value; }
    Leaf child() const { return {}; }
    Hidden *cycle() { return this; }
    using PublicAlias = Leaf;
  };
  struct FinalHidden final {
  protected:
    int blocked() const { return 3; }
  public:
    int read() const { return 5; }
  };
  template<class T> struct TemplateHidden { T item; };
  Hidden object;
  Hidden privateGetter() const { return {}; }
public:
  using Alias = Hidden;
  Hidden byValue() const { return {}; }
  Hidden *byPointer() { return &object; }
  Hidden &byReference() { return object; }
  const Hidden &byConstReference() const { return object; }
  Alias byAlias() const { return {}; }
  FinalHidden finalValue() const { return {}; }
  TemplateHidden<int> concreteTemplate() const { return {}; }
  template<class T> TemplateHidden<T> dependentTemplate() const { return {}; }
  template<class T> T unknownTemplate() const;
protected:
  Hidden protectedGetter() const { return {}; }
};
class DerivedOwner : public Owner {
public:
  using Owner::protectedGetter;
};
} // namespace returned_fixture
