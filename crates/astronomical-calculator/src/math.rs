#[allow(unused_imports)]
use core_maths::CoreFloat;

/// Normalizes an angle in degrees to the range [0, 360).
///
/// This function takes any angle value (positive or negative) and converts it
/// to an equivalent angle in the range [0, 360). Values outside this range
/// are wrapped around using modulo arithmetic.
///
/// # Arguments
///
/// * `degrees` - The angle in degrees to normalize
///
/// # Returns
///
/// The normalized angle in degrees, in the range [0, 360)
pub(crate) fn normalize_degrees_360(degrees: f64) -> f64 {
    let degrees = degrees / 360.0;
    let mut limited = 360.0 * (degrees - degrees.floor());
    if limited < 0.0 {
        limited += 360.0;
    }
    limited
}

/// Normalizes an angle in degrees to the range [-180, 180].
///
/// This function takes any angle value (positive or negative) and converts it
/// to an equivalent angle in the range [-180, 180]. Values outside this range
/// are wrapped around using modulo arithmetic, with angles greater than 180
/// being converted to their negative equivalent (e.g., 270° becomes -90°).
///
/// # Arguments
///
/// * `degrees` - The angle in degrees to normalize
///
/// # Returns
///
/// The normalized angle in degrees, in the range [-180, 180]
pub(crate) fn normalize_degrees_180pm(degrees: f64) -> f64 {
    let degrees = degrees / 360.0;
    let mut limited = 360.0 * (degrees - degrees.floor());
    if limited < -180.0 {
        limited += 360.0;
    } else if limited > 180.0 {
        limited -= 360.0;
    }
    limited
}

/// Normalizes an angle in degrees to the range [0, 180].
///
/// This function takes any angle value (positive or negative) and converts it
/// to an equivalent angle in the range [0, 180]. Values outside this range
/// are wrapped around using modulo arithmetic.
///
/// # Arguments
///
/// * `degrees` - The angle in degrees to normalize
///
/// # Returns
///
/// The normalized angle in degrees, in the range [0, 180]
pub(crate) fn normalize_degrees_180(degrees: f64) -> f64 {
    let degrees = degrees / 180.0;
    let mut limited = 180.0 * (degrees - degrees.floor());
    if limited < 0.0 {
        limited += 180.0;
    }
    limited
}

/// Normalizes a value to the unit interval [0, 1).
///
/// This function takes any real number and converts it to an equivalent value
/// in the range [0, 1) by extracting the fractional part. Values outside this
/// range are wrapped around using modulo arithmetic.
///
/// # Arguments
///
/// * `value` - The value to normalize
///
/// # Returns
///
/// The normalized value in the range [0, 1)
pub(crate) fn normalize_unit_interval(value: f64) -> f64 {
    let mut limited = value - value.floor();
    if limited < 0.0 {
        limited += 1.0;
    }
    limited
}

/// Evaluate a cubic polynomial at `x`.
///
/// Interprets the arguments as coefficients of:
///
/// \(a_3 x^3 + a_2 x^2 + a_1 x + a_0\)
///
/// using Horner's method for numerical stability and efficiency.
///
/// # Arguments
///
/// * `a3` - Coefficient for \(x^3\)
/// * `a2` - Coefficient for \(x^2\)
/// * `a1` - Coefficient for \(x^1\)
/// * `a0` - Constant term
/// * `x`  - Point at which to evaluate the polynomial
///
/// # Returns
///
/// The value of the cubic polynomial evaluated at `x`
pub(crate) fn eval_cubic(a3: f64, a2: f64, a1: f64, a0: f64, x: f64) -> f64 {
    ((a3 * x + a2) * x + a1) * x + a0
}
