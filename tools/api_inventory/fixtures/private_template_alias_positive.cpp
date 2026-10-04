#include "include/Fixture.hxx"

int main() {
  fixture::AliasExposureRoot::EnabledAlias enabled;
  enabled.common_member();
  enabled.enabled_member();
  fixture::AliasExposureRoot::DisabledAlias disabled;
  disabled.common_member();
  fixture::AliasExposureDerived derived;
  derived.call_protected_member();
  fixture::AliasDefaultPublicRoot::Alias default_public;
  default_public.implicit_public_member();
  return 0;
}
