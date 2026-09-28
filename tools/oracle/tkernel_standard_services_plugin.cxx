// Plugin::Load failure paths: a temporary "Plugin" resource file names the library.
// Included by tkernel_standard_services_probe.cxx before its case table.
#include <Plugin.hxx>
#include <Standard_GUID.hxx>
#include <Standard_Transient.hxx>
#include <cstdlib>
#include <fstream>
#include <unistd.h>

namespace standard_services_probe
{
// "services_plugin <guid> <hex resource file> <verbose>": writes the file into a fresh
// directory named by CSF_PluginDefaults, loads the GUID and reports the result and what
// was printed to std::cout.
static std::string plugin_case(std::istream& theInput)
{
  std::string aGuid;
  int         isVerbose = 0;
  theInput >> aGuid;
  const std::string aFile = tkernel_probe::next_bytes(theInput);
  theInput >> isVerbose;
  char aDirectory[] = "/tmp/tkernel_plugin_probe_XXXXXX";
  if (mkdtemp(aDirectory) == nullptr)
    return "mkdtemp failed";
  const std::string aPath = std::string(aDirectory) + "/Plugin";
  std::ofstream(aPath, std::ios::binary) << aFile;
  setenv("CSF_PluginDefaults", aDirectory, 1);
  unsetenv("CSF_PluginUserDefaults");
  std::ostringstream aConsole;
  std::streambuf*    aStdout = std::cout.rdbuf(aConsole.rdbuf());
  const std::string  aResult = tkernel_probe::guarded([&] {
    const occ::handle<Standard_Transient> aProduct = Plugin::Load(Standard_GUID(aGuid.c_str()), isVerbose != 0);
    return std::string(aProduct.IsNull() ? "null" : "product");
  });
  std::cout.rdbuf(aStdout);
  unlink(aPath.c_str());
  rmdir(aDirectory);
  unsetenv("CSF_PluginDefaults");
  return tkernel_probe::hex(aResult) + " " + tkernel_probe::hex(aConsole.str());
}
} // namespace standard_services_probe
