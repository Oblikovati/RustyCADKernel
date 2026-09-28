// Message report operations: alerts, attributes, Message_Report, Message_Level and
// Message_PrinterToReport (twin of oracle_probe/message/report.rs).
//
// Alert kinds: P Message_Alert, X/Y simple alerts, N no-merge alert, R merge-refusing
// alert, E<name> extended+attribute, M<name> extended+meter, S<name> extended+stream,
// O<name> extended+object (null object).
// report_run <step>...  add:<g>:<kind> ext:<g>:<kind> nest:<g>:<kind> limit:<n> clear
//                       clearg:<g> cleart:<kind> metric:<m>:<0|1> clearmetrics has:<kind>
//                       merge:<g|a>; prints alerts, Dump, DumpJson and SendMessages
// level_run <step>...   default report active in the default messenger: open:<name> close
//                       add:<g>:<kind> levadd:<g>:<kind> send:<g>:<bytes> stream:<g>:<bytes>
//                       metric:<m>:<0|1>; prints the alert tree after closing every level
// ptr_run <step>...     PrinterToReport on a fresh report: send/cstr:<g>:<bytes>
//                       ext:<g>:<units> stream:<g>:<bytes> obj:<g> trace:<g> metric:<m>:<0|1>
#include <Message_AlertExtended.hxx>
#include <Message_AttributeMeter.hxx>
#include <Message_AttributeObject.hxx>
#include <Message_AttributeStream.hxx>
#include <Message_CompositeAlerts.hxx>
#include <Message_Level.hxx>
#include <Message_Messenger.hxx>
#include <Message_Printer.hxx>
#include <Message_PrinterToReport.hxx>
#include <Message_Report.hxx>

#include <cctype>
#include <memory>

DEFINE_SIMPLE_ALERT(MsgProbe_AlertX)
DEFINE_SIMPLE_ALERT(MsgProbe_AlertY)

class MsgProbe_NoMerge : public Message_Alert
{
public:
  bool SupportsMerge() const override { return false; }
  DEFINE_STANDARD_RTTI_INLINE(MsgProbe_NoMerge, Message_Alert)
};

class MsgProbe_Refuse : public Message_Alert
{
public:
  bool Merge(const occ::handle<Message_Alert>&) override { return false; }
  DEFINE_STANDARD_RTTI_INLINE(MsgProbe_Refuse, Message_Alert)
};

//! Collects "<gravity>:<hex text>|" for every message it receives.
class MsgProbe_Printer : public Message_Printer
{
public:
  mutable std::string myLog;

protected:
  void send(const TCollection_AsciiString& theText, const Message_Gravity theGravity) const override
  {
    myLog += std::to_string(int(theGravity)) + ":" + msgp_ascii(theText) + "|";
  }
};

static occ::handle<Message_Attribute> msgp_attribute(const std::string& theKind)
{
  const TCollection_AsciiString aName(theKind.substr(1).c_str());
  switch (theKind[0])
  {
    case 'M': return new Message_AttributeMeter(aName);
    case 'S':
    {
      Standard_SStream aStream;
      aStream << "\"Key\": 1";
      return new Message_AttributeStream(aStream, aName);
    }
    case 'O': return new Message_AttributeObject(occ::handle<Standard_Transient>(), aName);
    default: return new Message_Attribute(aName);
  }
}

static occ::handle<Message_Alert> msgp_alert(const std::string& theKind)
{
  switch (theKind[0])
  {
    case 'P': return new Message_Alert();
    case 'X': return new MsgProbe_AlertX();
    case 'Y': return new MsgProbe_AlertY();
    case 'N': return new MsgProbe_NoMerge();
    case 'R': return new MsgProbe_Refuse();
    default:
    {
      occ::handle<Message_AlertExtended> anAlert = new Message_AlertExtended();
      anAlert->SetAttribute(msgp_attribute(theKind));
      return anAlert;
    }
  }
}

