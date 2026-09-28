// OSD process probe operations (OSD_Process, OSD_Host, OSD_Environment, OSD_SharedLibrary,
// OSD_MemInfo); twin of oracle_probe/osd/process.rs. Both probes run on the same host, so
// host names, loader messages and the working directory agree; values that differ between
// two processes (pid, executable, memory sizes, the clock) are reduced to their shape.
//
// osd_env <name> <value> <value2>  Constructor, Build, Value, SetValue, Remove, fresh read.
// osd_env_name <name> <new>        OSD_Environment(Name) and SetName validation, Value.
// osd_host                         Every OSD_Host query and its error state.
// osd_process                      TerminalType, UserName, IsSuperUser, ProcessId>0, date.
// osd_chdir <dir>                  SetCurrentDirectory (restored), CurrentDirectory, errors.
// osd_shlib <name> <mode> <symbol> OSD_SharedLibrary open/symbol/error/close sequence.
// osd_meminfo <mask> <update>      MemInfo activity, unknown-counter values and ToString.
#include <OSD_Environment.hxx>
#include <OSD_Host.hxx>
#include <OSD_MemInfo.hxx>
#include <OSD_OSDError.hxx>
#include <OSD_Path.hxx>
#include <OSD_Process.hxx>
#include <OSD_SharedLibrary.hxx>
#include <Quantity_Date.hxx>
#include <Standard_ConstructionError.hxx>

template <typename T>
static void osd_process_error(std::ostream& out, T& theObject)
{
  out << ' ' << theObject.Failed();
  try
  {
    theObject.Perror();
    out << " ok";
  }
  catch (const Standard_Failure& theFailure)
  {
    out << ' ' << theFailure.ExceptionType() << osd_escape(theFailure.what());
  }
  out << ' ' << theObject.Error();
  theObject.Reset();
}

static void osd_env_probe(std::istream& in, std::ostream& out)
{
  const std::string aName = osd_token(in), aValue = osd_token(in), aValue2 = osd_token(in);
  try
  {
    OSD_Environment anEnv(aName.c_str(), aValue.c_str());
    anEnv.Build();
    out << osd_escape(anEnv.Value().ToCString()) << ' ' << anEnv.Failed();
    try
    {
      anEnv.SetValue(aValue2.c_str());
      out << " set";
    }
    catch (const Standard_Failure& theFailure)
    {
      out << ' ' << theFailure.ExceptionType() << osd_escape(theFailure.what());
    }
    anEnv.Build();
    out << ' ' << osd_escape(anEnv.Value().ToCString());
    anEnv.Remove();
    out << ' ' << osd_escape(anEnv.Value().ToCString()) << ' ' << anEnv.Failed();
    OSD_Environment aFresh(aName.c_str());
    out << ' ' << osd_escape(aFresh.Value().ToCString());
  }
  catch (const Standard_Failure& theFailure)
  {
    out << theFailure.ExceptionType() << osd_escape(theFailure.what());
  }
}

static void osd_env_name_probe(std::istream& in, std::ostream& out)
{
  const std::string aName = osd_token(in), aNew = osd_token(in);
  try
  {
    OSD_Environment anEnv(aName.c_str());
    out << osd_escape(anEnv.Name().ToCString()) << ' ' << osd_escape(anEnv.Value().ToCString());
    anEnv.SetName(aNew.c_str());
    out << ' ' << osd_escape(anEnv.Name().ToCString()) << ' '
        << osd_escape(anEnv.Value().ToCString());
  }
  catch (const Standard_Failure& theFailure)
  {
    out << ' ' << theFailure.ExceptionType() << osd_escape(theFailure.what());
  }
}

static void osd_host_probe(std::ostream& out)
{
  OSD_Host aHost;
  out << osd_escape(aHost.SystemVersion().ToCString()) << ' ' << aHost.SystemId() << ' '
      << osd_escape(aHost.HostName().ToCString()) << ' ' << aHost.MachineType() << ' '
      << aHost.AvailableMemory() << ' ' << osd_escape(aHost.InternetAddress().ToCString());
  osd_process_error(out, aHost);
}

static void osd_process_names_probe(std::ostream& out)
{
  OSD_Process aProcess;
  TCollection_AsciiString aTerm;
  aProcess.TerminalType(aTerm);
  Quantity_Date           aDate = aProcess.SystemDate();
  out << osd_escape(aTerm.ToCString()) << ' ' << osd_escape(aProcess.UserName().ToCString())
      << ' ' << aProcess.IsSuperUser() << ' ' << (aProcess.ProcessId() > 0) << ' '
      << (aDate.Year() >= 2020);
  osd_process_error(out, aProcess);
}

