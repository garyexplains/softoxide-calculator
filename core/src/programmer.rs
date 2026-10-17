//! Exact fixed-width integer arithmetic; no conversion through f64.
use crate::expression::{error, lex, CalcError, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntegerConfig {
    pub base: u32,
    pub width: u32,
    pub signed: bool,
    pub arithmetic_shift: bool,
}
impl Default for IntegerConfig {
    fn default() -> Self {
        Self {
            base: 16,
            width: 64,
            signed: false,
            arithmetic_shift: false,
        }
    }
}
impl IntegerConfig {
    pub fn validate(self) -> Result<(), CalcError> {
        if ![2, 8, 10, 16].contains(&self.base) || ![8, 16, 32, 64].contains(&self.width) {
            return Err(error("Invalid integer base or width"));
        }
        Ok(())
    }
    pub fn mask(self) -> u64 {
        if self.width == 64 {
            u64::MAX
        } else {
            (1u64 << self.width) - 1
        }
    }
    pub fn signed_value(self, bits: u64) -> i128 {
        let bits = bits & self.mask();
        if bits & (1u64 << (self.width - 1)) != 0 {
            bits as i128 - (1i128 << self.width)
        } else {
            bits as i128
        }
    }
    pub fn format(self, bits: u64) -> String {
        let bits = bits & self.mask();
        match self.base {
            2 => format!("{bits:b}"),
            8 => format!("{bits:o}"),
            16 => format!("{bits:X}"),
            _ if self.signed => self.signed_value(bits).to_string(),
            _ => bits.to_string(),
        }
    }
}
pub fn binary(op: &str, a: u64, b: u64, c: IntegerConfig) -> Result<u64, CalcError> {
    c.validate()?;
    let a = a & c.mask();
    let b = b & c.mask();
    let n = match op {
        "+" => a.wrapping_add(b),
        "-" => a.wrapping_sub(b),
        "*" => a.wrapping_mul(b),
        "/" | "%" | "mod" if b == 0 => return Err(error("Cannot divide by zero")),
        "/" if c.signed => (c.signed_value(a) / c.signed_value(b)) as u64,
        "/" => a / b,
        "%" | "mod" if c.signed => (c.signed_value(a) % c.signed_value(b)) as u64,
        "%" | "mod" => a % b,
        "&" | "and" => a & b,
        "|" | "or" => a | b,
        "^" | "xor" => a ^ b,
        "nor" => !(a | b),
        "<<" | "shl" => {
            if b >= c.width as u64 {
                0
            } else {
                a << b
            }
        }
        ">>" | "shr" if c.arithmetic_shift => {
            if b >= c.width as u64 {
                if c.signed_value(a) < 0 {
                    c.mask()
                } else {
                    0
                }
            } else {
                (c.signed_value(a) >> b) as u64
            }
        }
        ">>" | "shr" => {
            if b >= c.width as u64 {
                0
            } else {
                a >> b
            }
        }
        _ => return Err(error("Unknown integer operator")),
    };
    Ok(n & c.mask())
}
pub fn unary(op: &str, a: u64, c: IntegerConfig) -> Result<u64, CalcError> {
    c.validate()?;
    let a = a & c.mask();
    let n = match op {
        "not" => !a,
        "neg" => 0u64.wrapping_sub(a),
        "rol" => (a << 1) | (a >> (c.width - 1)),
        "ror" => (a >> 1) | ((a & 1) << (c.width - 1)),
        "shl" => a << 1,
        "shr" => return binary(">>", a, 1, c),
        "byteflip" => {
            let mut n = 0;
            for shift in (0..c.width).step_by(16) {
                let pair = (a >> shift) & 0xffff;
                n |= (((pair & 255) << 8) | (pair >> 8)) << shift;
            }
            n
        }
        "wordflip" => {
            let mut n = 0;
            for shift in (0..c.width).step_by(32) {
                let pair = (a >> shift) & 0xffff_ffff;
                n |= (((pair & 0xffff) << 16) | (pair >> 16)) << shift;
            }
            n
        }
        _ => return Err(error("Unknown integer function")),
    };
    Ok(n & c.mask())
}
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
    config: IntegerConfig,
    repeat: Option<(String, u64)>,
}
impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn take(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }
    fn parse(&mut self, minimum: u8) -> Result<u64, CalcError> {
        self.depth += 1;
        if self.depth > 128 {
            return Err(error("Expression nesting is too deep"));
        }
        let mut left = match self.take() {
            Token::Number(text) => {
                u64::from_str_radix(&text, self.config.base)
                    .map_err(|_| error("Invalid digit or integer larger than 64 bits"))?
                    & self.config.mask()
            }
            Token::Op(op) if op == "-" || op == "+" || op == "~" => {
                let value = self.parse(60)?;
                match op.as_str() {
                    "-" => 0u64.wrapping_sub(value) & self.config.mask(),
                    "~" => !value & self.config.mask(),
                    _ => value,
                }
            }
            Token::Left => {
                let v = self.parse(0)?;
                if self.take() != Token::Right {
                    return Err(error("Missing closing parenthesis"));
                }
                v
            }
            _ => return Err(error("Expected an integer")),
        };
        while let Token::Op(op) = self.peek() {
            let op = op.clone();
            let priority = match op.as_str() {
                "|" | "or" | "nor" => 10,
                "^" | "xor" => 15,
                "&" | "and" => 20,
                "<<" | ">>" | "shl" | "shr" => 25,
                "+" | "-" => 30,
                "*" | "/" | "%" | "mod" => 40,
                _ => break,
            };
            if priority < minimum {
                break;
            }
            self.take();
            let right = self.parse(priority + 1)?;
            left = binary(&op, left, right, self.config)?;
            if self.depth == 1 {
                self.repeat = Some((op, right));
            }
        }
        self.depth -= 1;
        Ok(left)
    }
}
pub fn evaluate(input: &str, config: IntegerConfig) -> Result<u64, CalcError> {
    Ok(evaluate_details(input, config)?.0)
}
pub(crate) fn evaluate_details(
    input: &str,
    config: IntegerConfig,
) -> Result<(u64, Option<(String, u64)>), CalcError> {
    config.validate()?;
    let mut p = Parser {
        tokens: lex(input, config.base)?,
        pos: 0,
        depth: 0,
        config,
        repeat: None,
    };
    let n = p.parse(0)?;
    if p.peek() != &Token::End {
        return Err(error("Unexpected input after integer expression"));
    }
    Ok((n, p.repeat))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_64_bit_and_wrapping() {
        let c = IntegerConfig::default();
        assert_eq!(evaluate("FFFFFFFFFFFFFFFF", c).unwrap(), u64::MAX);
        assert_eq!(evaluate("FFFFFFFFFFFFFFFF + 1", c).unwrap(), 0);
        assert_eq!(evaluate("FF & 0F", c).unwrap(), 15);
        assert_eq!(evaluate("FF xor 0F", c).unwrap(), 240);
        let d = IntegerConfig { base: 10, ..c };
        assert_eq!(evaluate("9007199254740993+1", d).unwrap(), 9007199254740994);
    }
    #[test]
    fn shift_and_complement_boundaries() {
        for width in [8, 16, 32, 64] {
            let c = IntegerConfig {
                width,
                ..Default::default()
            };
            assert_eq!(binary("<<", 1, width as u64, c).unwrap(), 0);
            assert_eq!(unary("rol", 1u64 << (width - 1), c).unwrap(), 1);
            assert_eq!(unary("ror", 1, c).unwrap(), 1u64 << (width - 1));
            assert_eq!(unary("not", 0, c).unwrap(), c.mask());
        }
        let c = IntegerConfig {
            width: 16,
            ..Default::default()
        };
        assert_eq!(unary("byteflip", 0xABCD, c).unwrap(), 0xCDAB);
        assert_eq!(
            unary("wordflip", 0xABCD1234, IntegerConfig::default()).unwrap(),
            0x1234ABCD
        );
    }
    #[test]
    fn signed_division_and_shift() {
        let c = IntegerConfig {
            width: 8,
            signed: true,
            arithmetic_shift: true,
            base: 10,
        };
        assert_eq!(c.format(evaluate("-8/2", c).unwrap()), "-4");
        assert_eq!(binary(">>", 255, 8, c).unwrap(), 255);
        assert!(evaluate("1/0", c).is_err());
        assert!(evaluate("999999999999999999999999999", c).is_err());
    }
}
