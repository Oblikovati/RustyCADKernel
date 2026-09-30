//! Report output: plain dump, messenger delivery and JSON dump.
use super::Report;
use crate::message::{AlertExtended, CompositeAlerts, Gravity, Messenger, downcast_alert, json};
use crate::standard::RuntimeTyped;
use crate::standard::dump::DumpStream;
use std::io::{self, Write};
use std::sync::Arc;

/// Child container of an extended alert, if the alert is one and has children.
fn children(alert: &crate::message::AlertHandle) -> Option<Arc<CompositeAlerts>> {
    downcast_alert::<AlertExtended>(alert)?.composite_alerts(false)
}

impl Report {
    /// Writes every message key, one per line, gravity by gravity; `report.dump(&mut output)?`.
    pub fn dump(&self, output: &mut dyn Write) -> io::Result<()> {
        for gravity in Gravity::ALL {
            self.dump_gravity(output, gravity)?;
        }
        Ok(())
    }
    /// Writes the message keys of one gravity, children after their parent;
    /// `report.dump_gravity(&mut output, Gravity::Warning)?`.
    pub fn dump_gravity(&self, output: &mut dyn Write, gravity: Gravity) -> io::Result<()> {
        match self.state().composite.clone() {
            Some(composite) => dump_messages(output, gravity, &composite),
            None => Ok(()),
        }
    }
    /// Sends every message key to a messenger, gravity by gravity; `report.send_messages(&messenger)`.
    pub fn send_messages(&self, messenger: &Messenger) {
        for gravity in Gravity::ALL {
            self.send_messages_gravity(messenger, gravity);
        }
    }
    /// Sends the message keys of one gravity; `report.send_messages_gravity(&messenger, g)`.
    pub fn send_messages_gravity(&self, messenger: &Messenger, gravity: Gravity) {
        if let Some(composite) = self.state().composite.clone() {
            send_messages(messenger, gravity, &composite);
        }
    }
    /// `DumpJson`: alerts, open level count, active metrics, limit and activation flag;
    /// `report.dump_json(&mut stream, -1)`.
    pub fn dump_json(&self, output: &mut DumpStream, depth: i32) {
        json::class_begin(output, Self::static_type().name());
        let (composite, levels) = {
            let state = self.state();
            (state.composite.clone(), state.levels.len())
        };
        if let Some(composite) = composite {
            json::field_dumped(output, depth, "CompositAlerts", |field, depth| {
                composite.dump_json(field, depth)
            });
        }
        json::field_integer(output, "AlertLevels", levels as i32);
        for (number, metric) in self.active_metrics().iter().enumerate() {
            json::field_integer(
                output,
                &format!("ActiveMetric{}", number + 1),
                *metric as i32,
            );
        }
        json::field_integer(output, "Limit", self.limit());
        json::field_integer(
            output,
            "IsActiveInMessenger",
            i32::from(self.is_active_in_messenger(None)),
        );
    }
}

fn dump_messages(
    output: &mut dyn Write,
    gravity: Gravity,
    composite: &CompositeAlerts,
) -> io::Result<()> {
    for alert in composite.alerts(gravity) {
        output.write_all(alert.get_message_key().as_bytes())?;
        output.write_all(b"\n")?;
        if let Some(children) = children(&alert) {
            dump_messages(output, gravity, &children)?;
        }
    }
    Ok(())
}

fn send_messages(messenger: &Messenger, gravity: Gravity, composite: &CompositeAlerts) {
    for alert in composite.alerts(gravity) {
        // `Send(const char*)`: the key as a C string.
        let key = alert.get_message_key();
        messenger.send_c_str(key.as_bytes(), gravity);
        if let Some(children) = children(&alert) {
            send_messages(messenger, gravity, &children);
        }
    }
}
