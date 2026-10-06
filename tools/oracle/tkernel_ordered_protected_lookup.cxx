// OCCT 3d097a0: protected output references retain native callback failure progress.
#include "ordered_protected_lookup/fixture.hxx"
int main(){runSuite<LookupSet>("set");runSuite<LookupMap>("data");}
