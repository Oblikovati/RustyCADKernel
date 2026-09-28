// Differential cases for Standard memory managers, synchronization, locale, buffered readers, dump and Plugin.
#include "tkernel_api_common.hxx"
#include "tkernel_standard_services_memory.cxx"

namespace standard_services_probe
{

static const tkernel_probe::Table cases = {
  {"services_standard_allocate", standard_allocate},
  {"services_standard_allocate_optimal", standard_allocate_optimal},
  {"services_standard_reallocate", standard_reallocate},
  {"services_standard_aligned", standard_aligned},
  {"services_standard_misc", standard_misc},
  {"services_mmgr", mmgr_script},
};
} // namespace standard_services_probe
