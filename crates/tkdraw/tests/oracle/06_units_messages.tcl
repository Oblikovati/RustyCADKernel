# Draw_UnitCommands and Draw_MessageCommands.
puts "unit: [unit 1 m mm] [unit 2.5 in mm] [unit 180 deg rad]"
puts "unit args: [catch {unit 1 m} message]"
puts "unitconvtoSI: [string trim [unitconvtoSI 1 mm]] / [string trim [unitconvtoSI 2 km]]"
puts "unitconvtoMDTV: [string trim [unitconvtoMDTV 1 m]]"
puts "unitparsing: [string trim [unitparsing kg/m/s]]"
puts "unitparsing args: [catch {unitparsing} message] <$message>"
SendMessage Hello World
puts "SendMessage args: [catch {SendMessage} message] <$message>"
puts "SetMessagePrinter args: [catch {SetMessagePrinter} message] <$message>"
puts "SetMessagePrinter bad: [catch {SetMessagePrinter -type nothing} message] <$message>"
puts "SetMessagePrinter bad arg: [catch {SetMessagePrinter -bogus} message] <$message>"
SetMessagePrinter -type draw
puts "draw printer: <[SendMessage to the result]>"
SetMessagePrinter -type draw -state off
puts "ClearReport: [catch {ClearReport} message] <$message>"
puts "SetReportMetric bad: [catch {SetReportMetric 99} message] <$message>"
puts "SetReportMetric: [SetReportMetric 1 5]"
puts "CollectMetricMessages off: [catch {CollectMetricMessages -activate 0} message] <$message>"
puts "CollectMetricMessages: [CollectMetricMessages -activate 1] [catch {CollectMetricMessages -activate 1} message] <$message>"
CollectMetricMessages -activate 0
puts "ClearReport after: [ClearReport]"
puts "PrintReport: <[PrintReport -dump]>"
