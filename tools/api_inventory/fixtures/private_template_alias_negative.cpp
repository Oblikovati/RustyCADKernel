#include "include/Fixture.hxx"

using HiddenByName = fixture::AliasExposureRoot::Hidden<true>;

int main() { return sizeof(HiddenByName); }