static occ::handle<Standard_Type> msgp_kind_type(char theKind)
{
  switch (theKind)
  {
    case 'P': return STANDARD_TYPE(Message_Alert);
    case 'X': return STANDARD_TYPE(MsgProbe_AlertX);
    case 'Y': return STANDARD_TYPE(MsgProbe_AlertY);
    case 'N': return STANDARD_TYPE(MsgProbe_NoMerge);
    case 'R': return STANDARD_TYPE(MsgProbe_Refuse);
    default: return STANDARD_TYPE(Message_AlertExtended);
  }
}

static void msgp_walk(std::ostream&                               out,
                      const occ::handle<Message_CompositeAlerts>& theAlerts,
                      const occ::handle<Message_Report>&          theReport,
                      int                                         theDepth);

//! One alert list: depth, gravity, type, key, meter flags, then the children.
static void msgp_walk_list(std::ostream&                                       out,
                           const NCollection_List<occ::handle<Message_Alert>>& theList,
                           int                                                 theGravity,
                           const occ::handle<Message_Report>&                  theReport,
                           int                                                 theDepth)
{
  for (NCollection_List<occ::handle<Message_Alert>>::Iterator anIt(theList); anIt.More(); anIt.Next())
  {
    const occ::handle<Message_Alert>& anAlert = anIt.Value();
    out << ' ' << theDepth << theGravity << ':' << anAlert->DynamicType()->Name() << ':'
        << msgp_hex(anAlert->GetMessageKey());
    occ::handle<Message_AlertExtended> anExtended = occ::down_cast<Message_AlertExtended>(anAlert);
    if (anExtended.IsNull())
    {
      continue;
    }
    occ::handle<Message_AttributeMeter> aMeter = occ::down_cast<Message_AttributeMeter>(anExtended->Attribute());
    for (NCollection_IndexedMap<Message_MetricType>::Iterator aMetric(theReport->ActiveMetrics()); !aMeter.IsNull() && aMetric.More(); aMetric.Next())
    {
      out << ':' << aMeter->HasMetric(aMetric.Value()) << aMeter->IsMetricValid(aMetric.Value());
    }
    msgp_walk(out, anExtended->CompositeAlerts(), theReport, theDepth + 1);
  }
}

static void msgp_walk(std::ostream&                               out,
                      const occ::handle<Message_CompositeAlerts>& theAlerts,
                      const occ::handle<Message_Report>&          theReport,
                      int                                         theDepth)
{
  for (int aGravity = Message_Trace; !theAlerts.IsNull() && aGravity <= Message_Fail; ++aGravity)
  {
    msgp_walk_list(out, theAlerts->Alerts((Message_Gravity)aGravity), aGravity, theReport, theDepth);
  }
}

static void msgp_walk_report(std::ostream& out, const occ::handle<Message_Report>& theReport)
{
  out << " A:";
  for (int aGravity = Message_Trace; aGravity <= Message_Fail; ++aGravity)
  {
    msgp_walk_list(out, theReport->GetAlerts((Message_Gravity)aGravity), aGravity, theReport, 0);
  }
}

static void msgp_report_state(std::ostream& out, const occ::handle<Message_Report>& theReport)
{
  for (int aGravity = Message_Trace; aGravity <= Message_Fail; ++aGravity)
  {
    out << ' ' << theReport->GetAlerts((Message_Gravity)aGravity).Extent();
  }
  std::ostringstream aDump;
  theReport->Dump(aDump);
  Standard_SStream aJson;
  theReport->DumpJson(aJson);
  occ::handle<MsgProbe_Printer>   aPrinter   = new MsgProbe_Printer();
  occ::handle<Message_Messenger>  aMessenger = new Message_Messenger(aPrinter);
  aPrinter->SetTraceLevel(Message_Trace);
  theReport->SendMessages(aMessenger);
  out << " D:" << msgp_hex(aDump.str()) << " J:" << msgp_hex(aJson.str()) << " S:" << msgp_hex(aPrinter->myLog);
}

