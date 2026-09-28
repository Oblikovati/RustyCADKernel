// Differential cases for Standard memory managers, synchronization, locale, buffered readers, dump and Plugin.
#include "tkernel_api_common.hxx"
#include "tkernel_standard_services_memory.cxx"
#include "tkernel_standard_services_sync.cxx"
#include "tkernel_standard_services_locale.cxx"
#include "tkernel_standard_services_readers.cxx"

namespace standard_services_probe
{

static const tkernel_probe::Table cases = {
  {"services_standard_allocate", standard_allocate},
  {"services_standard_allocate_optimal", standard_allocate_optimal},
  {"services_standard_reallocate", standard_reallocate},
  {"services_standard_aligned", standard_aligned},
  {"services_standard_misc", standard_misc},
  {"services_mmgr", mmgr_script},
  {"services_mutex", mutex_script},
  {"services_condition", condition_script},
  {"services_clocale", clocale_case},
  {"services_read_buffer", read_buffer_case},
  {"services_read_line", read_line_case},
};
} // namespace standard_services_probe
