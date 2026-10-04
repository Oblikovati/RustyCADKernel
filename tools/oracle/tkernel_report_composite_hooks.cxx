// Publicly expose native protected hooks and verify callback reads at OCCT 3d097a0.
#include <Message_AlertExtended.hxx>
#include <Message_Attribute.hxx>
#include <Message_CompositeAlerts.hxx>
#include <Message_Messenger.hxx>
#include <Message_Printer.hxx>
#include <Message_Report.hxx>
#include <iostream>
#include <sstream>
#include <stdexcept>

struct ReportOutputProbe : Message_Report
{
  using Message_Report::dumpMessages;
  using Message_Report::sendMessages;
};

struct InspectingPrinter : Message_Printer
{
  explicit InspectingPrinter(const Message_Report& report) : myReport(report) {}
  mutable std::string keys;
  const Message_Report& myReport;
  int expectedCount = 0;
  void send(const TCollection_AsciiString& text, Message_Gravity gravity) const override
  {
    if (myReport.GetAlerts(gravity).Size() != expectedCount) throw std::logic_error("unexpected report alert count");
    keys += text.ToCString();
    keys += '\n';
  }
};

occ::handle<Message_AlertExtended> namedAlert(const char* name)
{
  occ::handle<Message_AlertExtended> alert = new Message_AlertExtended();
  alert->SetAttribute(new Message_Attribute(TCollection_AsciiString(name)));
  return alert;
}

occ::handle<Message_CompositeAlerts> externalHierarchy()
{
  occ::handle<Message_CompositeAlerts> composite = new Message_CompositeAlerts();
  auto parent = namedAlert("parent");
  parent->CompositeAlerts(true)->AddAlert(Message_Info, namedAlert("child"));
  parent->CompositeAlerts(true)->AddAlert(Message_Warning, namedAlert("hidden"));
  composite->AddAlert(Message_Info, parent);
  return composite;
}

struct AppendingPrinter : Message_Printer
{
  explicit AppendingPrinter(Message_Report& report) : myReport(report) {}
  Message_Report& myReport;
  mutable std::string keys;
  mutable bool appended = false;
  const char* trigger = "first";
  occ::handle<Message_CompositeAlerts> external;
  occ::handle<Message_Alert> removed;
  void send(const TCollection_AsciiString& text, Message_Gravity gravity) const override
  {
    keys += text.ToCString();
    keys += '\n';
    if (appended || text != trigger) return;
    appended = true;
    if (!external.IsNull())
    {
      external->RemoveAlert(gravity, removed);
      external->AddAlert(gravity, namedAlert("late"));
      return;
    }
    myReport.AddAlert(gravity, namedAlert("late"));
  }
};

bool inspectAppendingDelivery()
{
  Message_Report report;
  report.AddAlert(Message_Info, namedAlert("first"));
  occ::handle<AppendingPrinter> printer = new AppendingPrinter(report);
  occ::handle<Message_Messenger> messenger = new Message_Messenger(printer);
  report.SendMessages(messenger, Message_Info);
  return printer->keys == "first\nlate\n";
}

bool inspectEarlierRemoval()
{
  ReportOutputProbe report;
  occ::handle<AppendingPrinter> printer = new AppendingPrinter(report);
  printer->external = new Message_CompositeAlerts();
  printer->removed = namedAlert("first");
  printer->trigger = "second";
  printer->external->AddAlert(Message_Info, printer->removed);
  printer->external->AddAlert(Message_Info, namedAlert("second"));
  printer->external->AddAlert(Message_Info, namedAlert("third"));
  report.sendMessages(new Message_Messenger(printer), Message_Info, printer->external);
  return printer->keys == "first\nsecond\nthird\nlate\n";
}

struct AppendingBuffer : std::stringbuf
{
  explicit AppendingBuffer(Message_Report& report) : myReport(report) {}
  Message_Report& myReport;
  bool appended = false;
  std::streamsize xsputn(const char* text, std::streamsize length) override
  {
    const auto result = std::stringbuf::xsputn(text, length);
    if (!appended)
    {
      appended = true;
      myReport.AddAlert(Message_Info, namedAlert("late"));
    }
    return result;
  }
};

bool inspectAppendingDump()
{
  Message_Report report;
  report.AddAlert(Message_Info, namedAlert("first"));
  AppendingBuffer buffer(report);
  std::ostream stream(&buffer);
  report.Dump(stream, Message_Info);
  return buffer.str() == "first\nlate\n";
}

bool inspectPublicDelivery(ReportOutputProbe& report, const occ::handle<InspectingPrinter>& printer,
                          const occ::handle<Message_Messenger>& messenger)
{
  report.AddAlert(Message_Info, namedAlert("owned"));
  printer->expectedCount = 1;
  printer->keys.clear();
  report.SendMessages(messenger, Message_Info);
  return printer->keys == "owned\n";
}

int main()
{
  ReportOutputProbe report;
  auto composite = externalHierarchy();
  occ::handle<InspectingPrinter> printer = new InspectingPrinter(report);
  occ::handle<Message_Messenger> messenger = new Message_Messenger(printer);
  std::ostringstream output;
  report.dumpMessages(output, Message_Info, composite);
  report.sendMessages(messenger, Message_Info, composite);
  report.dumpMessages(output, Message_Info, {});
  report.sendMessages(messenger, Message_Info, {});
  if (output.str() != printer->keys || !report.GetAlerts(Message_Info).IsEmpty()) return 1;
  if (!inspectPublicDelivery(report, printer, messenger)) return 2;
  if (!inspectAppendingDelivery()) return 3;
  if (!inspectEarlierRemoval()) return 4;
  if (!inspectAppendingDump()) return 5;
  std::cout << output.str();
}
