// OSD differential operations, one included file per class family.
// Each family function answers the operations it owns and returns false for the rest;
// the Rust twin is crates/tkernel/examples/oracle_probe/osd/.
#include <istream>
#include <ostream>
#include <string>

// Shared by the families: request tokens use "-" for an empty string, and text is
// printed on one line with backslash, newline, carriage return and tab escaped.
static std::string osd_token(std::istream& in)
{
  std::string aToken;
  in >> aToken;
  return aToken == "-" ? std::string() : aToken;
}

static std::string osd_escape(const std::string& theText)
{
  std::string anOut;
  for (char aChar : theText)
  {
    switch (aChar)
    {
      case '\\': anOut += "\\\\"; break;
      case '\n': anOut += "\\n"; break;
      case '\r': anOut += "\\r"; break;
      case '\t': anOut += "\\t"; break;
      default: anOut += aChar;
    }
  }
  return "[" + anOut + "]";
}

#include "tkernel_osd_foundation_probe.cxx"
#include "tkernel_osd_path_probe.cxx"
#include "tkernel_osd_files_probe.cxx"
#include "tkernel_osd_file_system_probe.cxx"
#include "tkernel_osd_process_probe.cxx"
#include "tkernel_osd_timing_probe.cxx"
#include "tkernel_osd_threads_probe.cxx"

static bool osd_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  return osd_foundation_probe(op, in, out) || osd_path_probe(op, in, out)
         || osd_files_probe(op, in, out) || osd_file_system_probe(op, in, out)
         || osd_process_probe(op, in, out) || osd_timing_probe(op, in, out)
         || osd_threads_probe(op, in, out);
}
