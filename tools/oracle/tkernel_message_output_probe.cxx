// Message output operations: Message_Algorithm, Message_Messenger, Message_PrinterOStream
// (twin of oracle_probe/message/output.rs).
//
// algo_run <step>...        st:<s> si:<s>:<int> ss/sa:<s>:<bytes>:<norep> se:<s>:<units>:<norep>
//                           sn:<s> sm:<s>:<units> clear cat:<key bytes>:<units> send:<g>:<max>
//                           sendst:<s>:<g>:<max> o<step> (on the second algorithm) add addf:<s>
//                           nums:<s>:<max> strs:<s>:<max>; prints the messenger log and flags
// messenger_ops <step>...   p r o:<path> dup rm rmt:<B|R|O> send:<g>:<bytes> trace:<g> json
// printer_file <path> <prefill> <append> <g> <colorize> <g>:<bytes>...  file printer, then the file bytes
// printer_badfile <path>    a printer whose file cannot open falls back to std::cout
// printer_console <color> <intense>  SetConsoleTextColor into a string stream
#include <Message_Algorithm.hxx>
#include <Message_PrinterOStream.hxx>
#include <TColStd_HPackedMapOfInteger.hxx>

//! A Message_Algorithm with an explicit messenger collecting everything it sends.
struct MsgProbe_Algorithms
{
  occ::handle<Message_Algorithm> myMain  = new Message_Algorithm();
  occ::handle<Message_Algorithm> myOther = new Message_Algorithm();
  occ::handle<MsgProbe_Printer>  myPrinter = new MsgProbe_Printer();
  MsgProbe_Algorithms()
  {
    myPrinter->SetTraceLevel(Message_Trace);
    occ::handle<Message_Messenger> aMessenger = new Message_Messenger(myPrinter);
    myMain->SetMessenger(aMessenger);
    myOther->SetMessenger(aMessenger);
  }
};

static Message_Status msgp_status(const std::string& theText)
{
  return (Message_Status)std::stoi(theText);
}

//! Status-setting steps shared by both algorithms; false for other steps.
static bool msgp_algo_set(const occ::handle<Message_Algorithm>& theAlgo, const std::vector<std::string>& theStep)
{
  const std::string& anOp = theStep[0];
  const bool isNoRep = theStep.size() > 3 && theStep[3] == "1";
  if (anOp == "st") theAlgo->SetStatus(msgp_status(theStep[1]));
  else if (anOp == "si") theAlgo->SetStatus(msgp_status(theStep[1]), std::stoi(theStep[2]));
  else if (anOp == "ss") theAlgo->SetStatus(msgp_status(theStep[1]), msgp_bytes(theStep[2]).c_str(), isNoRep);
  else if (anOp == "sa") theAlgo->SetStatus(msgp_status(theStep[1]), TCollection_AsciiString(msgp_bytes(theStep[2]).c_str()), isNoRep);
  else if (anOp == "se") theAlgo->SetStatus(msgp_status(theStep[1]), msgp_ext(theStep[2]), isNoRep);
  else if (anOp == "sn") theAlgo->SetStatus(msgp_status(theStep[1]), occ::handle<TCollection_HExtendedString>());
  else if (anOp == "sm")
  {
    Message_Msg aMsg;
    aMsg.Set(msgp_ext(theStep[2]));
    theAlgo->SetStatus(msgp_status(theStep[1]), aMsg);
  }
  else if (anOp == "clear") theAlgo->ClearStatus();
  else return false;
  return true;
}

static void msgp_algo_query(const occ::handle<Message_Algorithm>& theAlgo, const std::vector<std::string>& theStep, std::ostream& out)
{
  const Message_Status aStatus = msgp_status(theStep[1]);
  const int aMax = std::stoi(theStep[2]);
  if (theStep[0] == "nums")
  {
    occ::handle<TColStd_HPackedMapOfInteger> aNumbers = theAlgo->GetMessageNumbers(aStatus);
    out << " N" << (aNumbers.IsNull() ? "-" : msgp_units(Message_Algorithm::PrepareReport(aNumbers, aMax)));
    return;
  }
  auto aStrings = theAlgo->GetMessageStrings(aStatus);
  out << " S" << (aStrings.IsNull() ? "-" : msgp_units(Message_Algorithm::PrepareReport(aStrings->Sequence(), aMax)));
}

