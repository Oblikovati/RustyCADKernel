// Standard_ErrorHandler cases: handler linking, callbacks, Abort and Raise.
// Included by tkernel_standard_runtime_probe.cxx, which defines OCC_CONVERT_SIGNALS.
#include <Standard_ErrorHandler.hxx>
#include <csetjmp>
#include <memory>

namespace standard_runtime_probe
{
// A callback that logs its DestroyCallback() calls as "cb<index>".
class RuntimeProbeCallback : public Standard_ErrorHandler::Callback
{
public:
  RuntimeProbeCallback(int theIndex, std::string& theLog)
      : myIndex(theIndex),
        myLog(theLog)
  {
  }

  void DestroyCallback() override { myLog += "cb" + std::to_string(myIndex) + ','; }

private:
  int          myIndex;
  std::string& myLog;
};

// "runtime_handler_script <op>...": 'h' links a handler, 'D<i>' destroys handler i, 'c' makes
// a callback, 'R<i>'/'U<i>' register/unregister callback i, 't' logs IsInTryBlock. Objects stay
// alive until the end; callbacks go first, so no callback outlives the handler it points to.
static std::string handler_script_case(std::istream& theInput)
{
  std::string                                         aLog;
  std::vector<std::unique_ptr<Standard_ErrorHandler>> aHandlers;
  std::vector<std::unique_ptr<RuntimeProbeCallback>>  aCallbacks;
  std::string                                         anOp;
  while (theInput >> anOp)
  {
    const std::size_t anIndex = anOp.size() > 1 ? std::stoul(anOp.substr(1)) : 0;
    switch (anOp[0])
    {
      case 'h': aHandlers.push_back(std::make_unique<Standard_ErrorHandler>()); break;
      case 'D': aHandlers.at(anIndex)->Destroy(); break;
      case 'c': aCallbacks.push_back(std::make_unique<RuntimeProbeCallback>(int(aCallbacks.size()), aLog)); break;
      case 'R': aCallbacks.at(anIndex)->RegisterCallback(); break;
      case 'U': aCallbacks.at(anIndex)->UnregisterCallback(); break;
      case 't': aLog += std::string("T") + bit(Standard_ErrorHandler::IsInTryBlock()) + ','; break;
      default: throw std::runtime_error("unknown handler op " + anOp);
    }
  }
  aCallbacks.clear();
  while (!aHandlers.empty())
    aHandlers.pop_back();
  return (aLog.empty() ? std::string("-") : aLog) + ' ' + bit(Standard_ErrorHandler::IsInTryBlock());
}

// Class name held by a handler's Error() variant, "none" for the empty state.
static std::string error_name(const Standard_ErrorHandler::SignalException& theError)
{
  return std::visit(
    [](const auto& theException) -> std::string {
      if constexpr (std::is_same_v<std::decay_t<decltype(theException)>, std::monostate>)
        return "none";
      else
        return theException.ExceptionType();
    },
    theError);
}

// Nested handlers; the innermost lands from Abort, reports Error() and what Raise() throws.
template <class Exception>
static std::string abort_with(const char* theMessage, int theDepth)
{
  const Exception                                     anError(theMessage);
  std::vector<std::unique_ptr<Standard_ErrorHandler>> anOuter;
  for (int aLevel = 1; aLevel < theDepth; ++aLevel)
    anOuter.push_back(std::make_unique<Standard_ErrorHandler>());
  std::string aResult;
  {
    Standard_ErrorHandler aHandler;
    // Nothing between setjmp and longjmp has a destructor or is modified before the jump.
    if (setjmp(aHandler.Label()))
    {
      aResult = error_name(aHandler.Error()) + ' '
                + tkernel_probe::guarded([&]() -> std::string {
                    aHandler.Raise();
                    return "not raised";
                  });
    }
    else
    {
      Standard_ErrorHandler::Abort(anError);
    }
  }
  const std::string anOuterError = anOuter.empty() ? "-" : error_name(anOuter.back()->Error());
  aResult += ' ' + anOuterError + ' ' + bit(Standard_ErrorHandler::IsInTryBlock());
  anOuter.clear();
  return aResult + bit(Standard_ErrorHandler::IsInTryBlock());
}

// "runtime_abort <Class> <hex message> <depth>".
static std::string abort_case(std::istream& theInput)
{
  using Aborter = std::function<std::string(const char*, int)>;
  static const std::map<std::string, Aborter> THE_ABORTERS = {
#define TKERNEL_ABORTER(C) {#C, abort_with<C>},
    TKERNEL_ABORTER(OSD_SIGBUS) TKERNEL_ABORTER(OSD_SIGHUP) TKERNEL_ABORTER(OSD_SIGILL)
      TKERNEL_ABORTER(OSD_SIGINT) TKERNEL_ABORTER(OSD_SIGKILL) TKERNEL_ABORTER(OSD_SIGQUIT)
        TKERNEL_ABORTER(OSD_SIGSEGV) TKERNEL_ABORTER(OSD_SIGSYS)
          TKERNEL_ABORTER(OSD_Exception_ACCESS_VIOLATION)
            TKERNEL_ABORTER(OSD_Exception_ARRAY_BOUNDS_EXCEEDED)
              TKERNEL_ABORTER(OSD_Exception_ILLEGAL_INSTRUCTION)
                TKERNEL_ABORTER(OSD_Exception_IN_PAGE_ERROR)
                  TKERNEL_ABORTER(OSD_Exception_INT_OVERFLOW)
                    TKERNEL_ABORTER(OSD_Exception_INVALID_DISPOSITION)
                      TKERNEL_ABORTER(OSD_Exception_NONCONTINUABLE_EXCEPTION)
                        TKERNEL_ABORTER(OSD_Exception_PRIV_INSTRUCTION)
                          TKERNEL_ABORTER(OSD_Exception_STACK_OVERFLOW)
                            TKERNEL_ABORTER(OSD_Exception_STATUS_NO_MEMORY)
                              TKERNEL_ABORTER(Standard_DivideByZero)
                                TKERNEL_ABORTER(Standard_NumericError)
                                  TKERNEL_ABORTER(Standard_Overflow)
                                    TKERNEL_ABORTER(Standard_ProgramError)
                                      TKERNEL_ABORTER(Standard_Underflow)
#undef TKERNEL_ABORTER
  };
  std::string aClass;
  int         aDepth = 1;
  theInput >> aClass;
  const std::string aMessage = tkernel_probe::next_bytes(theInput);
  theInput >> aDepth;
  return THE_ABORTERS.at(aClass)(aMessage.c_str(), aDepth);
}
} // namespace standard_runtime_probe
