// Rust port of DataExchange/TKXSBase/Interface/Interface_MSG.cxx (Intervalled, dates,
// blanks and Print) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text helpers of `Interface_MSG`: rounding to interval limits, dates as texts,
//! and blank padding (at most 76 blanks, the native `blank` buffer).
use super::InterfaceMsg;
use rustycad_tkernel::{
    osd::Process,
    standard::{
        Failure,
        c_string::{FormatArg, sprintf_into},
    },
};
use std::io::{self, Write};

/// The length of the native `blank` buffer.
const MAX_BLANK: i32 = 76;

/// The interval limits of `Intervalled` for one order: (threshold, lower, upper); the
/// last row applies above every threshold.
fn interval_table(order: i32) -> &'static [(f64, f64, f64)] {
    match order {
        i32::MIN..=1 => &[(f64::INFINITY, 1.0, 10.0)],
        2 => &[(3.0, 1.0, 3.0), (f64::INFINITY, 3.0, 10.0)],
        3 => &[(2.0, 1.0, 2.0), (5.0, 2.0, 5.0), (f64::INFINITY, 5.0, 10.0)],
        4 => &[
            (2.0, 1.0, 2.0),
            (3.0, 2.0, 3.0),
            (6.0, 3.0, 6.0),
            (f64::INFINITY, 6.0, 10.0),
        ],
        5 | 6 => &[
            (1.5, 1.0, 1.5),
            (2.0, 1.5, 2.0),
            (3.0, 2.0, 3.0),
            (5.0, 3.0, 5.0),
            (7.0, 5.0, 7.0),
            (f64::INFINITY, 7.0, 10.0),
        ],
        _ => &[
            (1.2, 1.0, 1.2),
            (1.5, 1.2, 1.5),
            (2.0, 1.5, 2.0),
            (2.5, 2.0, 2.5),
            (3.0, 2.5, 3.0),
            (4.0, 3.0, 4.0),
            (5.0, 4.0, 5.0),
            (6.0, 5.0, 6.0),
            (8.0, 6.0, 8.0),
            (f64::INFINITY, 8.0, 10.0),
        ],
    }
}

/// The power of ten `bl` below `vl` (natively at most 200 steps), or `None` for 0.
fn decade(vl: f64) -> Option<f64> {
    let mut bl = 1.0;
    if vl >= 1.0 {
        let mut bu = 10.0;
        for _ in 0..200 {
            if vl < bu {
                break;
            }
            bl = bu;
            bu *= 10.0;
        }
        return Some(bl);
    }
    bl = 0.1;
    for _ in 0..200 {
        if vl >= bl {
            break;
        }
        bl /= 10.0;
    }
    if vl == 0.0 { None } else { Some(bl) }
}

impl InterfaceMsg {
    /// `Intervalled(val, order = 3, upper = false)`: `val` rounded down (or up) to an
    /// interval limit of a geometric progression (`order` limits per decade).
    pub fn intervalled(val: f64, order: i32, upper: bool) -> f64 {
        let vl = val.abs();
        let Some(bl) = decade(vl) else { return 0.0 };
        let rst = vl / bl;
        let table = interval_table(order);
        let row = table
            .iter()
            .find(|(threshold, _, _)| rst <= *threshold)
            .unwrap_or(&table[table.len() - 1]);
        let rst = if upper { row.2 } else { row.1 };
        if val < 0.0 { -(bl * rst) } else { bl * rst }
    }
    /// `TDate(text, yy, mm, dd, hh, mn, ss, format = "")`: "YYYY-MM-DD:HH-MN-SS", or the
    /// C format after "C:"; with a null year and a non-null second, the leading null
    /// fields take the system date. Another format leaves `text` unchanged.
    #[allow(clippy::too_many_arguments)]
    pub fn t_date(
        text: &mut Vec<u8>,
        yy: i32,
        mm: i32,
        dd: i32,
        hh: i32,
        mn: i32,
        ss: i32,
        format: &str,
    ) -> Result<(), Failure> {
        let fields = completed_date([yy, mm, dd, hh, mn, ss])?;
        let form: &[u8] = if format.is_empty() {
            b"%4.4d-%2.2d-%2.2d:%2.2d-%2.2d-%2.2d"
        } else if format.len() >= 2 && matches!(&format.as_bytes()[..2], b"c:" | b"C:") {
            &format.as_bytes()[2..]
        } else {
            return Ok(());
        };
        let args = fields.map(FormatArg::Int);
        sprintf_into(text, form, &args)
            .map_err(|e| Failure::new(&format!("Interface_MSG::TDate: format {format:?}: {e}")))?;
        text.pop(); // the terminating NUL
        Ok(())
    }
    /// `NDate(text, yy, mm, dd, hh, mn, ss)`: adds the digits of the six numeric fields
    /// of `text` to the values (natively in-out); always true.
    pub fn n_date(text: &str, fields: &mut [i32; 6]) -> bool {
        let mut num = 1;
        for byte in text.bytes() {
            if byte.is_ascii_digit() {
                if num & 1 == 0 {
                    num += 1;
                }
                if (1..=11).contains(&num) {
                    let slot = &mut fields[((num - 1) / 2) as usize];
                    *slot = slot.wrapping_mul(10).wrapping_add(i32::from(byte - b'0'));
                }
            } else if num & 1 != 0 {
                num += 1;
            }
        }
        num > 0
    }
    /// `CDate(text1, text2)`: 0 equal, -1 when `text1` is before, 1 after.
    pub fn c_date(text1: &str, text2: &str) -> i32 {
        let (mut first, mut second) = ([0; 6], [0; 6]);
        if !Self::n_date(text1, &mut first) || !Self::n_date(text2, &mut second) {
            return 0;
        }
        match first.cmp(&second) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Equal => 0,
        }
    }
    /// `Blanks(val, max)`: the blanks that complete the printing of the integer `val`
    /// (one place less for a negative value) up to `max` places.
    pub fn blanks_for_integer(val: i32, max: i32) -> String {
        if val < 0 {
            return Self::blanks_for_integer(val.wrapping_neg(), max - 1);
        }
        let digits = (1..=9)
            .find(|d| i64::from(val) < 10_i64.pow(*d))
            .unwrap_or(10) as i32;
        Self::blanks(10 - digits + max - 10)
    }
    /// `Blanks(val, max)`: the blanks that complete the text `val` up to `max` places.
    pub fn blanks_for_text(val: &str, max: i32) -> String {
        let lng = val.len() as i32;
        if lng > MAX_BLANK || lng > max {
            return String::new();
        }
        Self::blanks(max - lng)
    }
    /// `Blanks(count)`: `count` blanks, between 0 and 76.
    pub fn blanks(count: i32) -> String {
        " ".repeat(count.clamp(0, MAX_BLANK) as usize)
    }
    /// `Print(S, val, max, just = -1)`: `val` padded to `max` places (at most 76), left
    /// (-1), centered (0) or right (1) justified; a longer text is printed alone.
    pub fn print(stream: &mut dyn Write, val: &str, max: i32, just: i32) -> io::Result<()> {
        let max = max.min(MAX_BLANK);
        let lng = val.len() as i32;
        if lng > max {
            return stream.write_all(val.as_bytes());
        }
        let m1 = (max - lng) / 2;
        let m2 = max - lng - m1;
        let text = match just {
            j if j < 0 => format!("{val}{}", Self::blanks(m1 + m2)),
            0 => format!("{}{val}{}", Self::blanks(m1), Self::blanks(m2)),
            _ => format!("{}{val}", Self::blanks(m1 + m2)),
        };
        stream.write_all(text.as_bytes())
    }
}