static void msgp_algo_step(MsgProbe_Algorithms& theAlgos, const std::vector<std::string>& theStep, std::ostream& out)
{
  const std::string& anOp = theStep[0];
  if (msgp_algo_set(theAlgos.myMain, theStep)) return;
  if (anOp[0] == 'o')
  {
    std::vector<std::string> aStep = theStep;
    aStep[0] = anOp.substr(1);
    msgp_algo_set(theAlgos.myOther, aStep);
  }
  else if (anOp == "cat") Message_MsgFile::AddMsg(TCollection_AsciiString(msgp_bytes(theStep[1]).c_str()), msgp_ext(theStep[2]));
  else if (anOp == "send") theAlgos.myMain->SendMessages((Message_Gravity)std::stoi(theStep[1]), std::stoi(theStep[2]));
  else if (anOp == "sendst") theAlgos.myMain->SendStatusMessages(Message_ExecStatus(msgp_status(theStep[1])), (Message_Gravity)std::stoi(theStep[2]), std::stoi(theStep[3]));
  else if (anOp == "add") theAlgos.myMain->AddStatus(theAlgos.myOther);
  else if (anOp == "addf") theAlgos.myMain->AddStatus(Message_ExecStatus(msgp_status(theStep[1])), theAlgos.myOther);
  else msgp_algo_query(theAlgos.myMain, theStep, out);
}

static void msgp_algo_run(std::istream& in, std::ostream& out)
{
  MsgProbe_Algorithms anAlgos;
  for (const std::string& aStep : msgp_rest(in))
  {
    try
    {
      msgp_algo_step(anAlgos, msgp_fields(aStep), out);
    }
    catch (const Standard_Failure& theFailure)
    {
      out << " E" << theFailure.ExceptionType();
    }
  }
  out << " L:" << msgp_hex(anAlgos.myPrinter->myLog) << ' ';
  for (int anIndex = Message_ExecStatus::FirstStatus; anIndex < Message_ExecStatus::LastStatus; ++anIndex)
  {
    out << anAlgos.myMain->GetStatus().IsSet(Message_ExecStatus::StatusByIndex(anIndex));
  }
}

static occ::handle<Standard_Type> msgp_printer_type(char theKind)
{
  switch (theKind)
  {
    case 'R': return STANDARD_TYPE(Message_PrinterToReport);
    case 'O': return STANDARD_TYPE(Message_PrinterOStream);
    default: return STANDARD_TYPE(Message_Printer);
  }
}

static void msgp_messenger_ops(std::istream& in, std::ostream& out)
{
  occ::handle<Message_Messenger> aMessenger = new Message_Messenger();
  aMessenger->ChangePrinters().Clear();
  std::vector<occ::handle<MsgProbe_Printer>> aCollectors;
  occ::handle<Message_Printer>               aLast;
  for (const std::string& aText : msgp_rest(in))
  {
    const std::vector<std::string> aStep = msgp_fields(aText);
    const std::string&             anOp  = aStep[0];
    if (anOp == "p") { aCollectors.push_back(new MsgProbe_Printer()); aLast = aCollectors.back(); out << ' ' << aMessenger->AddPrinter(aLast); }
    else if (anOp == "r") { aLast = new Message_PrinterToReport(); out << ' ' << aMessenger->AddPrinter(aLast); }
    else if (anOp == "o") { aLast = new Message_PrinterOStream(msgp_bytes(aStep[1]).c_str(), false); out << ' ' << aMessenger->AddPrinter(aLast); }
    else if (anOp == "dup") out << ' ' << aMessenger->AddPrinter(aLast);
    else if (anOp == "rm") out << ' ' << aMessenger->RemovePrinter(aLast);
    else if (anOp == "rmt") out << ' ' << aMessenger->RemovePrinters(msgp_printer_type(aStep[1][0]));
    else if (anOp == "send") aMessenger->Send(msgp_bytes(aStep[2]).c_str(), (Message_Gravity)std::stoi(aStep[1]));
    else if (anOp == "trace") for (auto& aCollector : aCollectors) aCollector->SetTraceLevel((Message_Gravity)std::stoi(aStep[1]));
    else if (anOp == "json")
    {
      Standard_SStream aJson;
      aMessenger->DumpJson(aJson);
      out << " J" << msgp_hex(aJson.str());
    }
  }
  out << " n" << aMessenger->Printers().Size();
  for (const occ::handle<MsgProbe_Printer>& aCollector : aCollectors)
  {
    out << ' ' << msgp_hex(aCollector->myLog);
  }
}

