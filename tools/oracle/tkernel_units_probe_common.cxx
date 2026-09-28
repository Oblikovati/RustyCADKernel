// Shared encoding and capture helpers of the units probe family.
#include <Message.hxx>
#include <Message_Messenger.hxx>
#include <Message_Printer.hxx>
#include <Message_PrinterOStream.hxx>
#include <Standard_Failure.hxx>
#include <TCollection_AsciiString.hxx>
#include <Units_Dimensions.hxx>
#include <Units_ShiftedToken.hxx>
#include <Units_Token.hxx>
#include <NCollection_HSequence.hxx>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <functional>
#include <iostream>
#include <istream>
#include <ostream>
#include <sstream>
#include <string>

static std::string units_unhex(const std::string& theText)
{
  std::string aValue;
  if (theText == "-")
    return aValue;
  for (size_t anIndex = 0; anIndex + 1 < theText.size(); anIndex += 2)
    aValue.push_back(static_cast<char>(std::stoi(theText.substr(anIndex, 2), nullptr, 16)));
  return aValue;
}

static std::string units_hex(const std::string& theBytes)
{
  static const char* const THE_DIGITS = "0123456789abcdef";
  std::string aText;
  for (unsigned char aByte : theBytes)
  {
    aText += THE_DIGITS[aByte >> 4];
    aText += THE_DIGITS[aByte & 15];
  }
  return aText.empty() ? "-" : aText;
}

static std::string units_arg(std::istream& in)
{
  std::string aToken;
  in >> aToken;
  return units_unhex(aToken);
}

static double units_real(std::istream& in)
{
  double aValue = 0.0;
  in >> aValue;
  return aValue;
}

static uint64_t units_bits(double theValue)
{
  uint64_t aBits = 0;
  std::memcpy(&aBits, &theValue, sizeof(aBits));
  return aBits;
}

static std::string& units_messages()
{
  static std::string aMessages;
  return aMessages;
}

//! Records every messenger payload with its gravity instead of printing it.
class UnitsCapturePrinter : public Message_Printer
{
protected:
  void send(const TCollection_AsciiString& theString, const Message_Gravity theGravity) const override
  {
    units_messages() += theGravity == Message_Warning ? "W:" : (theGravity == Message_Info ? "I:" : "X:");
    units_messages() += theString.ToCString();
    units_messages() += "\n";
  }
};

//! Replaces the console printers once; the resource variables are cleared so that
//! both probes start from empty current-unit resources.
static void units_install()
{
  static bool isInstalled = false;
  if (isInstalled)
    return;
  isInstalled = true;
  const occ::handle<Message_Messenger>& aMessenger = Message::DefaultMessenger();
  aMessenger->RemovePrinters(STANDARD_TYPE(Message_PrinterOStream));
  aMessenger->AddPrinter(new UnitsCapturePrinter());
  for (const char* aName : {"CSF_CurrentUnitsDefaults", "CSF_CurrentUnitsUserDefaults",
                            "CSF_MDTVCurrentUnitsDefaults", "CSF_MDTVCurrentUnitsUserDefaults",
                            "CSF_UnitsDefaults", "CSF_UnitsUserDefaults"})
    unsetenv(aName);
}

//! Hides the object address of "Units_Token::Dump of <address>".
static std::string units_mask(const std::string& theText)
{
  static const std::string THE_MARK = "Units_Token::Dump of ";
  std::string aText = theText;
  for (size_t aPos = aText.find(THE_MARK); aPos != std::string::npos; aPos = aText.find(THE_MARK, aPos + 1))
  {
    const size_t aStart = aPos + THE_MARK.size();
    const size_t anEnd  = aText.find('\n', aStart);
    aText.replace(aStart, anEnd - aStart, "@");
  }
  return aText;
}

//! Runs one request body with std::cout and the messenger captured.
static void units_run(std::ostream& out, const std::function<void(std::ostream&)>& theBody)
{
  units_install();
  units_messages().clear();
  std::ostringstream aConsole;
  std::streambuf*    aSaved = std::cout.rdbuf(aConsole.rdbuf());
  std::ostringstream aResult;
  try
  {
    theBody(aResult);
  }
  catch (const Standard_Failure& theFailure)
  {
    aResult << "EXC " << theFailure.ExceptionType() << ' ' << units_hex(theFailure.GetMessageString());
  }
  std::cout.rdbuf(aSaved);
  out << aResult.str() << " cout=" << units_hex(units_mask(aConsole.str()))
      << " msg=" << units_hex(units_messages());
}

static void units_dims(std::ostream& out, const occ::handle<Units_Dimensions>& theDims)
{
  if (theDims.IsNull())
  {
    out << "null";
    return;
  }
  out << units_bits(theDims->Mass()) << ',' << units_bits(theDims->Length()) << ','
      << units_bits(theDims->Time()) << ',' << units_bits(theDims->ElectricCurrent()) << ','
      << units_bits(theDims->ThermodynamicTemperature()) << ','
      << units_bits(theDims->AmountOfSubstance()) << ','
      << units_bits(theDims->LuminousIntensity()) << ',' << units_bits(theDims->PlaneAngle())
      << ',' << units_bits(theDims->SolidAngle());
}

static void units_token(std::ostream& out, const occ::handle<Units_Token>& theToken)
{
  if (theToken.IsNull())
  {
    out << "null";
    return;
  }
  const bool isShifted = theToken->IsKind(STANDARD_TYPE(Units_ShiftedToken));
  out << (isShifted ? 'S' : 'P') << ' ' << units_hex(theToken->Word().ToCString()) << ' '
      << units_hex(theToken->Mean().ToCString()) << ' ' << units_bits(theToken->Value()) << ' ';
  units_dims(out, theToken->Dimensions());
  if (isShifted)
    out << ' ' << units_bits(occ::down_cast<Units_ShiftedToken>(theToken)->Move());
}

static void units_tokens(std::ostream& out,
                         const occ::handle<NCollection_HSequence<occ::handle<Units_Token>>>& theSeq)
{
  if (theSeq.IsNull())
  {
    out << "nullseq";
    return;
  }
  out << theSeq->Length();
  for (int anIndex = 1; anIndex <= theSeq->Length(); ++anIndex)
  {
    out << " [";
    units_token(out, theSeq->Value(anIndex));
    out << ']';
  }
}
