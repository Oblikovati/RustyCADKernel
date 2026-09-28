// Standard_Failure, Standard_OutOfMemory, stack capture and Standard::StackTrace cases.
// Included by tkernel_standard_runtime_probe.cxx.
#include <Standard_OutOfMemory.hxx>
#include <Standard_OutOfRange.hxx>

namespace standard_runtime_probe
{
static std::string printed(const Standard_Failure& theFailure)
{
  std::ostringstream aStream;
  aStream << theFailure;
  return aStream.str();
}

// ExceptionType, what(), GetMessageString(), GetStackString() and operator<< of one failure.
static std::string failure_fields(const Standard_Failure& theFailure)
{
  return std::string(theFailure.ExceptionType()) + ' ' + tkernel_probe::hex(std::string(theFailure.what()))
         + ' ' + tkernel_probe::hex(std::string(theFailure.GetMessageString())) + ' '
         + tkernel_probe::hex(std::string(theFailure.GetStackString())) + ' '
         + tkernel_probe::hex(printed(theFailure));
}

// "runtime_failure <hex message>": the fields of Standard_Failure(msg), of a copy and of an
// assigned object, whether copies share the message, and the default-constructed failure.
static std::string failure_case(std::istream& theInput)
{
  const std::string      aMessage = tkernel_probe::next_bytes(theInput);
  const Standard_Failure aFailure(aMessage.c_str());
  const Standard_Failure aCopy(aFailure);
  Standard_Failure       anAssigned;
  anAssigned = aFailure;
  return failure_fields(aFailure) + ' ' + failure_fields(aCopy) + ' ' + failure_fields(anAssigned) + ' '
         + bit(aCopy.what() == aFailure.what()) + bit(anAssigned.what() == aFailure.what()) + ' '
         + failure_fields(Standard_Failure());
}

// "runtime_failure_stack <hex message> <hex stack>": the (message, stack trace) constructor.
static std::string failure_stack_case(std::istream& theInput)
{
  const std::string aMessage = tkernel_probe::next_bytes(theInput);
  const std::string aStack   = tkernel_probe::next_bytes(theInput);
  return failure_fields(Standard_Failure(aMessage.c_str(), aStack.c_str()));
}

// A message token: "null" is nullptr, anything else hex bytes.
static std::string next_message(std::istream& theInput, bool& theIsNull)
{
  std::string aToken;
  theInput >> aToken;
  theIsNull = aToken == "null";
  return theIsNull ? std::string() : tkernel_probe::decode(aToken);
}

// "runtime_out_of_memory <message> <new message>": the fixed buffer, Print without the
// buffer, catch as Standard_ProgramError, a copy, and SetMessageString.
static std::string out_of_memory_case(std::istream& theInput)
{
  bool              isNull = false, isNewNull = false;
  const std::string aMessage    = next_message(theInput, isNull);
  const std::string aNewMessage = next_message(theInput, isNewNull);
  Standard_OutOfMemory anError(isNull ? nullptr : aMessage.c_str());
  const Standard_OutOfMemory aCopy(anError);
  anError.SetMessageString(isNewNull ? nullptr : aNewMessage.c_str());
  char aCaught = '0';
  try
  {
    throw aCopy;
  }
  catch (const Standard_ProgramError&)
  {
    aCaught = '1';
  }
  return failure_fields(aCopy) + ' ' + failure_fields(anError) + ' ' + aCaught;
}

// Header/footer/frame shape of a trace: "H" for the header, "F" per frame, "T" for the footer.
static std::string trace_shape(const std::string& theTrace)
{
  std::string aShape;
  std::size_t aStart = 0;
  while (aStart < theTrace.size())
  {
    const std::size_t anEnd  = theTrace.find('\n', aStart + 1);
    const std::string aLine  = theTrace.substr(aStart, anEnd == std::string::npos ? anEnd : anEnd - aStart);
    aShape += aLine == "\n==Backtrace==" ? 'H' : aLine == "\n=============" ? 'T' : 'F';
    aStart = anEnd == std::string::npos ? theTrace.size() : anEnd;
  }
  return aShape.empty() ? "-" : aShape;
}

// "runtime_stack_length <n>": DefaultStackTraceLength round trip and the trace shape that the
// message constructors of Standard_Failure, Standard_OutOfRange and Standard_OutOfMemory capture.
static std::string stack_length_case(std::istream& theInput)
{
  int aLength = 0;
  theInput >> aLength;
  Standard_Failure::SetDefaultStackTraceLength(aLength);
  const Standard_Failure     aFailure("m");
  const Standard_OutOfRange  aRange("r");
  const Standard_OutOfMemory anOutOfMemory("o");
  const Standard_Failure     anExplicit("e", "");
  const std::string anOutOfMemoryText = printed(anOutOfMemory);
  const std::string aResult = std::to_string(Standard_Failure::DefaultStackTraceLength()) + ' '
                              + trace_shape(aFailure.GetStackString()).substr(0, 1) + ' '
                              + trace_shape(aRange.GetStackString()).substr(0, 1) + ' '
                              + anOutOfMemoryText.substr(0, 20)
                              + trace_shape(anOutOfMemoryText.substr(20)).substr(0, 1) + ' '
                              + tkernel_probe::hex(printed(anExplicit));
  Standard_Failure::SetDefaultStackTraceLength(0);
  const std::string aStack = aFailure.GetStackString();
  return aResult + ' ' + bit(aStack.size() > 28 && aStack.compare(aStack.size() - 14, 14, "\n=============") == 0);
}

// "runtime_stack_trace <size> <traces> <skip> <hex prefix>": Standard::StackTrace on a
// buffer of <size> bytes that already holds the prefix; the kept prefix and trace shape.
static std::string stack_trace_case(std::istream& theInput)
{
  int aSize = 0, aTraces = 0, aSkip = 0;
  theInput >> aSize >> aTraces >> aSkip;
  const std::string aPrefix = tkernel_probe::next_bytes(theInput);
  std::vector<char> aBuffer(std::max<std::size_t>(aSize > 0 ? aSize : 1, aPrefix.size() + 1), '\0');
  std::copy(aPrefix.begin(), aPrefix.end(), aBuffer.begin());
  const bool        isDone = Standard::StackTrace(aBuffer.data(), aSize, aTraces, nullptr, aSkip);
  const std::string aText(aBuffer.data());
  return std::string(1, bit(isDone)) + ' ' + tkernel_probe::hex(aText.substr(0, aPrefix.size())) + ' '
         + trace_shape(aText.substr(aPrefix.size()));
}
} // namespace standard_runtime_probe
