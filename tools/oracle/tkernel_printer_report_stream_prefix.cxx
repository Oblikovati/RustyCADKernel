// Pinned OCCT 3d097a0: C-string stream names and the protected meter hook.
#include <Message_AlertExtended.hxx>
#include <Message_Attribute.hxx>
#include <Message_PrinterToReport.hxx>
#include <Message_Report.hxx>
#include <iostream>
#include <sstream>

struct MeterPrinterProbe : Message_PrinterToReport
{
  using Message_PrinterToReport::sendMetricAlert;
};

void sendBytes(MeterPrinterProbe& printer, const char* bytes, std::size_t size)
{
  std::stringstream stream;
  stream.write(bytes, static_cast<std::streamsize>(size));
  printer.SendStringStream(stream, Message_Info);
}

void printAlerts(const occ::handle<Message_Report>& report)
{
  for (const auto& alert : report->GetAlerts(Message_Info))
  {
    auto extended = occ::down_cast<Message_AlertExtended>(alert);
    const auto& attribute = extended->Attribute();
    std::cout << attribute->DynamicType()->Name() << ':' << attribute->GetName().Length()
              << ':' << attribute->GetName().ToCString() << '\n';
  }
  report->Clear();
}

int main()
{
  occ::handle<Message_Report> report = new Message_Report();
  MeterPrinterProbe printer;
  printer.SetReport(report);
  const char separatorAfterNul[] = "name\0\"hidden\": 1";
  sendBytes(printer, separatorAfterNul, sizeof(separatorAfterNul) - 1);
  if (!report->GetAlerts(Message_Info).IsEmpty()) return 1;
  sendBytes(printer, "\"Key\": 1", 8);
  printAlerts(report);
  report->SetActiveMetric(Message_MetricType_WallClock, true);
  const char meterName[] = "meter\0hidden";
  sendBytes(printer, meterName, sizeof(meterName) - 1);
  printAlerts(report);
  report->ClearMetrics();
  const char pendingName[] = "name\0hidden";
  sendBytes(printer, pendingName, sizeof(pendingName) - 1);
  printer.Send(TCollection_AsciiString("plain"), Message_Info);
  printAlerts(report);
  sendBytes(printer, "pending", 7);
  printer.sendMetricAlert(TCollection_AsciiString("direct"), Message_Info);
  printer.Send(TCollection_AsciiString("plain"), Message_Info);
  printAlerts(report);
}
