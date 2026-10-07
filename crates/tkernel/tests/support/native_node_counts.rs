//! Native map probes print, beside each callback, how many nodes the map's arena holds (an `a`
//! suffix after a numeric event, such as `K7a1`) and a live-node column. Per-entry node
//! allocation is protected allocator state (plan D1 class B6): slot storage allocates no node,
//! so rows are compared with those counts removed and every other column kept.

/// Removes every `a<count>` suffix that follows a digit or a `:` id, keeping the event order.
///
/// ```ignore
/// assert_eq!(without_node_counts("x K7a1,E4:1a0,"), "x K7,E4:1,");
/// ```
pub fn without_node_counts(row: &str) -> String {
    let characters: Vec<char> = row.chars().collect();
    let mut kept = String::with_capacity(row.len());
    let mut index = 0;
    while index < characters.len() {
        if is_count_suffix(&characters, index) {
            index = skip_count(&characters, index + 1);
            continue;
        }
        kept.push(characters[index]);
        index += 1;
    }
    kept
}

/// Replaces one space-separated column (0 is the label) with `*`.
pub fn mask_column(row: &str, column: usize) -> String {
    let mut fields: Vec<&str> = row.split(' ').collect();
    if let Some(field) = fields.get_mut(column) {
        *field = "*";
    }
    fields.join(" ")
}

fn is_count_suffix(characters: &[char], index: usize) -> bool {
    let previous_is_id = index > 0 && characters[index - 1].is_ascii_digit();
    let next_is_count = characters
        .get(index + 1)
        .is_some_and(|next| next.is_ascii_digit() || *next == '-');
    characters[index] == 'a' && previous_is_id && next_is_count
}

fn skip_count(characters: &[char], mut index: usize) -> usize {
    if characters.get(index) == Some(&'-') {
        index += 1;
    }
    while characters.get(index).is_some_and(char::is_ascii_digit) {
        index += 1;
    }
    index
}
