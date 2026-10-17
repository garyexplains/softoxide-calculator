use crate::expression::{error, finite, CalcError};
#[derive(Clone, Copy, Debug)]
pub struct Unit {
    pub name: &'static str,
    pub category: &'static str,
    pub scale: f64,
    pub offset: f64,
}
pub const UNITS: &[Unit] = &[
    Unit {
        name: "Meters",
        category: "Length",
        scale: 1.0,
        offset: 0.0,
    },
    Unit {
        name: "Kilometers",
        category: "Length",
        scale: 1000.0,
        offset: 0.0,
    },
    Unit {
        name: "Centimeters",
        category: "Length",
        scale: 0.01,
        offset: 0.0,
    },
    Unit {
        name: "Millimeters",
        category: "Length",
        scale: 0.001,
        offset: 0.0,
    },
    Unit {
        name: "Miles",
        category: "Length",
        scale: 1609.344,
        offset: 0.0,
    },
    Unit {
        name: "Yards",
        category: "Length",
        scale: 0.9144,
        offset: 0.0,
    },
    Unit {
        name: "Feet",
        category: "Length",
        scale: 0.3048,
        offset: 0.0,
    },
    Unit {
        name: "Inches",
        category: "Length",
        scale: 0.0254,
        offset: 0.0,
    },
    Unit {
        name: "Kilograms",
        category: "Mass",
        scale: 1.0,
        offset: 0.0,
    },
    Unit {
        name: "Grams",
        category: "Mass",
        scale: 0.001,
        offset: 0.0,
    },
    Unit {
        name: "Pounds",
        category: "Mass",
        scale: 0.45359237,
        offset: 0.0,
    },
    Unit {
        name: "Ounces",
        category: "Mass",
        scale: 0.028349523125,
        offset: 0.0,
    },
    Unit {
        name: "Celsius",
        category: "Temperature",
        scale: 1.0,
        offset: 273.15,
    },
    Unit {
        name: "Fahrenheit",
        category: "Temperature",
        scale: 5.0 / 9.0,
        offset: 255.3722222222222,
    },
    Unit {
        name: "Kelvin",
        category: "Temperature",
        scale: 1.0,
        offset: 0.0,
    },
    Unit {
        name: "Seconds",
        category: "Time",
        scale: 1.0,
        offset: 0.0,
    },
    Unit {
        name: "Minutes",
        category: "Time",
        scale: 60.0,
        offset: 0.0,
    },
    Unit {
        name: "Hours",
        category: "Time",
        scale: 3600.0,
        offset: 0.0,
    },
    Unit {
        name: "Days",
        category: "Time",
        scale: 86400.0,
        offset: 0.0,
    },
    Unit {
        name: "Liters",
        category: "Volume",
        scale: 1.0,
        offset: 0.0,
    },
    Unit {
        name: "Milliliters",
        category: "Volume",
        scale: 0.001,
        offset: 0.0,
    },
    Unit {
        name: "US gallons",
        category: "Volume",
        scale: 3.785411784,
        offset: 0.0,
    },
    Unit {
        name: "US fluid ounces",
        category: "Volume",
        scale: 0.0295735295625,
        offset: 0.0,
    },
    Unit {
        name: "Square meters",
        category: "Area",
        scale: 1.0,
        offset: 0.0,
    },
    Unit {
        name: "Square feet",
        category: "Area",
        scale: 0.09290304,
        offset: 0.0,
    },
    Unit {
        name: "Hectares",
        category: "Area",
        scale: 10000.0,
        offset: 0.0,
    },
    Unit {
        name: "Acres",
        category: "Area",
        scale: 4046.8564224,
        offset: 0.0,
    },
];
pub fn convert(value: f64, from: usize, to: usize) -> Result<f64, CalcError> {
    let a = UNITS.get(from).ok_or_else(|| error("Unknown input unit"))?;
    let b = UNITS.get(to).ok_or_else(|| error("Unknown result unit"))?;
    if a.category != b.category {
        return Err(error("Units have different dimensions"));
    }
    let canonical = value * a.scale + a.offset;
    if a.category == "Temperature" && canonical < -1e-10 {
        return Err(error("Temperature is below absolute zero"));
    }
    finite((canonical - b.offset) / b.scale)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn affine_and_dimension_validation() {
        assert!((convert(0.0, 12, 13).unwrap() - 32.0).abs() < 1e-10);
        assert!((convert(32.0, 13, 12).unwrap()).abs() < 1e-10);
        assert_eq!(convert(1.0, 1, 0).unwrap(), 1000.0);
        assert!(convert(1.0, 0, 8).is_err());
        assert!(convert(-274.0, 12, 14).is_err());
    }
}
