//! Independent known-answer and invariant checks for the implemented functions.
//! These test this calculator's documented policies, not Apple's private engine.
use calculator_core::{
    evaluate, expression,
    programmer::{self, IntegerConfig},
    storage, units, Angle, Calculator, Mode,
};

fn close(actual: f64, expected: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "expected {expected}, got {actual}"
    );
}
macro_rules! calculation {
    ($name:ident,$input:expr,$expected:expr) => {
        #[test]
        fn $name() {
            close(evaluate($input, Angle::Degrees).unwrap(), $expected);
        }
    };
}
calculation!(basic_add, "12+7", 19.0);
calculation!(basic_subtract, "12-7", 5.0);
calculation!(basic_multiply, "12*7", 84.0);
calculation!(basic_divide, "84/7", 12.0);
calculation!(basic_precedence, "2+3*4", 14.0);
calculation!(basic_parentheses, "(2+3)*4", 20.0);
calculation!(basic_percent_standalone, "50%", 0.5);
calculation!(basic_percent_add, "200+10%", 220.0);
calculation!(basic_percent_subtract, "200-10%", 180.0);
calculation!(basic_percent_multiply, "200*10%", 20.0);
calculation!(basic_percent_divide, "200/10%", 2000.0);
calculation!(basic_exponent_entry, "1.25e3+5e-1", 1250.5);
calculation!(scientific_square, "square(12)", 144.0);
calculation!(scientific_cube, "cube(-3)", -27.0);
calculation!(scientific_power, "2^10", 1024.0);
calculation!(scientific_power_function, "pow(3,4)", 81.0);
calculation!(scientific_power_associativity, "2^3^2", 512.0);
calculation!(scientific_square_root, "sqrt(144)", 12.0);
calculation!(scientific_cube_root, "cbrt(-27)", -3.0);
calculation!(scientific_nth_root, "root(81,4)", 3.0);
calculation!(scientific_reverse_root, "rroot(4,81)", 3.0);
calculation!(scientific_negative_odd_root, "root(-32,5)", -2.0);
calculation!(scientific_reciprocal, "recip(8)", 0.125);
calculation!(scientific_exp, "exp(1)", std::f64::consts::E);
calculation!(scientific_exp10, "exp10(3)", 1000.0);
calculation!(scientific_exp2, "exp2(10)", 1024.0);
calculation!(scientific_ln, "ln(e)", 1.0);
calculation!(scientific_log10, "log10(1000)", 3.0);
calculation!(scientific_log2, "log2(1024)", 10.0);
calculation!(scientific_custom_log, "log(81,3)", 4.0);
calculation!(scientific_factorial, "fact(10)", 3628800.0);
calculation!(scientific_postfix_factorial, "5!", 120.0);
calculation!(scientific_zero_factorial, "fact(0)", 1.0);
calculation!(scientific_sin, "sin(30)", 0.5);
calculation!(scientific_cos, "cos(60)", 0.5);
calculation!(scientific_tan, "tan(45)", 1.0);
calculation!(scientific_asin, "asin(0.5)", 30.0);
calculation!(scientific_acos, "acos(0.5)", 60.0);
calculation!(scientific_atan, "atan(1)", 45.0);
calculation!(scientific_sinh, "sinh(1)", 1.1752011936438014);
calculation!(scientific_cosh, "cosh(1)", 1.5430806348152437);
calculation!(scientific_tanh, "tanh(1)", 0.7615941559557649);
calculation!(scientific_asinh, "asinh(1)", 0.881373587019543);
calculation!(scientific_acosh, "acosh(2)", 1.3169578969248166);
calculation!(scientific_atanh, "atanh(0.5)", 0.5493061443340548);
calculation!(scientific_negate, "neg(12)", -12.0);
calculation!(scientific_absolute_value, "abs(-12)", 12.0);
calculation!(scientific_modulo, "17 mod 5", 2.0);

#[test]
fn scientific_radians_and_inverse_identities() {
    close(evaluate("sin(pi/2)", Angle::Radians).unwrap(), 1.0);
    close(
        evaluate("acos(0)", Angle::Radians).unwrap(),
        std::f64::consts::FRAC_PI_2,
    );
    for angle in [Angle::Degrees, Angle::Radians] {
        for x in [-0.75, -0.1, 0.0, 0.4, 0.9] {
            let sine = expression::function("sin", &[x], angle).unwrap();
            close(expression::function("asin", &[sine], angle).unwrap(), x);
            let hyperbolic = expression::function("sinh", &[x], angle).unwrap();
            close(
                expression::function("asinh", &[hyperbolic], angle).unwrap(),
                x,
            );
        }
    }
}
#[test]
fn scientific_invalid_domains_and_overflow() {
    for input in [
        "1/0",
        "recip(0)",
        "sqrt(-1)",
        "root(1,0)",
        "root(-16,2)",
        "rroot(0,1)",
        "ln(0)",
        "log10(-1)",
        "log(10,1)",
        "asin(2)",
        "acos(-2)",
        "acosh(0)",
        "atanh(1)",
        "tan(90)",
        "fact(-1)",
        "fact(0.5)",
        "fact(171)",
        "exp(10000)",
    ] {
        assert!(
            evaluate(input, Angle::Degrees).is_err(),
            "{input} must report an error"
        );
    }
}
#[test]
fn constants_and_random_range() {
    for (name, expected) in [("pi", std::f64::consts::PI), ("e", std::f64::consts::E)] {
        let mut c = Calculator::default();
        c.constant(name);
        close(evaluate(&c.input, c.angle).unwrap(), expected);
    }
    let mut c = Calculator::default();
    let mut values = Vec::new();
    for _ in 0..128 {
        c.clear();
        c.constant("random");
        let n = evaluate(&c.input, c.angle).unwrap();
        assert!((0.0..1.0).contains(&n));
        values.push(n);
    }
    assert!(
        values.windows(2).any(|v| v[0] != v[1]),
        "random generator must advance"
    );
}

