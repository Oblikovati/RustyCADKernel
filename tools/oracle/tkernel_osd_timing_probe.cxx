// OSD timing probe operations (OSD_Timer, OSD_Chronometer, OSD package statics and
// signals); twin of oracle_probe/osd/timing.rs. Doubles travel as IEEE-754 bit patterns
// in hex so that the comparison is exact.
//
// osd_timer_reset <bits>          Reset(double) on a fresh timer: ElapsedTime, IsStarted,
//                                 Show(s, m, h, cpu) and the Show(ostream) text.
// osd_chrono <0|1>                A stopped chronometer: flags, Show texts and values,
//                                 SetThisThreadOnly stopped and while started.
// osd_real_to_cstring <bits>      RealToCString.
// osd_cstring_to_real <hex|->     CStringToReal of the hex-encoded bytes.
// osd_signal_stack <n>            SignalStackTraceLength before/after a set, then 0 again.
// osd_signal <mode> <0|1>         SetSignal, SignalMode, ToCatchFloatingSignals, then traps off.
// osd_thread_signal <mode> <0|1>  SetThreadLocalSignal, ToCatchFloatingSignals, traps off.
// osd_floating <0|1>              SetFloatingSignal, ToCatchFloatingSignals, traps off.
// osd_ctrl_break                  SetSignal(Set), raise(SIGINT), ControlBreak x2, Unset.
// osd_sleep <s> <ms>              SecSleep and MilliSecSleep (0 only in the corpus).
#include <OSD.hxx>
#include <OSD_Chronometer.hxx>
#include <OSD_Exception_CTRL_BREAK.hxx>
#include <OSD_Timer.hxx>
#include <Standard_ProgramError.hxx>

#include <csignal>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <sstream>

static double osd_timing_real(std::istream& in)
{
  std::string aHex;
  in >> aHex;
  const uint64_t aBits = std::stoull(aHex, nullptr, 16);
  double         aValue;
  std::memcpy(&aValue, &aBits, sizeof(aValue));
  return aValue;
}

static std::string osd_timing_bits(double theValue)
{
  uint64_t aBits;
  std::memcpy(&aBits, &theValue, sizeof(aBits));
  std::ostringstream aText;
  aText << std::hex << std::setw(16) << std::setfill('0') << aBits;
  return aText.str();
}

static void osd_timer_reset(std::istream& in, std::ostream& out)
{
  OSD_Timer aTimer;
  aTimer.Reset(osd_timing_real(in));
  double aSeconds = 0.0, aCpu = 0.0;
  int    aMinutes = 0, anHours = 0;
  aTimer.Show(aSeconds, aMinutes, anHours, aCpu);
  std::ostringstream aText;
  aTimer.Show(aText);
  out << osd_timing_bits(aTimer.ElapsedTime()) << ' ' << aTimer.IsStarted() << ' '
      << osd_timing_bits(aSeconds) << ' ' << aMinutes << ' ' << anHours << ' '
      << osd_timing_bits(aCpu) << ' ' << osd_escape(aText.str());
}

static void osd_chrono(std::istream& in, std::ostream& out)
{
  int aFlag = 0;
  in >> aFlag;
  OSD_Chronometer aChrono(aFlag != 0);
  std::ostringstream aText;
  aChrono.Show(aText);
  double aUser = -1.0, aSys = -1.0, aUserOnly = -1.0;
  aChrono.Show(aUser, aSys);
  aChrono.Show(aUserOnly);
  out << aChrono.IsStarted() << ' ' << aChrono.IsThisThreadOnly() << ' ' << osd_escape(aText.str())
      << ' ' << osd_timing_bits(aUser) << ' ' << osd_timing_bits(aSys) << ' '
      << osd_timing_bits(aUserOnly) << ' ' << osd_timing_bits(aChrono.UserTimeCPU()) << ' '
      << osd_timing_bits(aChrono.SystemTimeCPU());
  aChrono.SetThisThreadOnly(aFlag == 0);
  out << ' ' << aChrono.IsThisThreadOnly();
  aChrono.Start();
  out << ' ' << aChrono.IsStarted();
  try
  {
    aChrono.SetThisThreadOnly(aFlag != 0);
    out << " ok";
  }
  catch (const Standard_Failure& theFailure)
  {
    out << ' ' << theFailure.ExceptionType() << osd_escape(theFailure.what());
  }
  aChrono.Stop();
  out << ' ' << aChrono.IsStarted() << ' ' << aChrono.IsThisThreadOnly();
  aChrono.Reset();
  std::ostringstream aReset;
  aChrono.Show(aReset);
  out << ' ' << osd_escape(aReset.str());
}

