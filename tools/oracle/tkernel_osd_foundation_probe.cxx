// OSD foundation probe operations (enumerations, OSD_Error, OSD_Protection, exception
// classes); twin of oracle_probe/osd/foundation.rs.
//
// osd_prot_row <a>             Add and Sub of protection <a> with each of the 16 rights.
// osd_prot_default             The default OSD_Protection values.
// osd_prot_set <s> <u> <g> <w> Constructor, Values, the four setters and getters.
// osd_error <errno> <from> <m> SetValue, Failed, Perror (raised message), Error, Reset.
// osd_exception <index>        Raises the index-th OSD exception class, caught as Failure.
// osd_enums                    Enumerator counts, from the last enumerator's value.
#include <OSD_Error.hxx>
#include <OSD_Exception.hxx>
#include <OSD_Exception_ACCESS_VIOLATION.hxx>
#include <OSD_Exception_ARRAY_BOUNDS_EXCEEDED.hxx>
#include <OSD_Exception_CTRL_BREAK.hxx>
#include <OSD_Exception_ILLEGAL_INSTRUCTION.hxx>
#include <OSD_Exception_IN_PAGE_ERROR.hxx>
#include <OSD_Exception_INT_OVERFLOW.hxx>
#include <OSD_Exception_INVALID_DISPOSITION.hxx>
#include <OSD_Exception_NONCONTINUABLE_EXCEPTION.hxx>
#include <OSD_Exception_PRIV_INSTRUCTION.hxx>
#include <OSD_Exception_STACK_OVERFLOW.hxx>
#include <OSD_Exception_STATUS_NO_MEMORY.hxx>
#include <OSD_FromWhere.hxx>
#include <OSD_KindFile.hxx>
#include <OSD_LoadMode.hxx>
#include <OSD_LockType.hxx>
#include <OSD_OEMType.hxx>
#include <OSD_OSDError.hxx>
#include <OSD_OpenMode.hxx>
#include <OSD_Protection.hxx>
#include <OSD_SIGBUS.hxx>
#include <OSD_SIGHUP.hxx>
#include <OSD_SIGILL.hxx>
#include <OSD_SIGINT.hxx>
#include <OSD_SIGKILL.hxx>
#include <OSD_SIGQUIT.hxx>
#include <OSD_SIGSEGV.hxx>
#include <OSD_SIGSYS.hxx>
#include <OSD_Signal.hxx>
#include <OSD_SignalMode.hxx>
#include <OSD_SingleProtection.hxx>
#include <OSD_SysType.hxx>
#include <OSD_WhoAmI.hxx>

static OSD_SingleProtection osd_single(std::istream& in)
{
  int aValue = 0;
  in >> aValue;
  return static_cast<OSD_SingleProtection>(aValue);
}

static void osd_prot_out(std::ostream& out, const OSD_Protection& theProt)
{
  out << theProt.System() << ' ' << theProt.User() << ' ' << theProt.Group() << ' '
      << theProt.World();
}

static void osd_prot_row(std::istream& in, std::ostream& out)
{
  const OSD_SingleProtection aLeft = osd_single(in);
  OSD_Protection             aProt;
  for (int aRight = 0; aRight < 16; ++aRight)
  {
    OSD_SingleProtection anAdd = aLeft, aSub = aLeft;
    aProt.Add(anAdd, static_cast<OSD_SingleProtection>(aRight));
    aProt.Sub(aSub, static_cast<OSD_SingleProtection>(aRight));
    out << (aRight == 0 ? "" : " ") << anAdd << '/' << aSub;
  }
}

static void osd_prot_set(std::istream& in, std::ostream& out)
{
  const OSD_SingleProtection s = osd_single(in), u = osd_single(in), g = osd_single(in),
                             w = osd_single(in);
  OSD_Protection aProt(s, u, g, w);
  OSD_SingleProtection s2, u2, g2, w2;
  aProt.Values(s2, u2, g2, w2);
  out << s2 << ' ' << u2 << ' ' << g2 << ' ' << w2 << ' ';
  // Rotate the rights through the individual setters, then SetValues back.
  aProt.SetSystem(w);
  aProt.SetUser(s);
  aProt.SetGroup(u);
  aProt.SetWorld(g);
  osd_prot_out(out, aProt);
  aProt.SetValues(g, w, s, u);
  out << ' ';
  osd_prot_out(out, aProt);
}

