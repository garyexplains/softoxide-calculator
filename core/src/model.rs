use crate::{
    expression::{self, error, finite, Angle, CalcError, Expr},
    programmer::{self, IntegerConfig},
    units,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Basic,
    Scientific,
    Programmer,
}
impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Scientific => "Scientific",
            Self::Programmer => "Programmer",
        }
    }
}
#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: String,
    pub timestamp: u64,
    pub mode: Mode,
    pub angle: Angle,
    pub integer: IntegerConfig,
    pub rpn: bool,
    pub conversion: Option<(usize, usize)>,
}
#[derive(Clone, Debug)]
enum Repeat {
    Real(String, f64),
    Integer(String, u64),
}
#[derive(Clone, Debug)]
pub struct Calculator {
    pub input: String,
    pub display: String,
    pub error: Option<String>,
    pub mode: Mode,
    pub angle: Angle,
    pub integer: IntegerConfig,
    pub rpn: bool,
    pub stack: Vec<f64>,
    pub memory: Option<f64>,
    pub integer_memory: Option<u64>,
    pub history: Vec<HistoryEntry>,
    pub conversion: Option<(usize, usize)>,
    pub last_expression: String,
    repeat: Option<Repeat>,
    finished: bool,
    seed: u64,
}
impl Default for Calculator {
    fn default() -> Self {
        Self {
            input: String::new(),
            display: "0".into(),
            error: None,
            mode: Mode::Basic,
            angle: Angle::Degrees,
            integer: IntegerConfig::default(),
            rpn: false,
            stack: vec![],
            memory: None,
            integer_memory: None,
            history: vec![],
            conversion: None,
            last_expression: String::new(),
            repeat: None,
            finished: false,
            seed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
                | 1,
        }
    }
}
impl Calculator {
    fn operand_start(text: &str) -> usize {
        let chars: Vec<_> = text.char_indices().collect();
        let Some(&(last, ch)) = chars.last() else {
            return 0;
        };
        let mut start = last;
        if ch == ')' {
            let mut depth = 0;
            for &(i, c) in chars.iter().rev() {
                if c == ')' {
                    depth += 1;
                }
                if c == '(' {
                    depth -= 1;
                    if depth == 0 {
                        start = i;
                        break;
                    }
                }
            }
            while start > 0 && text.as_bytes()[start - 1].is_ascii_alphabetic() {
                start -= 1;
            }
        } else {
            for &(i, c) in chars.iter().rev() {
                if c.is_ascii_alphanumeric()
                    || c == '.'
                    || ((c == '+' || c == '-')
                        && i > 0
                        && matches!(text.as_bytes()[i - 1], b'e' | b'E'))
                {
                    start = i;
                } else {
                    break;
                }
            }
        }
        let prefix = text[..start].trim_end();
        if prefix.ends_with('-') {
            let minus = prefix.len() - 1;
            if prefix[..minus]
                .trim_end()
                .chars()
                .last()
                .is_none_or(|c| "(+*/^-".contains(c))
            {
                start = minus;
            }
        }
        start
    }
    pub fn edited(&mut self) {
        self.finished = false;
        self.repeat = None;
        self.error = None;
    }
    pub fn append(&mut self, text: &str) {
        if self.finished {
            self.input.clear();
            self.finished = false;
            self.repeat = None;
        }
        if self.input.len() + text.len() > 4096 {
            self.error = Some("Expression is too long".into());
            return;
        }
        self.input.push_str(text);
        self.error = None;
    }
    pub fn digit(&mut self, digit: char) {
        if self.mode == Mode::Programmer && digit.to_digit(self.integer.base).is_none() {
            return;
        }
        self.append(&digit.to_string());
    }
    fn insert_operand(&mut self, text: &str) {
        if self.finished {
            self.input.clear();
        }
        let trimmed = self.input.trim_end();
        let word_operator = trimmed
            .split_whitespace()
            .last()
            .is_some_and(|word| ["xor", "nor", "mod", "and", "or", "shl", "shr"].contains(&word));
        if !word_operator
            && trimmed
                .chars()
                .last()
                .is_some_and(|c| !"+-*/^&|~(,".contains(c))
        {
            let start = Self::operand_start(trimmed);
            self.input.truncate(start);
        }
        self.edited();
        self.append(text);
    }
    pub fn operator(&mut self, op: &str) {
        if self.rpn {
            self.rpn_binary(op);
            return;
        }
        if self.finished {
            self.input = self.canonical_result();
            self.finished = false;
        }
        if self.input.trim().is_empty() {
            if op == "-" {
                self.append("-");
            }
            return;
        }
        self.input = self.input.trim_end().to_string();
        if let Some(index) = self.input.rfind(' ') {
            let tail = &self.input[index + 1..];
            if [
                "+", "-", "*", "/", "^", "&", "|", "xor", "nor", "mod", "<<", ">>",
            ]
            .contains(&tail)
            {
                self.input.truncate(index);
            }
        }
        self.input.push_str(&format!(" {op} "));
        self.error = None;
        self.repeat = None;
    }
    pub fn clear(&mut self) {
        self.input.clear();
        self.display = "0".into();
        self.error = None;
        self.repeat = None;
        self.finished = false;
        self.stack.clear();
        self.last_expression.clear();
    }
    pub fn clear_entry(&mut self) {
        if self.finished {
            self.input.clear();
            self.display = "0".into();
        } else {
            let trimmed = self.input.trim_end();
            self.input = trimmed[..Self::operand_start(trimmed)].to_owned();
        }
        self.finished = false;
        self.repeat = None;
        self.error = None;
    }
    pub fn delete(&mut self) {
        self.input.pop();
        self.edited();
    }
    fn real(&self) -> Result<f64, CalcError> {
        if self.rpn && self.input.trim().is_empty() {
            self.stack
                .last()
                .copied()
                .ok_or_else(|| error("The stack is empty"))
        } else if self.input.trim().is_empty() {
            Ok(0.0)
        } else {
            expression::evaluate(&self.input, self.angle)
        }
    }
    pub fn integer_value(&self) -> Result<u64, CalcError> {
        if self.input.trim().is_empty() {
            Ok(0)
        } else {
            programmer::evaluate(&self.input, self.integer)
        }
    }
    fn canonical_result(&self) -> String {
        if self.mode == Mode::Programmer {
            self.integer_value()
                .map(|n| self.integer.format(n))
                .unwrap_or_else(|_| "0".into())
        } else {
            self.real()
                .map(|n| n.to_string())
                .unwrap_or_else(|_| "0".into())
        }
    }
    fn fail(&mut self, err: CalcError) {
        self.error = Some(err.0);
    }
    fn record(&mut self, expression: String, result: String) {
        if self.history.last().is_some_and(|h| {
            h.expression == expression
                && h.result == result
                && h.mode == self.mode
                && h.angle == self.angle
                && h.integer == self.integer
                && h.rpn == self.rpn
                && h.conversion == self.conversion
        }) {
            return;
        }
        self.history.push(HistoryEntry {
            expression,
            result,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            mode: self.mode,
            angle: self.angle,
            integer: self.integer,
            rpn: self.rpn,
            conversion: self.conversion,
        });
        if self.history.len() > 200 {
            self.history.remove(0);
        }
    }
    pub fn equals(&mut self) {
        if self.rpn {
            self.enter();
            return;
        }
        let mut source = self.input.clone();
        if self.mode == Mode::Programmer {
            if !self.finished {
                self.repeat = programmer::evaluate_details(
                    if source.is_empty() { "0" } else { &source },
                    self.integer,
                )
                .ok()
                .and_then(|(_, repeat)| repeat.map(|(op, rhs)| Repeat::Integer(op, rhs)));
            }
            let result = if self.finished {
                match self.repeat.clone() {
                    Some(Repeat::Integer(op, rhs)) => self.integer_value().and_then(|lhs| {
                        source = format!(
                            "({}) {op} ({})",
                            self.integer.format(lhs),
                            self.integer.format(rhs)
                        );
                        programmer::binary(&op, lhs, rhs, self.integer)
                    }),
                    _ => self.integer_value(),
                }
            } else {
                self.integer_value()
            };
            match result {
                Ok(n) => {
                    self.display = self.integer.format(n);
                    self.last_expression = source.clone();
                    self.input = self.display.clone();
                    self.finished = true;
                    self.error = None;
                    self.record(source, self.display.clone());
                }
                Err(err) => self.fail(err),
            }
            return;
        }
        let repeated = self.finished;
        let tree = if source.trim().is_empty() {
            expression::parse("0")
        } else {
            expression::parse(&source)
        };
        let result = if repeated {
            match self.repeat.clone() {
                Some(Repeat::Real(op, rhs)) => self.real().and_then(|lhs| {
                    source = format!("({lhs}) {op} ({rhs})");
                    expression::binary(&op, lhs, rhs)
                }),
                _ => self.real(),
            }
        } else {
            tree.as_ref()
                .map_err(Clone::clone)
                .and_then(|expr| expression::eval(expr, self.angle))
        };
        match result {
            Ok(n) => {
                let converted = match self.conversion {
                    Some((a, b)) => units::convert(n, a, b),
                    None => Ok(n),
                };
                match converted {
                    Ok(value) => {
                        if !repeated {
                            self.repeat = match tree {
                                Ok(Expr::Binary(op, a, b)) => {
                                    let rhs = expression::eval(&b, self.angle);
                                    let lhs = expression::eval(&a, self.angle);
                                    rhs.ok().zip(lhs.ok()).map(|(rhs, lhs)| {
                                        Repeat::Real(
                                            op.clone(),
                                            if (op == "+" || op == "-")
                                                && matches!(*b, Expr::Percent(_))
                                            {
                                                rhs * lhs
                                            } else {
                                                rhs
                                            },
                                        )
                                    })
                                }
                                _ => None,
                            };
                        }
                        self.display = expression::format_number(value);
                        self.last_expression = source.clone();
                        self.record(source, self.display.clone());
                        self.input = n.to_string();
                        self.error = None;
                        self.finished = true;
                    }
                    Err(err) => self.fail(err),
                }
            }
            Err(err) => self.fail(err),
        }
    }
    pub fn apply_function(&mut self, name: &str) {
        if self.mode == Mode::Programmer {
            match self
                .integer_value()
                .and_then(|n| programmer::unary(name, n, self.integer))
            {
                Ok(n) => {
                    self.input = self.integer.format(n);
                    self.display = self.input.clone();
                    self.error = None;
                    self.finished = false;
                    self.repeat = None;
                }
                Err(e) => self.fail(e),
            }
            return;
        }
        if !self.rpn {
            let old = if self.finished {
                self.canonical_result()
            } else if self.input.is_empty() {
                "0".into()
            } else {
                self.input.trim_end().to_owned()
            };
            let start = Self::operand_start(&old);
            let candidate = if name == "percent" {
                format!("{}({})%", &old[..start], &old[start..])
            } else {
                format!("{}{name}({})", &old[..start], &old[start..])
            };
            match expression::evaluate(&candidate, self.angle) {
                Ok(n) => {
                    self.input = candidate;
                    self.display = expression::format_number(n);
                    self.finished = false;
                    self.repeat = None;
                    self.error = None;
                }
                Err(e) => self.fail(e),
            }
            return;
        }
        let result = self.real().and_then(|x| {
            if name == "percent" {
                finite(x / 100.0)
            } else {
                expression::function(name, &[x], self.angle)
            }
        });
        match result {
            Ok(n) => {
                if self.rpn {
                    if !self.input.is_empty() {
                        self.input = n.to_string();
                    } else if let Some(top) = self.stack.last_mut() {
                        *top = n;
                    }
                } else {
                    let old = if self.input.is_empty() {
                        "0".to_owned()
                    } else {
                        self.input.clone()
                    };
                    self.input = if name == "percent" {
                        format!("({old})%")
                    } else {
                        format!("{name}({old})")
                    };
                }
                self.display = expression::format_number(n);
                self.finished = false;
                self.error = None;
                self.repeat = None;
            }
            Err(e) => self.fail(e),
        }
    }
    pub fn percent(&mut self) {
        if self.rpn {
            self.apply_function("percent");
        } else if self.mode == Mode::Programmer {
            self.operator("mod");
        } else {
            if self.finished {
                self.input = self.canonical_result();
                self.finished = false;
            }
            self.append("%");
        }
    }
    pub fn change_sign(&mut self) {
        if self.rpn || self.mode == Mode::Programmer {
            self.apply_function("neg");
            return;
        }
        if self.finished {
            self.input = self.canonical_result();
            self.finished = false;
        }
        if self.input.trim().is_empty() || self.input.ends_with(' ') {
            self.input.push('-');
            self.edited();
            return;
        }
        let start = Self::operand_start(&self.input);
        let operand = self.input[start..].to_owned();
        self.input.truncate(start);
        if let Some(positive) = operand.strip_prefix('-') {
            self.input.push_str(positive);
        } else {
            self.input.push('-');
            self.input.push_str(&operand);
        }
        self.edited();
    }
    pub fn constant(&mut self, name: &str) {
        if self.mode == Mode::Programmer {
            return;
        }
        let value = match name {
            "pi" => std::f64::consts::PI,
            "e" => std::f64::consts::E,
            _ => {
                self.seed ^= self.seed << 13;
                self.seed ^= self.seed >> 7;
                self.seed ^= self.seed << 17;
                (self.seed >> 11) as f64 / (1u64 << 53) as f64
            }
        };
        self.insert_operand(&format!("({value})"));
    }
    pub fn memory(&mut self, action: &str) {
        if self.mode == Mode::Programmer {
            let result = self.integer_value();
            match action {
                "MC" => self.integer_memory = None,
                "MR" => {
                    let n = self.integer_memory.unwrap_or(0);
                    self.insert_operand(&self.integer.format(n));
                }
                "M+" | "M-" => match result.and_then(|n| {
                    programmer::binary(
                        if action == "M+" { "+" } else { "-" },
                        self.integer_memory.unwrap_or(0),
                        n,
                        self.integer,
                    )
                }) {
                    Ok(n) => self.integer_memory = Some(n),
                    Err(e) => self.fail(e),
                },
                _ => {}
            }
        } else {
            match action {
                "MC" => self.memory = None,
                "MR" => self.insert_operand(&format!("({})", self.memory.unwrap_or(0.0))),
                "M+" | "M-" => match self.real().and_then(|n| {
                    finite(self.memory.unwrap_or(0.0) + if action == "M+" { n } else { -n })
                }) {
                    Ok(n) => self.memory = Some(n),
                    Err(e) => self.fail(e),
                },
                _ => {}
            }
        }
    }
    pub fn set_mode(&mut self, mode: Mode) {
        if self.mode == mode {
            return;
        }
        self.clear();
        self.mode = mode;
        self.rpn = false;
        self.conversion = None;
    }
    pub fn set_base(&mut self, base: u32) {
        match self.integer_value() {
            Ok(n) => {
                self.integer.base = base;
                self.input = self.integer.format(n);
                self.display = self.input.clone();
                self.finished = false;
                self.repeat = None;
                self.error = None;
            }
            Err(e) => self.fail(e),
        }
    }
    pub fn set_width(&mut self, width: u32) {
        match self.integer_value() {
            Ok(n) => {
                self.integer.width = width;
                self.input = self.integer.format(n);
                self.display = self.input.clone();
                self.edited();
            }
            Err(e) => self.fail(e),
        }
    }
    pub fn set_rpn(&mut self, enabled: bool) {
        if self.rpn != enabled {
            self.clear();
            self.rpn = enabled;
            self.conversion = None;
        }
    }
    pub fn enter(&mut self) {
        let result = self.real();
        match result {
            Ok(n) => {
                if self.stack.len() >= 128 {
                    self.fail(error("Stack limit reached (128 values)"));
                    return;
                }
                self.stack.push(n);
                self.record(
                    if self.input.is_empty() {
                        "Duplicate top".into()
                    } else {
                        self.input.clone()
                    },
                    expression::format_number(n),
                );
                self.input.clear();
                self.display = expression::format_number(n);
                self.error = None;
                self.repeat = None;
                self.finished = false;
            }
            Err(e) => self.fail(e),
        }
    }
    fn rpn_binary(&mut self, op: &str) {
        let mut working = self.stack.clone();
        if !self.input.trim().is_empty() {
            match expression::evaluate(&self.input, self.angle) {
                Ok(n) => working.push(n),
                Err(e) => {
                    self.fail(e);
                    return;
                }
            }
        }
        if working.len() < 2 {
            self.fail(error("This operation needs two stack values"));
            return;
        }
        let b = working.pop().unwrap();
        let a = working.pop().unwrap();
        let result = if ["root", "rroot", "log", "pow"].contains(&op) {
            expression::function(op, &[a, b], self.angle)
        } else {
            expression::binary(op, a, b)
        };
        match result {
            Ok(n) => {
                working.push(n);
                self.stack = working;
                self.input.clear();
                self.display = expression::format_number(n);
                self.error = None;
                self.record(format!("{a} {op} {b}"), self.display.clone());
            }
            Err(e) => self.fail(e),
        }
    }
    pub fn stack_action(&mut self, action: &str) {
        let mut working = self.stack.clone();
        if !self.input.trim().is_empty() {
            match expression::evaluate(&self.input, self.angle) {
                Ok(n) => working.push(n),
                Err(e) => {
                    self.fail(e);
                    return;
                }
            }
        }
        if working.len() > 128 {
            self.fail(error("Stack limit reached (128 values)"));
            return;
        }
        match action {
            "drop" => {
                if working.pop().is_none() {
                    self.fail(error("The stack is empty"));
                    return;
                }
            }
            "swap" => {
                let len = working.len();
                if len < 2 {
                    self.fail(error("Swap needs two stack values"));
                    return;
                }
                working.swap(len - 1, len - 2);
            }
            "up" if !working.is_empty() => working.rotate_left(1),
            "down" if !working.is_empty() => working.rotate_right(1),
            _ => {}
        }
        self.stack = working;
        self.input.clear();
        self.display = expression::format_number(self.stack.last().copied().unwrap_or(0.0));
        self.error = None;
    }
    pub fn restore(&mut self, entry: &HistoryEntry) {
        self.clear();
        self.mode = entry.mode;
        self.angle = entry.angle;
        self.integer = entry.integer;
        self.rpn = entry.rpn;
        self.conversion = entry.conversion;
        // RPN records are stored as ordinary expressions; restore their value onto the stack.
        if self.rpn {
            if let Ok(n) = entry.result.replace('−', "-").parse::<f64>() {
                self.stack.push(n);
            }
            self.display = entry.result.clone();
        } else {
            self.input = entry.expression.clone();
            self.display = entry.result.clone();
        }
        self.last_expression = entry.expression.clone();
    }
    pub fn preview(&self) -> String {
        if self.input.trim().is_empty() {
            return self.display.clone();
        }
        if self.mode == Mode::Programmer {
            self.integer_value()
                .map(|n| self.integer.format(n))
                .unwrap_or_else(|_| self.display.clone())
        } else {
            self.real()
                .and_then(|n| match self.conversion {
                    Some((a, b)) => units::convert(n, a, b),
                    None => Ok(n),
                })
                .map(expression::format_number)
                .unwrap_or_else(|_| self.display.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trailing_operand_edits_preserve_expression() {
        let mut c = Calculator::default();
        c.append("2+9");
        c.apply_function("sqrt");
        c.operator("*");
        c.append("4");
        c.equals();
        assert_eq!(c.display, "14");
        c.clear();
        c.change_sign();
        c.digit('3');
        assert_eq!(c.input, "-3");
        c.change_sign();
        assert_eq!(c.input, "3");
        c.clear();
        c.append("2+123");
        c.clear_entry();
        c.append("4");
        c.equals();
        assert_eq!(c.display, "6");
    }
    #[test]
    fn rpn_functions_and_failed_swap_are_transactional() {
        let mut c = Calculator::default();
        c.set_rpn(true);
        c.append("8");
        c.enter();
        c.append("3");
        c.operator("root");
        assert_eq!(c.stack, vec![2.0]);
        c.clear();
        c.append("2");
        c.stack_action("swap");
        assert!(c.error.is_some());
        assert!(c.stack.is_empty());
        assert_eq!(c.input, "2");
    }
    #[test]
    fn integer_repeat_and_operator_replacement() {
        let mut c = Calculator::default();
        c.set_mode(Mode::Programmer);
        c.append("2");
        c.operator("+");
        c.append("3");
        c.equals();
        c.equals();
        assert_eq!(c.display, "8");
        c.clear();
        c.append("2");
        c.operator("+");
        c.operator("*");
        c.append("3");
        c.equals();
        assert_eq!(c.display, "6");
    }
    #[test]
    fn editing_repeat_percent_and_memory() {
        let mut c = Calculator::default();
        c.append("2+3");
        c.equals();
        assert_eq!(c.display, "5");
        c.equals();
        assert_eq!(c.display, "8");
        c.equals();
        assert_eq!(c.display, "11");
        c.digit('9');
        assert_eq!(c.input, "9");
        c.memory("M+");
        c.clear();
        c.memory("MR");
        c.equals();
        assert_eq!(c.display, "9");
        c.clear();
        c.append("2");
        c.operator("+");
        c.append("111");
        c.memory("MR");
        c.equals();
        assert_eq!(c.display, "11");
        c.clear();
        c.append("200");
        c.operator("+");
        c.append("10");
        c.percent();
        c.equals();
        assert_eq!(c.display, "220");
    }
    #[test]
    fn rpn_transaction_and_order() {
        let mut c = Calculator::default();
        c.set_rpn(true);
        c.append("2");
        c.enter();
        c.append("3");
        c.operator("-");
        assert_eq!(c.stack, vec![-1.0]);
        c.append("0");
        c.operator("/");
        assert!(c.error.is_some());
        assert_eq!(c.stack, vec![-1.0]);
        assert_eq!(c.input, "0");
        c.clear();
        c.stack_action("up");
        assert!(c.stack.is_empty());
        c.operator("+");
        assert!(c.error.is_some());
    }
    #[test]
    fn programmer_base_and_history_context() {
        let mut c = Calculator::default();
        c.set_mode(Mode::Programmer);
        c.append("FFFFFFFFFFFFFFFF");
        c.equals();
        assert_eq!(c.display, "FFFFFFFFFFFFFFFF");
        c.set_base(10);
        assert_eq!(c.display, "18446744073709551615");
        c.operator("+");
        c.append("1");
        c.equals();
        assert_eq!(c.display, "0");
        let h = c.history[0].clone();
        c.restore(&h);
        assert_eq!(c.mode, Mode::Programmer);
        assert_eq!(c.integer.base, 16);
    }
    #[test]
    fn errors_and_history_dedup() {
        let mut c = Calculator::default();
        c.append("1/0");
        c.equals();
        assert!(c.history.is_empty());
        assert!(c.error.is_some());
        c.clear();
        c.append("2+3");
        c.equals();
        c.clear();
        c.append("2+3");
        c.equals();
        assert_eq!(c.history.len(), 1);
    }
}
