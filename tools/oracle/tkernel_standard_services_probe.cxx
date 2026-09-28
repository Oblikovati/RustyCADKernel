// Differential cases for Standard memory managers, synchronization, locale, buffered readers, dump and Plugin.
#include "tkernel_api_common.hxx"
#include "tkernel_standard_services_memory.cxx"
#include "tkernel_standard_services_sync.cxx"
#include "tkernel_standard_services_locale.cxx"
#include "tkernel_standard_services_readers.cxx"
#include "tkernel_standard_services_array_stream.cxx"
#include "tkernel_standard_services_dump.cxx"

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
  {"services_array_stream", array_stream_case},
  {"services_dump_separator", dump_separator},
  {"services_dump_key_to_class", dump_key_to_class},
  {"services_dump_character_values", dump_character_values},
  {"services_dump_real_values", dump_real_values},
  {"services_dump_field_to_name", dump_field_to_name},
  {"services_dump_format", dump_format},
  {"services_dump_split", dump_split},
  {"services_dump_keys", dump_keys},
  {"services_dump_pointer", dump_pointer},
  {"services_dump_process", dump_process},
  {"services_dump_init_value", dump_init_value},
  {"services_dump_init_reals", dump_init_reals},
};
} // namespace standard_services_probe
