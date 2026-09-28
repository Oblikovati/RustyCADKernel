// TKernel API differential oracle: one request line in, one response line out.
// Each included file owns the cases of one area and exposes them as `<area>::cases`.
#include "tkernel_api_common.hxx"
#include "tkernel_exceptions_probe.cxx"
#include "tkernel_ascii_string_api_probe.cxx"
#include "tkernel_extended_string_api_probe.cxx"
#include "tkernel_standard_values_probe.cxx"
#include "tkernel_standard_runtime_probe.cxx"
#include "tkernel_standard_services_probe.cxx"

// Merges the per-area tables; a duplicated operation name is a probe bug.
static bool collect_cases(tkernel_probe::Table& theCases)
{
  for (const tkernel_probe::Table* aTable : {&exceptions_probe::cases,
                                             &ascii_string_api_probe::cases,
                                             &extended_string_api_probe::cases,
                                             &standard_values_probe::cases,
                                             &standard_runtime_probe::cases,
                                             &standard_services_probe::cases})
  {
    for (const auto& aCase : *aTable)
    {
      if (!theCases.insert(aCase).second)
      {
        std::cerr << "Duplicate probe operation: " << aCase.first << std::endl;
        return false;
      }
    }
  }
  return true;
}

int main()
{
  tkernel_probe::Table aCases;
  if (!collect_cases(aCases))
    return 2;
  std::string aLine;
  while (std::getline(std::cin, aLine))
  {
    std::istringstream anInput(aLine);
    std::string        anOperation;
    anInput >> anOperation;
    const auto aFound = aCases.find(anOperation);
    if (aFound == aCases.end())
    {
      std::cerr << "Unknown probe operation: " << anOperation << std::endl;
      return 2;
    }
    std::cout << aFound->second(anInput) << '\n';
  }
  return 0;
}