static void osd_error_probe(std::istream& in, std::ostream& out)
{
  int anErrno = 0, aFrom = 0;
  in >> anErrno >> aFrom;
  const std::string aMessage = osd_token(in);
  OSD_Error anError;
  out << anError.Failed() << ' ' << anError.Error();
  anError.SetValue(anErrno, aFrom, aMessage.c_str());
  out << ' ' << anError.Failed() << ' ' << anError.Error();
  try
  {
    anError.Perror();
    out << " ok";
  }
  catch (const OSD_OSDError& theError)
  {
    out << ' ' << theError.ExceptionType() << osd_escape(theError.what());
  }
  out << ' ' << anError.Error();
  anError.Reset();
  out << ' ' << anError.Failed() << ' ' << anError.Error();
}

template <typename T>
static void osd_raise(std::ostream& out, const char* theMessage)
{
  try
  {
    throw T(theMessage);
  }
  catch (const Standard_Failure& theFailure)
  {
    out << theFailure.ExceptionType() << osd_escape(theFailure.what());
  }
}

static void osd_exception_probe(std::istream& in, std::ostream& out)
{
  int anIndex = 0;
  in >> anIndex;
  const std::string aText = osd_token(in);
  const char* aMessage = aText.c_str();
  switch (anIndex)
  {
    case 0: osd_raise<OSD_Exception>(out, aMessage); break;
    case 1: osd_raise<OSD_Exception_ACCESS_VIOLATION>(out, aMessage); break;
    case 2: osd_raise<OSD_Exception_ARRAY_BOUNDS_EXCEEDED>(out, aMessage); break;
    case 3: osd_raise<OSD_Exception_CTRL_BREAK>(out, aMessage); break;
    case 4: osd_raise<OSD_Exception_ILLEGAL_INSTRUCTION>(out, aMessage); break;
    case 5: osd_raise<OSD_Exception_IN_PAGE_ERROR>(out, aMessage); break;
    case 6: osd_raise<OSD_Exception_INT_OVERFLOW>(out, aMessage); break;
    case 7: osd_raise<OSD_Exception_INVALID_DISPOSITION>(out, aMessage); break;
    case 8: osd_raise<OSD_Exception_NONCONTINUABLE_EXCEPTION>(out, aMessage); break;
    case 9: osd_raise<OSD_Exception_PRIV_INSTRUCTION>(out, aMessage); break;
    case 10: osd_raise<OSD_Exception_STACK_OVERFLOW>(out, aMessage); break;
    case 11: osd_raise<OSD_Exception_STATUS_NO_MEMORY>(out, aMessage); break;
    case 12: osd_raise<OSD_Signal>(out, aMessage); break;
    case 13: osd_raise<OSD_SIGBUS>(out, aMessage); break;
    case 14: osd_raise<OSD_SIGHUP>(out, aMessage); break;
    case 15: osd_raise<OSD_SIGILL>(out, aMessage); break;
    case 16: osd_raise<OSD_SIGINT>(out, aMessage); break;
    case 17: osd_raise<OSD_SIGKILL>(out, aMessage); break;
    case 18: osd_raise<OSD_SIGQUIT>(out, aMessage); break;
    case 19: osd_raise<OSD_SIGSEGV>(out, aMessage); break;
    case 20: osd_raise<OSD_SIGSYS>(out, aMessage); break;
    default: osd_raise<OSD_OSDError>(out, aMessage);
  }
}

static void osd_enums_probe(std::ostream& out)
{
  out << OSD_Aix + 1 << ' ' << OSD_RWXD + 1 << ' ' << OSD_FromEnd + 1 << ' ' << OSD_UNKNOWN + 1
      << ' ' << OSD_ExclusiveLock + 1 << ' ' << OSD_ReadWrite + 1 << ' ' << OSD_AIX + 1 << ' '
      << OSD_RTLD_NOW + 1 << ' ' << OSD_WEnvironmentIterator + 1 << ' '
      << OSD_SignalMode_Unset + 1;
}

static bool osd_foundation_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "osd_prot_row")
    osd_prot_row(in, out);
  else if (op == "osd_prot_default")
    osd_prot_out(out, OSD_Protection());
  else if (op == "osd_prot_set")
    osd_prot_set(in, out);
  else if (op == "osd_error")
    osd_error_probe(in, out);
  else if (op == "osd_exception")
    osd_exception_probe(in, out);
  else if (op == "osd_enums")
    osd_enums_probe(out);
  else
    return false;
  return true;
}
