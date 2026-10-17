use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Angle {
    #[default]
    Degrees,
    Radians,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalcError(pub String);
impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for CalcError {}
pub(crate) fn error(message: &str) -> CalcError {
    CalcError(message.into())
}
pub(crate) fn finite(n: f64) -> Result<f64, CalcError> {
    if n.is_nan() {
        Err(error("Outside the real-number domain"))
    } else if n.is_infinite() {
        Err(error("Result overflow"))
    } else {
        Ok(n)
    }
}

pub fn format_number(n: f64) -> String {
    if n == 0.0 {
        return if n.is_sign_negative() { "−0" } else { "0" }.into();
    }
    let magnitude = n.abs().log10().floor() as i32;
    if !(-9..15).contains(&magnitude) {
        let text = format!("{n:.13e}");
        let (mantissa, exponent) = text.split_once('e').unwrap();
        return format!(
            "{}e{}",
            mantissa.trim_end_matches('0').trim_end_matches('.'),
            exponent
        );
    }
    let precision = (14 - magnitude).clamp(0, 24) as usize;
    let text = format!("{n:.precision$}");
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').into()
    } else {
        text
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token {
    Number(String),
    Name(String),
    Op(String),
    Left,
    Right,
    Comma,
    End,
}
pub(crate) fn lex(input: &str, base: u32) -> Result<Vec<Token>, CalcError> {
    if input.len() > 4096 {
        return Err(error("Expression is too long"));
    }
    let input = input
        .replace('−', "-")
        .replace('×', "*")
        .replace('÷', "/")
        .replace('π', "pi");
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if base != 10 && c.is_ascii_alphabetic() {
            let mut end = i;
            while end < chars.len() && chars[end].is_ascii_alphabetic() {
                end += 1;
            }
            let word = chars[i..end]
                .iter()
                .collect::<String>()
                .to_ascii_lowercase();
            if ["and", "or", "xor", "nor", "mod", "shl", "shr"].contains(&word.as_str()) {
                tokens.push(Token::Op(word));
                i = end;
                continue;
            }
        }
        let hex_digit = base == 16 && c.is_ascii_hexdigit();
        if c.is_ascii_digit() || c == '.' || hex_digit {
            let start = i;
            if base == 10 {
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                if i < chars.len() && matches!(chars[i], 'e' | 'E') {
                    i += 1;
                    if i < chars.len() && matches!(chars[i], '+' | '-') {
                        i += 1;
                    }
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
            } else {
                while i < chars.len() && chars[i].is_ascii_hexdigit() {
                    i += 1;
                }
                if i == start {
                    return Err(error("Integers cannot contain a decimal point"));
                }
            }
            tokens.push(Token::Number(chars[start..i].iter().collect()));
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let name: String = chars[start..i].iter().collect();
            let name = name.to_ascii_lowercase();
            if ["and", "or", "xor", "nor", "mod", "shl", "shr"].contains(&name.as_str()) {
                tokens.push(Token::Op(name));
            } else {
                tokens.push(Token::Name(name));
            }
            continue;
        }
        match c {
            '(' => tokens.push(Token::Left),
            ')' => tokens.push(Token::Right),
            ',' => tokens.push(Token::Comma),
            '+' | '-' | '*' | '/' | '^' | '%' | '!' | '&' | '|' | '~' => {
                tokens.push(Token::Op(c.to_string()))
            }
            '<' | '>' if i + 1 < chars.len() && chars[i + 1] == c => {
                tokens.push(Token::Op(format!("{c}{c}")));
                i += 1;
            }
            _ => return Err(error(&format!("Unexpected character: {c}"))),
        }
        i += 1;
    }
    tokens.push(Token::End);
    Ok(tokens)
}

#[derive(Clone, Debug)]
pub(crate) enum Expr {
    Number(f64),
    Unary(char, Box<Expr>),
    Binary(String, Box<Expr>, Box<Expr>),
    Percent(Box<Expr>),
    Factorial(Box<Expr>),
    Function(String, Vec<Expr>),
}
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
}
impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn take(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }
    fn parse(&mut self, minimum: u8) -> Result<Expr, CalcError> {
        self.depth += 1;
        if self.depth > 128 {
            return Err(error("Expression nesting is too deep"));
        }
        let mut left = match self.take() {
            Token::Number(n) => Expr::Number(n.parse().map_err(|_| error("Invalid number"))?),
            Token::Op(op) if op == "+" || op == "-" => {
                Expr::Unary(op.chars().next().unwrap(), Box::new(self.parse(25)?))
            }
            Token::Name(name) if name == "pi" => Expr::Number(std::f64::consts::PI),
            Token::Name(name) if name == "e" => Expr::Number(std::f64::consts::E),
            Token::Name(name) => {
                if self.take() != Token::Left {
                    return Err(error("A function needs parentheses"));
                }
                let mut args = vec![self.parse(0)?];
                while self.peek() == &Token::Comma {
                    self.take();
                    args.push(self.parse(0)?);
                }
                if self.take() != Token::Right {
                    return Err(error("Missing closing parenthesis"));
                }
                Expr::Function(name, args)
            }
            Token::Left => {
                let expr = self.parse(0)?;
                if self.take() != Token::Right {
                    return Err(error("Missing closing parenthesis"));
                }
                expr
            }
            _ => return Err(error("Expected a number or expression")),
        };
        while let Token::Op(op) = self.peek() {
            let op = op.clone();
            if op == "%" || op == "!" {
                if minimum > 40 {
                    break;
                }
                self.take();
                left = if op == "%" {
                    Expr::Percent(Box::new(left))
                } else {
                    Expr::Factorial(Box::new(left))
                };
                continue;
            }
            let (lbp, rbp) = match op.as_str() {
                "+" | "-" => (10, 11),
                "*" | "/" | "mod" => (20, 21),
                "^" => (30, 30),
                _ => break,
            };
            if lbp < minimum {
                break;
            }
            self.take();
            left = Expr::Binary(op, Box::new(left), Box::new(self.parse(rbp)?));
        }
        self.depth -= 1;
        Ok(left)
    }
}
pub(crate) fn parse(input: &str) -> Result<Expr, CalcError> {
    let mut parser = Parser {
        tokens: lex(input, 10)?,
        pos: 0,
        depth: 0,
    };
    let expr = parser.parse(0)?;
    if parser.peek() != &Token::End {
        return Err(error("Unexpected input after expression"));
    }
    Ok(expr)
}
pub fn evaluate(input: &str, angle: Angle) -> Result<f64, CalcError> {
    eval(&parse(input)?, angle)
}
pub(crate) fn binary(op: &str, lhs: f64, rhs: f64) -> Result<f64, CalcError> {
    finite(match op {
        "+" => lhs + rhs,
        "-" => lhs - rhs,
        "*" => lhs * rhs,
        "/" if rhs == 0.0 => return Err(error("Cannot divide by zero")),
        "/" => lhs / rhs,
        "mod" if rhs == 0.0 => return Err(error("Cannot take remainder by zero")),
        "mod" => lhs % rhs,
        "^" => lhs.powf(rhs),
        _ => return Err(error("Unknown operator")),
    })
}
pub(crate) fn eval(expr: &Expr, angle: Angle) -> Result<f64, CalcError> {
    match expr {
        Expr::Number(n) => finite(*n),
        Expr::Unary(op, child) => {
            let n = eval(child, angle)?;
            finite(if *op == '-' { -n } else { n })
        }
        Expr::Percent(child) => finite(eval(child, angle)? / 100.0),
        Expr::Factorial(child) => factorial(eval(child, angle)?),
        Expr::Binary(op, a, b) => {
            let lhs = eval(a, angle)?;
            let mut rhs = eval(b, angle)?;
            if (op == "+" || op == "-") && matches!(b.as_ref(), Expr::Percent(_)) {
                rhs *= lhs;
            }
            binary(op, lhs, rhs)
        }
        Expr::Function(name, args) => {
            let values: Result<Vec<_>, _> = args.iter().map(|arg| eval(arg, angle)).collect();
            function(name, &values?, angle)
        }
    }
}
pub fn factorial(n: f64) -> Result<f64, CalcError> {
    if n < 0.0 || n.fract() != 0.0 {
        return Err(error("Factorial requires a nonnegative integer"));
    }
    if n > 170.0 {
        return Err(error("Factorial overflow (maximum is 170)"));
    }
    Ok((1..=n as u32).fold(1.0, |result, value| result * value as f64))
}
pub fn function(name: &str, args: &[f64], angle: Angle) -> Result<f64, CalcError> {
    let x = *args
        .first()
        .ok_or_else(|| error("Missing function argument"))?;
    let circular = if angle == Angle::Degrees {
        x.to_radians()
    } else {
        x
    };
    let inverse = |n: f64| {
        if angle == Angle::Degrees {
            n.to_degrees()
        } else {
            n
        }
    };
    if ["root", "rroot", "log", "pow"].contains(&name) {
        if args.len() != 2 {
            return Err(error("This function needs two arguments"));
        }
        let y = args[1];
        return finite(match name {
            "pow" => x.powf(y),
            "rroot" if x != 0.0 && y < 0.0 && x.fract() == 0.0 && x % 2.0 != 0.0 => {
                -(-y).powf(1.0 / x)
            }
            "rroot" if x != 0.0 => y.powf(1.0 / x),
            "log" if x > 0.0 && y > 0.0 && y != 1.0 => x.log(y),
            "root" if y != 0.0 && x < 0.0 && y.fract() == 0.0 && y % 2.0 != 0.0 => {
                -(-x).powf(1.0 / y)
            }
            "root" if y != 0.0 => x.powf(1.0 / y),
            _ => return Err(error("Outside the function domain")),
        });
    }
    if args.len() != 1 {
        return Err(error("This function needs one argument"));
    }
    finite(match name {
        "sin" => circular.sin(),
        "cos" => circular.cos(),
        "tan" if circular.cos().abs() < 1e-15 => return Err(error("Tangent is undefined here")),
        "tan" => circular.tan(),
        "asin" => inverse(x.asin()),
        "acos" => inverse(x.acos()),
        "atan" => inverse(x.atan()),
        "sinh" => x.sinh(),
        "cosh" => x.cosh(),
        "tanh" => x.tanh(),
        "asinh" => x.asinh(),
        "acosh" => x.acosh(),
        "atanh" => x.atanh(),
        "sqrt" => x.sqrt(),
        "cbrt" => x.cbrt(),
        "abs" => x.abs(),
        "ln" if x > 0.0 => x.ln(),
        "log10" if x > 0.0 => x.log10(),
        "log2" if x > 0.0 => x.log2(),
        "ln" | "log10" | "log2" => return Err(error("Logarithm requires a positive value")),
        "exp" => x.exp(),
        "exp2" => x.exp2(),
        "exp10" => 10f64.powf(x),
        "recip" if x == 0.0 => return Err(error("Cannot divide by zero")),
        "recip" => 1.0 / x,
        "square" => x * x,
        "cube" => x * x * x,
        "neg" => -x,
        "fact" => return factorial(x),
        _ => return Err(error(&format!("Unknown function: {name}"))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn precedence_and_domains() {
        for (input, expected) in [
            ("2+3*4", 14.0),
            ("(2+3)*4", 20.0),
            ("2^3^2", 512.0),
            ("-2^2", -4.0),
            ("200+10%", 220.0),
            ("200*10%", 20.0),
            ("root(-8,3)", -2.0),
        ] {
            assert!(
                (evaluate(input, Angle::Degrees).unwrap() - expected).abs() < 1e-10,
                "{input}"
            );
        }
        for input in [
            "1/0",
            "sqrt(-1)",
            "ln(0)",
            "fact(0.5)",
            "(2+3",
            "1e+",
            "tan(90)",
        ] {
            assert!(evaluate(input, Angle::Degrees).is_err(), "{input}");
        }
    }
    #[test]
    fn functions_angles_and_formatting() {
        assert!((evaluate("sin(30)", Angle::Degrees).unwrap() - 0.5).abs() < 1e-14);
        assert!((evaluate("sin(pi/2)", Angle::Radians).unwrap() - 1.0).abs() < 1e-14);
        assert_eq!(format_number(0.1 + 0.2), "0.3");
        assert_eq!(format_number(1000.0), "1000");
        assert_eq!(evaluate("1e-3+2e-3", Angle::Degrees).unwrap(), 0.003);
    }
    #[test]
    fn malformed_input_never_panics() {
        for input in [
            "", ")", "sin", "sin(", "sin()", "sin(1,)", "sin(1,2)", ".", "2 3", "foo(1)", "1+",
            "(((",
        ] {
            assert!(evaluate(input, Angle::Degrees).is_err(), "{input}");
        }
    }
}
