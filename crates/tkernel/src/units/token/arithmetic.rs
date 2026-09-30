//! Non-virtual token arithmetic: every result is a new plain `Units_Token`, so a
//! shifted operand loses its move, as natively.
use super::{Token, object};
use crate::standard::{Failure, Handle};
use crate::tcollection::AsciiString;

/// `"(" + left + ")" + operator + "(" + right + ")"`, the native composite spelling.
pub(super) fn composite(left: &AsciiString, operator: &[u8], right: &[u8]) -> Vec<u8> {
    let mut word = vec![b'('];
    word.extend_from_slice(left.as_bytes());
    word.push(b')');
    word.extend_from_slice(operator);
    word.push(b'(');
    word.extend_from_slice(right);
    word.push(b')');
    word
}

impl Token {
    /// Ignores the integer and returns a blank token (the native body is commented
    /// out); `token.add_integer(1)`.
    pub fn add_integer(&self, _integer: i32) -> Handle<Token> {
        Handle::new(Token::new())
    }
    /// Sums the factors when the dimensions agree, else returns `Token(" ")`;
    /// `token.add(&other)?`.
    pub fn add(&self, token: &Handle<Token>) -> Result<Handle<Token>, Failure> {
        self.additive(token, "Units_Token::Add", |a, b| a + b)
    }
    /// Subtracts the factors when the dimensions agree, else returns `Token(" ")`;
    /// `token.subtract(&other)?`.
    pub fn subtract(&self, token: &Handle<Token>) -> Result<Handle<Token>, Failure> {
        self.additive(token, "Units_Token::Subtract", |a, b| a - b)
    }
    fn additive(
        &self,
        token: &Handle<Token>,
        member: &str,
        combine: fn(f64, f64) -> f64,
    ) -> Result<Handle<Token>, Failure> {
        let other = object(token, member)?;
        let dimensions = self.dimensions();
        if !object(&dimensions, member)?.is_equal(&other.dimensions())? {
            return Ok(Handle::new(Token::with_word(b" ")));
        }
        let value = combine(self.value(), other.value());
        let word = self.word();
        Ok(Handle::new(Token::with_dimensions(
            word.as_bytes(),
            b" ",
            value,
            &dimensions,
        )))
    }
    /// `(a)*(b)` with multiplied factors and summed exponents; `token.multiply(&other)?`.
    pub fn multiply(&self, token: &Handle<Token>) -> Result<Handle<Token>, Failure> {
        let other = object(token, "Units_Token::Multiply")?;
        let word = composite(&self.word(), b"*", other.word().as_bytes());
        let dimensions =
            object(&self.dimensions(), "Units_Token::Multiply")?.multiply(&other.dimensions())?;
        let value = self.value() * other.value();
        Ok(Handle::new(Token::with_dimensions(
            &word,
            b" ",
            value,
            &dimensions,
        )))
    }
    /// `(a)/(b)` without the zero guard, which needs the handle (see
    /// `TokenHandle::divide`); `token.divided_by(&other)?`.
    pub(super) fn divided_by(&self, other: &Token) -> Result<Handle<Token>, Failure> {
        let word = composite(&self.word(), b"/", other.word().as_bytes());
        let dimensions =
            object(&self.dimensions(), "Units_Token::Divide")?.divide(&other.dimensions())?;
        let value = self.value() / other.value();
        Ok(Handle::new(Token::with_dimensions(
            &word,
            b" ",
            value,
            &dimensions,
        )))
    }
    /// `(a)**(b)` raising factor and exponents to the other token's value;
    /// `token.power_token(&other)?`.
    pub fn power_token(&self, token: &Handle<Token>) -> Result<Handle<Token>, Failure> {
        let other = object(token, "Units_Token::Power")?;
        let word = composite(&self.word(), b"**", other.word().as_bytes());
        self.raised(&word, other.value())
    }
    /// `(a)**(e)` where `e` is spelled with C `%g`; `token.power(2.0)?`.
    pub fn power(&self, exponent: f64) -> Result<Handle<Token>, Failure> {
        let spelled = AsciiString::from_real(exponent);
        let word = composite(&self.word(), b"**", spelled.as_bytes());
        self.raised(&word, exponent)
    }
    fn raised(&self, word: &[u8], exponent: f64) -> Result<Handle<Token>, Failure> {
        let dimensions = object(&self.dimensions(), "Units_Token::Power")?.power(exponent);
        let value = self.value().powf(exponent);
        Ok(Handle::new(Token::with_dimensions(
            word,
            b" ",
            value,
            &dimensions,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::super::Dimensions;
    use super::*;

    fn unit(word: &[u8], value: f64, dimensions: Handle<Dimensions>) -> Handle<Token> {
        Handle::new(Token::with_dimensions(word, b"U", value, &dimensions))
    }

    #[test]
    fn composite_words_values_and_dimensions() {
        let metre = unit(b"m", 1.0, Dimensions::a_length());
        let second = unit(b"s", 1.0, Dimensions::a_time());
        let km = unit(b"km", 1000.0, Dimensions::a_length());
        let speed = km.get().unwrap().multiply(&second).unwrap();
        assert_eq!(speed.get().unwrap().word(), "(km)*(s)");
        let sum = km.get().unwrap().add(&metre).unwrap();
        assert_eq!(sum.get().unwrap().value(), 1001.0);
        assert_eq!(sum.get().unwrap().mean(), " ");
        let mismatch = km.get().unwrap().subtract(&second).unwrap();
        assert_eq!(mismatch.get().unwrap().word(), " ");
        let squared = km.get().unwrap().power(2.0).unwrap();
        assert_eq!(squared.get().unwrap().word(), "(km)**(2)");
        assert_eq!(squared.get().unwrap().value(), 1e6);
        let cube = metre
            .get()
            .unwrap()
            .power_token(&unit(b"3", 3.0, Dimensions::a_less()));
        assert_eq!(
            cube.unwrap()
                .get()
                .unwrap()
                .dimensions()
                .get()
                .unwrap()
                .length(),
            3.0
        );
        assert_eq!(km.get().unwrap().add_integer(3).get().unwrap().word(), " ");
    }
}