macro_rules! integer_case {
    ($name:ident,$input:expr,$expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(
                programmer::evaluate($input, IntegerConfig::default()).unwrap(),
                $expected
            );
        }
    };
}
integer_case!(programmer_add, "A+5", 15);
integer_case!(programmer_subtract, "A-5", 5);
integer_case!(programmer_multiply, "A*5", 50);
integer_case!(programmer_divide, "A/5", 2);
integer_case!(programmer_remainder, "11 mod 5", 2);
integer_case!(programmer_and, "AA & 0F", 10);
integer_case!(programmer_or, "A0 | 0F", 175);
integer_case!(programmer_xor, "AA xor FF", 85);
integer_case!(programmer_nor, "F0 nor 0F", u64::MAX - 255);
integer_case!(programmer_shift_left_custom, "1 << 4", 16);
integer_case!(programmer_shift_right_custom, "80 >> 3", 16);
integer_case!(programmer_exact_large_integer, "FFFFFFFFFFFFFFFF", u64::MAX);

#[test]
fn programmer_unary_functions_at_every_width() {
    for width in [8, 16, 32, 64] {
        let c = IntegerConfig {
            width,
            ..Default::default()
        };
        for (op, input, expected) in [
            ("not", 0, c.mask()),
            ("neg", 1, c.mask()),
            ("shl", 1, 2),
            ("shr", 8, 4),
            ("rol", 1u64 << (width - 1), 1),
            ("ror", 1, 1u64 << (width - 1)),
        ] {
            assert_eq!(
                programmer::unary(op, input, c).unwrap(),
                expected,
                "{op}, {width}-bit"
            );
        }
    }
    assert_eq!(
        programmer::unary(
            "byteflip",
            0xABCD,
            IntegerConfig {
                width: 16,
                ..Default::default()
            }
        )
        .unwrap(),
        0xCDAB
    );
    assert_eq!(
        programmer::unary(
            "wordflip",
            0xABCD1234,
            IntegerConfig {
                width: 32,
                ..Default::default()
            }
        )
        .unwrap(),
        0x1234ABCD
    );
}
#[test]
fn programmer_base_width_signed_matrix() {
    for width in [8, 16, 32, 64] {
        for base in [2, 8, 10, 16] {
            for signed in [false, true] {
                let c = IntegerConfig {
                    width,
                    base,
                    signed,
                    arithmetic_shift: false,
                };
                for bits in [0, 1, 42, c.mask(), 1u64 << (width - 1)] {
                    let text = c.format(bits);
                    assert_eq!(
                        programmer::evaluate(&text, c).unwrap(),
                        bits,
                        "{c:?}: {text}"
                    );
                }
                assert_eq!(programmer::binary("+", c.mask(), 1, c).unwrap(), 0);
                assert_eq!(programmer::binary("-", 0, 1, c).unwrap(), c.mask());
                assert_eq!(programmer::binary("<<", 1, width as u64, c).unwrap(), 0);
            }
        }
    }
}
#[test]
fn programmer_error_paths() {
    for input in ["1/0", "1 mod 0", "GG", "1.5", "FFFFFFFFFFFFFFFFF", "(1+2"] {
        assert!(
            programmer::evaluate(input, IntegerConfig::default()).is_err(),
            "{input}"
        );
    }
    assert!(programmer::evaluate(
        "2",
        IntegerConfig {
            base: 2,
            ..Default::default()
        }
    )
    .is_err());
}

