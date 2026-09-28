// Units, UnitsAPI and UnitsMethods differential operations (twin of oracle_probe/units/).
//
// The probe process is long-lived: the Units package statics (dictionary, lexicons,
// ToSI/FromSI cache, FirstQuantity and Units_Dimensions::Quantity caches), the UnitsAPI
// statics and the UnitsMethods cascade unit evolve across requests exactly as the Rust
// twin's single contexts do. Text the kernel writes to std::cout and the payloads of the
// default messenger are captured per request and printed hex-encoded; the address in
// "Units_Token::Dump of <address>" is replaced by "@".
//
// Every string argument is hex-encoded ("-" is the empty string); reals are printed as
// their IEEE-754 bit patterns; Standard_Failure becomes "EXC <type> <hex message>".
#include "tkernel_units_probe_common.cxx"
#include "tkernel_units_probe_core.cxx"
#include "tkernel_units_probe_objects.cxx"
#include "tkernel_units_probe_api.cxx"

static bool units_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  return units_core_probe(op, in, out) || units_objects_probe(op, in, out)
         || units_api_probe(op, in, out);
}
