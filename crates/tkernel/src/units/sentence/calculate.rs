//! `CalculateLocal` (variant skl 15.09.2005): brackets first, then `**`, then the
//! other operators left to right, rewriting the sequence in place.
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle};
use crate::units::support::{
    CHANGE_VALUE, REMOVE_ONE, REMOVE_RANGE, edit, item, length, new_sequence, object,
};
use crate::units::token::{Token, TokenHandle};

type Tokens = Handle<HSequence<Handle<Token>>>;

fn word_is(token: &Handle<Token>, text: &str) -> Result<bool, Failure> {
    Ok(object(token, "Units_Sentence::CalculateLocal")?.word() == text)
}

/// Reduces `tokens` to its value token; a null handle when nothing reduces (the
/// stale or never-assigned native `tok1`).
pub(crate) fn calculate_local(tokens: &Tokens) -> Result<Handle<Token>, Failure> {
    match length(tokens)? {
        1 => item(tokens, 1),
        2 => unary(tokens),
        _ => reduce(tokens),
    }
}

/// A two-token sequence: drops a leading `+`, folds a leading `-` into a negated
/// plain token (a shifted operand loses its move).
fn unary(tokens: &Tokens) -> Result<Handle<Token>, Failure> {
    if word_is(&item(tokens, 1)?, "+")? {
        edit(tokens, REMOVE_ONE, |items| items.remove(1))?;
    }
    if word_is(&item(tokens, 1)?, "-")? {
        let operand = item(tokens, 2)?;
        let operand = object(&operand, "Units_Sentence::CalculateLocal")?;
        let word = [b"-".as_slice(), operand.word().as_bytes()].concat();
        let mean = operand.mean();
        // Natively `Value() * (-1.0)`: unlike negation it keeps a NaN operand's sign bit.
        #[allow(clippy::neg_multiply)]
        let value = operand.value() * (-1.0);
        let negated = Token::with_dimensions(&word, mean.as_bytes(), value, &operand.dimensions());
        edit(tokens, REMOVE_ONE, |items| items.remove(1))?;
        edit(tokens, CHANGE_VALUE, |items| {
            items.set_value(1, Handle::new(negated))
        })?;
    }
    item(tokens, 1)
}

fn reduce(tokens: &Tokens) -> Result<Handle<Token>, Failure> {
    let mut result = Handle::default();
    loop {
        match first_bracket(tokens)? {
            Some(open) => result = bracket(tokens, open)?,
            None => return operators(tokens, result),
        }
    }
}

fn first_bracket(tokens: &Tokens) -> Result<Option<usize>, Failure> {
    for index in 1..=length(tokens)? {
        if word_is(&item(tokens, index)?, "(")? {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

/// Evaluates the bracket opened at `open` and replaces it by its value; an
/// unclosed bracket makes the native `Remove(i + 1, j)` fail.
fn bracket(tokens: &Tokens, open: usize) -> Result<Handle<Token>, Failure> {
    let inner = new_sequence::<Handle<Token>>();
    let total = length(tokens)?;
    let (mut depth, mut close) = (1, total + 1);
    for index in open + 1..=total {
        let token = item(tokens, index)?;
        depth += i32::from(word_is(&token, "(")?) - i32::from(word_is(&token, ")")?);
        if depth <= 0 {
            close = index;
            break;
        }
        inner.get().unwrap().append(token);
    }
    let value = calculate_local(&inner)?;
    edit(tokens, REMOVE_RANGE, |items| {
        items.remove_range(open + 1, close)
    })?;
    edit(tokens, CHANGE_VALUE, |items| {
        items.set_value(open, value.clone())
    })?;
    Ok(value)
}

/// Step 1 folds `**` with `pow`, step 2 every other operator (meaning `O`): `/` with
/// `operator/`, anything else with `operator*`.
fn operators(tokens: &Tokens, result: Handle<Token>) -> Result<Handle<Token>, Failure> {
    let power = |token: &Handle<Token>| word_is(token, "**");
    let raise = |left: &Handle<Token>, right: &Handle<Token>, _: &Handle<Token>| {
        object(left, "pow")?.power_token(right)
    };
    let result = fold(tokens, result, &power, &raise)?;
    let other =
        |token: &Handle<Token>| Ok(object(token, "Units_Sentence::CalculateLocal")?.mean() == "O");
    let combine = |left: &Handle<Token>, right: &Handle<Token>, operator: &Handle<Token>| {
        if word_is(operator, "/")? {
            left.divide(right)
        } else {
            object(left, "operator*")?.multiply(right)
        }
    };
    fold(tokens, result, &other, &combine)
}

type Selector<'a> = &'a dyn Fn(&Handle<Token>) -> Result<bool, Failure>;
type Combiner<'a> =
    &'a dyn Fn(&Handle<Token>, &Handle<Token>, &Handle<Token>) -> Result<Handle<Token>, Failure>;

fn fold(
    tokens: &Tokens,
    mut result: Handle<Token>,
    selected: Selector<'_>,
    combine: Combiner<'_>,
) -> Result<Handle<Token>, Failure> {
    let mut index = 1;
    while index <= length(tokens)? {
        let operator = item(tokens, index)?;
        if selected(&operator)? {
            result = apply(tokens, index, &operator, combine)?;
            index -= 1;
        }
        index += 1;
    }
    Ok(result)
}

/// Replaces `left op right` (op at `index`) by its value, in the native order: read
/// both neighbours (index 0 or past the end is Standard_OutOfRange), combine, remove.
fn apply(
    tokens: &Tokens,
    index: usize,
    operator: &Handle<Token>,
    combine: Combiner<'_>,
) -> Result<Handle<Token>, Failure> {
    let left = item(tokens, index - 1)?;
    let right = item(tokens, index + 1)?;
    let value = combine(&left, &right, operator)?;
    edit(tokens, REMOVE_ONE, |items| items.remove(index))?;
    edit(tokens, REMOVE_ONE, |items| items.remove(index))?;
    edit(tokens, CHANGE_VALUE, |items| {
        items.set_value(index - 1, value.clone())
    })?;
    Ok(value)
}