/// The date fields of `TDate`, completed with the system date as natively.
fn completed_date(mut fields: [i32; 6]) -> Result<[i32; 6], Failure> {
    if fields[0] != 0 || fields[5] == 0 {
        return Ok(fields);
    }
    let date = Process::new()
        .system_date()
        .map_err(|e| Failure::new(&format!("Interface_MSG::TDate: system date: {e:?}")))?;
    let now = [
        date.year(),
        date.month(),
        date.day(),
        date.hour(),
        date.minute(),
        date.second(),
    ];
    // The leading null fields are completed (the year always); the second only when the
    // five first fields were null, as the nested native tests.
    let null_prefix = fields.iter().take(5).take_while(|v| **v == 0).count();
    let completed = if null_prefix == 5 { 6 } else { null_prefix };
    fields[..completed].copy_from_slice(&now[..completed]);
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intervals_by_order() {
        assert_eq!(InterfaceMsg::intervalled(4.0, 3, false), 2.0);
        assert_eq!(InterfaceMsg::intervalled(4.0, 3, true), 5.0);
        assert_eq!(InterfaceMsg::intervalled(-0.042, 2, false), -0.03);
        assert_eq!(InterfaceMsg::intervalled(370.0, 10, true), 400.0);
        assert_eq!(InterfaceMsg::intervalled(0.0, 3, true), 0.0);
        assert_eq!(InterfaceMsg::intervalled(55.0, 0, true), 100.0);
        assert_eq!(InterfaceMsg::intervalled(6.5, 6, false), 5.0);
    }

    #[test]
    fn dates() {
        let mut text = b"keep".to_vec();
        InterfaceMsg::t_date(&mut text, 2024, 1, 2, 3, 4, 5, "").unwrap();
        assert_eq!(text, b"2024-01-02:03-04-05");
        InterfaceMsg::t_date(&mut text, 2024, 1, 2, 3, 4, 5, "C:%d/%d").unwrap();
        assert_eq!(text, b"2024/1");
        InterfaceMsg::t_date(&mut text, 1, 1, 1, 1, 1, 1, "x").unwrap();
        assert_eq!(text, b"2024/1");
        let mut fields = [0; 6];
        assert!(InterfaceMsg::n_date("2024-01-02:03-04-05", &mut fields));
        assert_eq!(fields, [2024, 1, 2, 3, 4, 5]);
        assert_eq!(InterfaceMsg::c_date("2024-01-02", "2024-01-03"), -1);
        assert_eq!(InterfaceMsg::c_date("2025", "2024-12-31"), 1);
        assert_eq!(InterfaceMsg::c_date("2024-1-2", "2024-01-02"), 0);
        let mut now = Vec::new();
        InterfaceMsg::t_date(&mut now, 0, 0, 0, 0, 0, 7, "").unwrap();
        assert_eq!(now.len(), 19);
    }

    #[test]
    fn blanks_and_print() {
        assert_eq!(InterfaceMsg::blanks_for_integer(5, 4), "   ");
        assert_eq!(InterfaceMsg::blanks_for_integer(-5, 4), "  ");
        assert_eq!(InterfaceMsg::blanks_for_integer(12345, 3), "");
        assert_eq!(InterfaceMsg::blanks_for_text("ab", 5), "   ");
        assert_eq!(InterfaceMsg::blanks_for_text("abcdef", 5), "");
        assert_eq!(InterfaceMsg::blanks(100).len(), 76);
        let mut out = Vec::new();
        InterfaceMsg::print(&mut out, "ab", 7, 0).unwrap();
        InterfaceMsg::print(&mut out, "cd", 4, 1).unwrap();
        InterfaceMsg::print(&mut out, "ef", 3, -1).unwrap();
        InterfaceMsg::print(&mut out, "toolong", 3, -1).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "  ab     cdef toolong");
    }
}