#[test]
fn conversion_known_reference_values() {
    for (value, from, to, expected) in [
        (1.0, 5, 0, 0.9144),
        (1.0, 4, 0, 1609.344),
        (1.0, 10, 8, 0.45359237),
        (0.0, 12, 13, 32.0),
        (100.0, 12, 13, 212.0),
        (0.0, 12, 14, 273.15),
        (1.0, 17, 15, 3600.0),
        (1.0, 21, 19, 3.785411784),
        (1.0, 26, 23, 4046.8564224),
    ] {
        close(units::convert(value, from, to).unwrap(), expected);
    }
}
#[test]
fn conversion_every_unit_pair_and_swap_roundtrip() {
    let mut pairs = 0;
    for (from, a) in units::UNITS.iter().enumerate() {
        for (to, b) in units::UNITS.iter().enumerate() {
            if a.category == b.category {
                for value in [0.0, 1.0, 42.5, 1000.0] {
                    let converted = units::convert(value, from, to).unwrap();
                    close(units::convert(converted, to, from).unwrap(), value);
                }
                pairs += 1;
            } else {
                assert!(units::convert(1.0, from, to).is_err());
            }
        }
    }
    assert!(pairs > 100);
    assert!(units::convert(-274.0, 12, 14).is_err());
}

fn rpn(values: &[f64]) -> Calculator {
    let mut c = Calculator::default();
    c.set_mode(Mode::Scientific);
    c.set_rpn(true);
    for value in values {
        c.append(&value.to_string());
        c.enter();
    }
    c
}
#[test]
fn rpn_binary_operations_and_order() {
    for (op, expected) in [
        ("+", 11.0),
        ("-", 5.0),
        ("*", 24.0),
        ("/", 8.0 / 3.0),
        ("^", 512.0),
        ("mod", 2.0),
        ("root", 2.0),
        ("rroot", 3f64.powf(0.125)),
        ("log", 1.892789260714372),
        ("pow", 512.0),
    ] {
        let mut c = rpn(&[8.0, 3.0]);
        c.operator(op);
        assert!(c.error.is_none(), "{op}: {:?}", c.error);
        assert_eq!(c.stack.len(), 1);
        close(c.stack[0], expected);
    }
}
#[test]
fn rpn_enter_duplicate_drop_swap_and_roll() {
    let mut c = rpn(&[1.0, 2.0, 3.0]);
    c.stack_action("swap");
    assert_eq!(c.stack, vec![1.0, 3.0, 2.0]);
    c.stack_action("up");
    assert_eq!(c.stack, vec![3.0, 2.0, 1.0]);
    c.stack_action("down");
    assert_eq!(c.stack, vec![1.0, 3.0, 2.0]);
    c.enter();
    assert_eq!(c.stack, vec![1.0, 3.0, 2.0, 2.0]);
    c.stack_action("drop");
    assert_eq!(c.stack, vec![1.0, 3.0, 2.0]);
    c.apply_function("square");
    assert_eq!(c.stack, vec![1.0, 3.0, 4.0]);
    c.percent();
    close(c.stack[2], 0.04);
}
#[test]
fn rpn_errors_preserve_operands_and_input() {
    let mut c = rpn(&[8.0]);
    c.append("0");
    c.operator("/");
    assert!(c.error.is_some());
    assert_eq!(c.stack, vec![8.0]);
    assert_eq!(c.input, "0");
    let mut c = rpn(&[]);
    c.append("2");
    c.stack_action("swap");
    assert!(c.error.is_some());
    assert!(c.stack.is_empty());
    assert_eq!(c.input, "2");
}
#[test]
fn memory_all_modes() {
    for mode in [Mode::Basic, Mode::Scientific, Mode::Programmer] {
        let mut c = Calculator::default();
        c.set_mode(mode);
        if mode == Mode::Programmer {
            c.set_base(10);
            c.clear();
        }
        c.append("12");
        c.memory("M+");
        c.clear();
        c.append("5");
        c.memory("M-");
        c.clear();
        c.memory("MR");
        c.equals();
        assert_eq!(c.display, "7", "{mode:?}");
        c.memory("MC");
        c.clear();
        c.memory("MR");
        c.equals();
        assert_eq!(c.display, "0", "{mode:?}");
    }
}
#[test]
fn editing_clear_delete_sign_and_repeated_equals() {
    let mut c = Calculator::default();
    c.append("123");
    c.delete();
    assert_eq!(c.input, "12");
    c.change_sign();
    c.equals();
    assert_eq!(c.display, "-12");
    c.clear();
    c.append("2");
    c.operator("+");
    c.append("999");
    c.clear_entry();
    c.append("3");
    c.equals();
    assert_eq!(c.display, "5");
    c.equals();
    assert_eq!(c.display, "8");
    let repeated = c.history.last().unwrap().clone();
    c.restore(&repeated);
    c.equals();
    assert_eq!(c.display, "8");
}
#[test]
fn history_restore_serialization_and_modes() {
    for mode in [Mode::Basic, Mode::Scientific, Mode::Programmer] {
        let mut c = Calculator::default();
        c.set_mode(mode);
        c.append("2+3");
        c.equals();
        let encoded = storage::encode(&c);
        let mut restored = storage::decode(&encoded).unwrap();
        let h = restored.history[0].clone();
        restored.restore(&h);
        restored.equals();
        assert_eq!(restored.display, "5");
        assert_eq!(restored.mode, mode);
    }
}