static void osd_chdir_probe(std::istream& in, std::ostream& out)
{
  const std::string aTarget = osd_token(in);
  OSD_Process       aProcess;
  const OSD_Path    aBack = aProcess.CurrentDirectory();
  aProcess.SetCurrentDirectory(OSD_Path(aTarget.c_str()));
  TCollection_AsciiString aHere;
  aProcess.CurrentDirectory().SystemName(aHere);
  out << osd_escape(aHere.ToCString()) << ' ' << osd_escape(aProcess.CurrentDirectory().Trek().ToCString());
  osd_process_error(out, aProcess);
  aProcess.SetCurrentDirectory(aBack);
  out << ' ' << aProcess.Failed();
}

static void osd_shlib_probe(std::istream& in, std::ostream& out)
{
  const std::string aName = osd_token(in);
  int               aMode = 0;
  in >> aMode;
  const std::string aSymbol = osd_token(in);
  OSD_SharedLibrary aLib;
  if (!aName.empty())
  {
    aLib.SetName(aName.c_str());
  }
  // Drop any loader error left by the runtime (the Rust standard library resolves weak
  // symbols through dlsym), so both probes start from a clean dlerror state.
  const char* anError = aLib.DlError();
  out << (aLib.Name() == nullptr ? "(null)" : osd_escape(aLib.Name()));
  const bool isOpen = aLib.DlOpen(aMode == 0 ? OSD_RTLD_LAZY : OSD_RTLD_NOW);
  anError           = aLib.DlError();
  out << ' ' << isOpen << ' ' << (anError == nullptr ? "(none)" : osd_escape(anError));
  if (!isOpen)
  {
    return;
  }
  const OSD_Function aFunc = aLib.DlSymb(aSymbol.c_str());
  anError                  = aLib.DlError();
  out << ' ' << (aFunc != nullptr) << ' ' << (anError == nullptr ? "(none)" : osd_escape(anError));
  aLib.DlClose();
  aLib.Destroy();
  out << ' ' << (aLib.Name() == nullptr);
}

static void osd_meminfo_probe(std::istream& in, std::ostream& out)
{
  int aMask = 0, anUpdate = 0;
  in >> aMask >> anUpdate;
  OSD_MemInfo anInfo(false);
  out << osd_escape(anInfo.ToString().ToCString());
  for (int aCounter = 0; aCounter < OSD_MemInfo::MemCounter_NB; ++aCounter)
  {
    anInfo.SetActive((OSD_MemInfo::Counter)aCounter, (aMask >> aCounter) & 1);
  }
  if (anUpdate != 0)
  {
    anInfo.Update();
  }
  for (int aCounter = 0; aCounter < OSD_MemInfo::MemCounter_NB; ++aCounter)
  {
    const OSD_MemInfo::Counter aKind = (OSD_MemInfo::Counter)aCounter;
    const size_t               aValue = anInfo.Value(aKind);
    out << ' ' << anInfo.IsActive(aKind) << (aValue == size_t(-1) ? 'u' : 'k')
        << (anInfo.ValueMiB(aKind) == size_t(-1) ? 'u' : 'k')
        << (anInfo.ValuePreciseMiB(aKind) == -1.0 ? 'u' : 'k');
  }
  if (anUpdate == 0)
  {
    out << ' ' << osd_escape(anInfo.ToString().ToCString());
  }
  anInfo.SetActive(false);
  out << ' ' << anInfo.IsActive(OSD_MemInfo::MemHeapUsage) << ' '
      << osd_escape(anInfo.ToString().ToCString());
  anInfo.Clear();
}

static bool osd_process_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "osd_env")
    osd_env_probe(in, out);
  else if (op == "osd_env_name")
    osd_env_name_probe(in, out);
  else if (op == "osd_host")
    osd_host_probe(out);
  else if (op == "osd_process")
    osd_process_names_probe(out);
  else if (op == "osd_chdir")
    osd_chdir_probe(in, out);
  else if (op == "osd_shlib")
    osd_shlib_probe(in, out);
  else if (op == "osd_meminfo")
    osd_meminfo_probe(in, out);
  else
    return false;
  return true;
}