static bool msgp_report_step(const occ::handle<Message_Report>& theReport,
                             occ::handle<Message_Report>&       theMerged,
                             occ::handle<Message_Alert>&        theLast,
                             const std::vector<std::string>&    theStep,
                             std::ostream&                      out)
{
  const std::string& anOp = theStep[0];
  const Message_Gravity aGravity = theStep.size() > 1 && std::isdigit(theStep[1][0]) ? (Message_Gravity)std::stoi(theStep[1]) : Message_Info;
  if (anOp == "add") { theLast = msgp_alert(theStep[2]); theReport->AddAlert(aGravity, theLast); }
  else if (anOp == "ext") theLast = Message_AlertExtended::AddAlert(theReport, msgp_attribute(theStep[2]), aGravity);
  else if (anOp == "nest")
  {
    occ::handle<Message_AlertExtended> aParent = occ::down_cast<Message_AlertExtended>(theLast);
    out << " n" << (!aParent.IsNull() && aParent->CompositeAlerts(true)->AddAlert(aGravity, msgp_alert(theStep[2])));
  }
  else if (anOp == "limit") theReport->SetLimit(std::stoi(theStep[1]));
  else if (anOp == "clear") theReport->Clear();
  else if (anOp == "clearg") theReport->Clear(aGravity);
  else if (anOp == "cleart") theReport->Clear(msgp_kind_type(theStep[1][0]));
  else if (anOp == "metric") theReport->SetActiveMetric((Message_MetricType)std::stoi(theStep[1]), theStep[2] == "1");
  else if (anOp == "clearmetrics") theReport->ClearMetrics();
  else if (anOp == "has")
  {
    out << " h" << theReport->HasAlert(msgp_kind_type(theStep[1][0]));
    for (int aG = Message_Trace; aG <= Message_Fail; ++aG) out << theReport->HasAlert(msgp_kind_type(theStep[1][0]), (Message_Gravity)aG);
  }
  else if (anOp == "merge")
  {
    if (theMerged.IsNull()) theMerged = new Message_Report();
    if (theStep[1] == "a") theMerged->Merge(theReport);
    else theMerged->Merge(theReport, aGravity);
  }
  else return false;
  return true;
}

static void msgp_report_run(std::istream& in, std::ostream& out)
{
  occ::handle<Message_Report> aReport = new Message_Report(), aMerged;
  occ::handle<Message_Alert>  aLast;
  for (const std::string& aStep : msgp_rest(in))
  {
    msgp_report_step(aReport, aMerged, aLast, msgp_fields(aStep), out);
  }
  msgp_walk_report(out, aReport);
  msgp_report_state(out, aReport);
  out << " metrics";
  for (NCollection_IndexedMap<Message_MetricType>::Iterator aMetric(aReport->ActiveMetrics()); aMetric.More(); aMetric.Next())
  {
    out << ':' << int(aMetric.Value());
  }
  if (!aMerged.IsNull())
  {
    out << " merged";
    msgp_report_state(out, aMerged);
  }
}

static void msgp_level_step(std::vector<std::unique_ptr<Message_Level>>& theLevels,
                            const std::vector<std::string>&              theStep)
{
  const occ::handle<Message_Report>& aReport = Message::DefaultReport(true);
  const std::string& anOp = theStep[0];
  const Message_Gravity aGravity = theStep.size() > 1 && std::isdigit(theStep[1][0]) ? (Message_Gravity)std::stoi(theStep[1]) : Message_Info;
  if (anOp == "open") theLevels.push_back(std::make_unique<Message_Level>(TCollection_AsciiString(theStep[1].c_str())));
  else if (anOp == "close" && !theLevels.empty()) theLevels.pop_back();
  else if (anOp == "add") aReport->AddAlert(aGravity, msgp_alert(theStep[2]));
  else if (anOp == "levadd" && !theLevels.empty()) theLevels.back()->AddAlert(aGravity, msgp_alert(theStep[2]));
  else if (anOp == "send") Message::DefaultMessenger()->Send(TCollection_AsciiString(msgp_bytes(theStep[2]).c_str()), aGravity);
  else if (anOp == "stream")
  {
    Standard_SStream aStream;
    aStream << msgp_bytes(theStep[2]);
    Message::DefaultMessenger()->Send(aStream, aGravity);
  }
  else if (anOp == "metric") aReport->SetActiveMetric((Message_MetricType)std::stoi(theStep[1]), theStep[2] == "1");
}