static void msgp_printer_file(std::istream& in, std::ostream& out)
{
  std::string aPath, aPrefill;
  int         anAppend = 0, aTrace = 0, aColorize = 0;
  in >> aPath >> aPrefill >> anAppend >> aTrace >> aColorize;
  const std::string aFile = msgp_bytes(aPath);
  std::filesystem::create_directories(std::filesystem::path(aFile).parent_path());
  // Files outlive a probe run: start from known content so appending is reproducible.
  std::ofstream(aFile, std::ios::binary) << msgp_bytes(aPrefill);
  {
    occ::handle<Message_PrinterOStream> aPrinter = new Message_PrinterOStream(aFile.c_str(), anAppend != 0, (Message_Gravity)aTrace);
    out << aPrinter->ToColorize();
    aPrinter->SetToColorize(aColorize != 0);
    for (const std::string& aText : msgp_rest(in))
    {
      const std::vector<std::string> aStep = msgp_fields(aText);
      aPrinter->Send(TCollection_AsciiString(msgp_bytes(aStep[1]).c_str()), (Message_Gravity)std::stoi(aStep[0]));
    }
    aPrinter->Close();
    aPrinter->Send(TCollection_AsciiString("after close"), Message_Fail);
  }
  std::ifstream      aStream(aFile, std::ios::binary);
  std::ostringstream aContent;
  aContent << aStream.rdbuf();
  out << ' ' << msgp_hex(aContent.str());
}

class Probe_MessageAlgorithm : public Message_Algorithm
{
public:
  DEFINE_STANDARD_RTTIEXT(Probe_MessageAlgorithm, Message_Algorithm)
};
IMPLEMENT_STANDARD_RTTIEXT(Probe_MessageAlgorithm, Message_Algorithm)

static void msgp_algo_handles(std::ostream& out)
{
  occ::handle<Probe_MessageAlgorithm> derived = new Probe_MessageAlgorithm;
  occ::handle<Message_Algorithm> base = derived;
  occ::handle<Standard_Transient> root = base;
  base->SetStatus(Message_Fail2, 17);
  const bool downcast = !occ::down_cast<Probe_MessageAlgorithm>(root).IsNull();
  out << derived->GetStatus().IsSet(Message_Fail2) << ' ' << base->GetMessageNumbers(Message_Fail2)->Map().Extent()
      << ' ' << root->DynamicType()->Name() << ' ' << base->DynamicType()->Name()
      << ' ' << downcast << ' ' << root->GetRefCount();
  base->ChangeStatus().Clear(Message_Fail2);
  out << ' ' << derived->GetStatus().IsFail() << ' ' << derived->GetMessageNumbers(Message_Fail2)->Map().Contains(17);
  base->ClearStatus();
  out << ' ' << derived->GetMessageNumbers(Message_Fail2).IsNull();
}

static bool message_output_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "algo_run") msgp_algo_run(in, out);
  else if (op == "algo_handles") msgp_algo_handles(out);
  else if (op == "messenger_ops") msgp_messenger_ops(in, out);
  else if (op == "printer_file") msgp_printer_file(in, out);
  else if (op == "printer_badfile")
  {
    std::string aPath;
    in >> aPath;
    occ::handle<Message_PrinterOStream> aPrinter = new Message_PrinterOStream(msgp_bytes(aPath).c_str(), false);
    out << aPrinter->ToColorize();
  }
  else if (op == "printer_console")
  {
    int aColor = 0, anIntense = 0;
    in >> aColor >> anIntense;
    std::ostringstream aStream;
    Message_PrinterOStream::SetConsoleTextColor(&aStream, (Message_ConsoleColor)aColor, anIntense != 0);
    out << msgp_hex(aStream.str());
  }
  else return false;
  return true;
}
