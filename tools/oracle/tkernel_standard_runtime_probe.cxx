// Differential cases for Standard handles, transients, type descriptors, failures, error handler and TColStd handles.
//
// The prebuilt TKernel is compiled with OCC_CONVERT_SIGNALS (adm/cmake/occt_defs_flags.cmake on
// Linux), which selects the exported Standard_ErrorHandler::Callback members; the probe must see
// the same declarations, so it defines the macro before the header is first included.
#if defined(_Standard_ErrorHandler_HeaderFile) && !defined(OCC_CONVERT_SIGNALS)
  #error "Standard_ErrorHandler.hxx was included without OCC_CONVERT_SIGNALS before this probe"
#endif
#ifndef OCC_CONVERT_SIGNALS
  #define OCC_CONVERT_SIGNALS
#endif
#include "tkernel_api_common.hxx"
#include <NCollection_HeapAllocator.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Standard_Persistent.hxx>
#include <Standard_ProgramError.hxx>
#include <Standard_Type.hxx>
#include <TColStd_HPackedMapOfInteger.hxx>
#include <functional>
#include <vector>

namespace standard_runtime_probe
{
using TypeEntry = std::pair<std::string, occ::handle<Standard_Type>>;

// Registered TKernel classes the Rust port also registers, in response column order.
static const std::vector<TypeEntry>& types()
{
  static const std::vector<TypeEntry> THE_TYPES = {
    {"Standard_Transient", STANDARD_TYPE(Standard_Transient)},
    {"Standard_Type", STANDARD_TYPE(Standard_Type)},
    {"Standard_Persistent", STANDARD_TYPE(Standard_Persistent)},
    {"TColStd_HPackedMapOfInteger", STANDARD_TYPE(TColStd_HPackedMapOfInteger)},
    {"NCollection_BaseAllocator", STANDARD_TYPE(NCollection_BaseAllocator)},
    {"NCollection_IncAllocator", STANDARD_TYPE(NCollection_IncAllocator)},
    {"NCollection_HeapAllocator", STANDARD_TYPE(NCollection_HeapAllocator)}};
  return THE_TYPES;
}

static const occ::handle<Standard_Type>& type_named(const std::string& theName)
{
  for (const TypeEntry& anEntry : types())
    if (anEntry.first == theName)
      return anEntry.second;
  throw std::runtime_error("unknown probe type " + theName);
}

static char bit(bool theValue)
{
  return theValue ? '1' : '0';
}

// "ADDR : <name>" when Print wrote the descriptor's own address before " : ", else "BAD".
static std::string normalized_print(const occ::handle<Standard_Type>& theType)
{
  std::ostringstream aPrinted, anAddress;
  theType->Print(aPrinted);
  anAddress << static_cast<const void*>(theType.get());
  const std::string aText = aPrinted.str(), aPrefix = anAddress.str() + " : ";
  if (aText.compare(0, aPrefix.size(), aPrefix) != 0 || aPrefix.compare(0, 2, "0x") != 0)
    return "BAD";
  return "ADDR : " + aText.substr(aPrefix.size());
}

// "runtime_type <Name>": name, parent, Print, SubType by handle against every column, RTTI.
static std::string type_case(std::istream& theInput)
{
  std::string aName;
  theInput >> aName;
  const occ::handle<Standard_Type>& aType = type_named(aName);
  std::string aSubTypes;
  for (const TypeEntry& anEntry : types())
    aSubTypes += bit(aType->SubType(anEntry.second));
  const std::string aParent = aType->Parent().IsNull() ? "-" : aType->Parent()->Name();
  return std::string(aType->Name()) + ' ' + aParent + ' ' + tkernel_probe::hex(normalized_print(aType))
         + ' ' + aSubTypes + bit(aType->SubType(occ::handle<Standard_Type>())) + ' '
         + aType->DynamicType()->Name() + ' ' + bit(aType->IsKind(STANDARD_TYPE(Standard_Transient)));
}

// "runtime_subtype_name <Name> <hex name>": SubType(const char*), the name ending at a NUL.
static std::string subtype_name_case(std::istream& theInput)
{
  std::string aName;
  theInput >> aName;
  const std::string aQuery = tkernel_probe::next_bytes(theInput);
  return std::string(1, bit(type_named(aName)->SubType(aQuery.c_str())));
}

static occ::handle<Standard_Transient> make_object(const std::string& theClass)
{
  if (theClass == "Standard_Transient")
    return new Standard_Transient();
  if (theClass == "Standard_Persistent")
    return new Standard_Persistent();
  if (theClass == "TColStd_HPackedMapOfInteger")
    return new TColStd_HPackedMapOfInteger();
  if (theClass == "NCollection_IncAllocator")
    return new NCollection_IncAllocator();
  throw std::runtime_error("unknown probe class " + theClass);
}

// "runtime_object <Class> <hex name>": IsInstance/IsKind by name and by every column type,
// occ:: helpers, DynamicType, This() on the owned object and on an unowned one.
static std::string object_case(std::istream& theInput)
{
  std::string aClass;
  theInput >> aClass;
  const std::string                     aName   = tkernel_probe::next_bytes(theInput);
  const occ::handle<Standard_Transient> anObject = make_object(aClass);
  std::string aBits{bit(anObject->IsInstance(aName.c_str())), bit(anObject->IsKind(aName.c_str())), ' '};
  for (const TypeEntry& anEntry : types())
    aBits += std::string{bit(anObject->IsInstance(anEntry.second)), bit(anObject->IsKind(anEntry.second))};
  aBits += std::string{' ', bit(occ::is_kind<Standard_Persistent>(anObject)),
                       bit(occ::is_instance<Standard_Persistent>(anObject.get())),
                       bit(occ::is_kind<Standard_Transient>(occ::handle<Standard_Transient>()))};
  const Standard_Transient anUnowned;
  return aBits + ' ' + anObject->DynamicType()->Name() + ' ' + std::to_string(anObject->GetRefCount()) + ' '
         + bit(anObject->This() == anObject.get()) + ' '
         + tkernel_probe::guarded([&] { return std::to_string(anUnowned.This() != nullptr); });
}
} // namespace standard_runtime_probe

#include "tkernel_standard_runtime_handles_probe.cxx"
#include "tkernel_standard_runtime_failure_probe.cxx"
#include "tkernel_standard_runtime_handler_probe.cxx"

namespace standard_runtime_probe
{
static const tkernel_probe::Table cases = {{"runtime_type", type_case},
                                           {"runtime_subtype_name", subtype_name_case},
                                           {"runtime_object", object_case},
                                           {"runtime_handle_script", handle_script_case},
                                           {"runtime_hpacked", hpacked_case},
                                           {"runtime_failure", failure_case},
                                           {"runtime_failure_stack", failure_stack_case},
                                           {"runtime_out_of_memory", out_of_memory_case},
                                           {"runtime_stack_length", stack_length_case},
                                           {"runtime_stack_trace", stack_trace_case},
                                           {"runtime_handler_script", handler_script_case},
                                           {"runtime_abort", abort_case}};
} // namespace standard_runtime_probe