static void msgp_level_run(std::istream& in, std::ostream& out)
{
  // The default messenger keeps no console printer: nothing may reach the probe's stdout.
  Message::DefaultMessenger()->ChangePrinters().Clear();
  const occ::handle<Message_Report>& aReport = Message::DefaultReport(true);
  aReport->Clear();
  aReport->ClearMetrics();
  aReport->UpdateActiveInMessenger();
  aReport->ActivateInMessenger(true);
  {
    std::vector<std::unique_ptr<Message_Level>> aLevels;
    for (const std::string& aStep : msgp_rest(in))
    {
      msgp_level_step(aLevels, msgp_fields(aStep));
    }
    while (!aLevels.empty())
    {
      aLevels.pop_back();
    }
  }
  msgp_walk_report(out, aReport);
  std::ostringstream aDump;
  aReport->Dump(aDump);
  out << " D:" << msgp_hex(aDump.str());
  aReport->ActivateInMessenger(false);
  out << ' ' << aReport->IsActiveInMessenger() << Message::DefaultMessenger()->Printers().Size();
  aReport->Clear();
  aReport->ClearMetrics();
}

static void msgp_ptr_step(const occ::handle<Message_PrinterToReport>& thePrinter,
                          const occ::handle<Message_Report>&          theReport,
                          const std::vector<std::string>&             theStep)
{
  const std::string& anOp = theStep[0];
  const Message_Gravity aGravity = theStep.size() > 1 && std::isdigit(theStep[1][0]) ? (Message_Gravity)std::stoi(theStep[1]) : Message_Info;
  const std::string aText = theStep.size() > 2 ? theStep[2] : std::string("-");
  if (anOp == "send") thePrinter->Send(TCollection_AsciiString(msgp_bytes(aText).c_str()), aGravity);
  else if (anOp == "cstr") thePrinter->Send(msgp_bytes(aText).c_str(), aGravity);
  else if (anOp == "ext") thePrinter->Send(msgp_ext(aText), aGravity);
  else if (anOp == "stream")
  {
    Standard_SStream aStream;
    aStream << msgp_bytes(aText);
    thePrinter->SendStringStream(aStream, aGravity);
  }
  else if (anOp == "obj") thePrinter->SendObject(occ::handle<Standard_Transient>(), aGravity);
  else if (anOp == "trace") thePrinter->SetTraceLevel(aGravity);
  else if (anOp == "metric") theReport->SetActiveMetric((Message_MetricType)std::stoi(theStep[1]), theStep[2] == "1");
}

static void msgp_ptr_run(std::istream& in, std::ostream& out)
{
  occ::handle<Message_Report>          aReport  = new Message_Report();
  occ::handle<Message_PrinterToReport> aPrinter = new Message_PrinterToReport();
  aPrinter->SetReport(aReport);
  for (const std::string& aStep : msgp_rest(in))
  {
    msgp_ptr_step(aPrinter, aReport, msgp_fields(aStep));
  }
  msgp_walk_report(out, aReport);
  std::ostringstream aDump;
  aReport->Dump(aDump);
  out << " D:" << msgp_hex(aDump.str()) << ' ' << aPrinter->GetTraceLevel();
}

static bool message_report_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "report_run") msgp_report_run(in, out);
  else if (op == "level_run") msgp_level_run(in, out);
  else if (op == "ptr_run") msgp_ptr_run(in, out);
  else return false;
  return true;
}
