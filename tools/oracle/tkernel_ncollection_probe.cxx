// NCollection differential operations, one included file per container family.
// Each family function answers the operations it owns and returns false for the rest;
// the Rust twin is crates/tkernel/examples/oracle_probe/ncollection/.
#include <istream>
#include <ostream>
#include <string>

#include "tkernel_ncollection_arrays_probe.cxx"
#include "tkernel_ncollection_lists_probe.cxx"
#include "tkernel_ncollection_hash_maps_probe.cxx"
#include "tkernel_ncollection_flat_maps_probe.cxx"
#include "tkernel_ncollection_allocators_probe.cxx"
#include "tkernel_ncollection_trees_probe.cxx"
#include "tkernel_ncollection_utilities_probe.cxx"

static bool ncollection_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  return ncollection_arrays_probe(op, in, out) ||
         ncollection_lists_probe(op, in, out) ||
         ncollection_hash_maps_probe(op, in, out) ||
         ncollection_flat_maps_probe(op, in, out) ||
         ncollection_allocators_probe(op, in, out) ||
         ncollection_trees_probe(op, in, out) ||
         ncollection_utilities_probe(op, in, out);
}