static void osd_real_to_cstring(std::istream& in, std::ostream& out)
{
  char                aBuffer[64] = {};
  Standard_PCharacter aString     = aBuffer;
  const bool          isOk        = OSD::RealToCString(osd_timing_real(in), aString);
  out << isOk << ' ' << osd_escape(aBuffer);
}

static void osd_cstring_to_real(std::istream& in, std::ostream& out)
{
  const std::string aHex = osd_token(in);
  std::string       aText;
  for (size_t anIndex = 0; anIndex + 1 < aHex.size(); anIndex += 2)
  {
    aText += static_cast<char>(std::stoi(aHex.substr(anIndex, 2), nullptr, 16));
  }
  double     aValue = -7.0;
  const bool isOk   = OSD::CStringToReal(aText.c_str(), aValue);
  out << isOk << ' ' << osd_timing_bits(aValue);
}

static void osd_signal_stack(std::istream& in, std::ostream& out)
{
  int aLength = 0;
  in >> aLength;
  out << OSD::SignalStackTraceLength();
  OSD::SetSignalStackTraceLength(aLength);
  out << ' ' << OSD::SignalStackTraceLength();
  OSD::SetSignalStackTraceLength(0);
  out << ' ' << OSD::SignalStackTraceLength();
}

static void osd_floating_off(std::ostream& out)
{
  OSD::SetFloatingSignal(false);
  out << ' ' << OSD::ToCatchFloatingSignals();
}

static void osd_signal(const std::string& op, std::istream& in, std::ostream& out)
{
  int aMode = 0, aFloating = 0;
  if (op != "osd_floating")
  {
    in >> aMode;
  }
  in >> aFloating;
  if (op == "osd_signal")
  {
    OSD::SetSignal(static_cast<OSD_SignalMode>(aMode), aFloating != 0);
  }
  else if (op == "osd_thread_signal")
  {
    OSD::SetThreadLocalSignal(static_cast<OSD_SignalMode>(aMode), aFloating != 0);
  }
  else
  {
    OSD::SetFloatingSignal(aFloating != 0);
  }
  const bool isCatching = OSD::ToCatchFloatingSignals();
  osd_floating_off(out);
  out << ' ' << OSD::SignalMode() << ' ' << isCatching;
}

static void osd_control_break(std::ostream& out)
{
  for (int aCall = 0; aCall < 3; ++aCall)
  {
    if (aCall == 1)
    {
      OSD::SetSignal(OSD_SignalMode_Set, false);
      std::raise(SIGINT);
    }
    try
    {
      OSD::ControlBreak();
      out << (aCall == 0 ? "" : " ") << "ok";
    }
    catch (const Standard_Failure& theFailure)
    {
      out << (aCall == 0 ? "" : " ") << theFailure.ExceptionType() << osd_escape(theFailure.what());
    }
  }
  OSD::SetSignal(OSD_SignalMode_Unset, false);
  out << ' ' << OSD::SignalMode();
  OSD::SetSignal(OSD_SignalMode_AsIs, false);
  out << ' ' << OSD::SignalMode();
}

static void osd_sleep(std::istream& in, std::ostream& out)
{
  int aSeconds = 0, aMilliseconds = 0;
  in >> aSeconds >> aMilliseconds;
  OSD::SecSleep(aSeconds);
  OSD::MilliSecSleep(aMilliseconds);
  out << "slept";
}

static bool osd_timing_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "osd_timer_reset")
    osd_timer_reset(in, out);
  else if (op == "osd_chrono")
    osd_chrono(in, out);
  else if (op == "osd_real_to_cstring")
    osd_real_to_cstring(in, out);
  else if (op == "osd_cstring_to_real")
    osd_cstring_to_real(in, out);
  else if (op == "osd_signal_stack")
    osd_signal_stack(in, out);
  else if (op == "osd_signal" || op == "osd_thread_signal" || op == "osd_floating")
    osd_signal(op, in, out);
  else if (op == "osd_ctrl_break")
    osd_control_break(out);
  else if (op == "osd_sleep")
    osd_sleep(in, out);
  else
    return false;
  return true;
}
