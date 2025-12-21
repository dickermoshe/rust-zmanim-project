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

/// Evaluate a cubic polynomial at `x`.
///
/// Interprets the arguments as coefficients of:
///
/// \(`a_3` x^3 + `a_2` x^2 + `a_1` x + `a_0`\)
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

/// Computes a polynomial using Horner's method for numerical stability.
///
/// Coefficients are ordered [a₀, a₁, a₂, ...] for a₀ + a₁x + a₂x² + ...
pub(crate) fn polynomial(coeffs: &[f64], x: f64) -> f64 {
    let Some(&last) = coeffs.last() else {
        return 0.0;
    };

    // Horner's method: reverse iteration for numerical stability
    let mut result = last;
    for &coeff in coeffs.iter().rev().skip(1) {
        result = result.mul_add(x, coeff);
    }
    result
}

/// Computes the floored modulo operation (Python-style modulo).
///
/// Unlike Rust's `%` operator which can return negative values, this function
/// always returns a non-negative result in the range [0, m). This matches
/// Python's modulo behavior and is useful for normalizing values to a positive range.
///
/// # Arguments
///
/// * `x` - The dividend
/// * `m` - The modulus (must be positive)
///
/// # Returns
///
/// The remainder `x mod m` in the range 0, m)
///
/// # Examples
///
/// ```
/// # fn floored_mod(x: f64, m: f64) -> f64 { ((x % m) + m) % m }
/// assert_eq!(floored_mod(7.0, 3.0), 1.0);
/// assert_eq!(floored_mod(-7.0, 3.0), 2.0);  // Unlike -7 % 3 which would be -1
/// assert_eq!(floored_mod(0.5, 1.0), 0.5);
/// assert_eq!(floored_mod(1.5, 1.0), 0.5);
/// ```
pub(crate) fn floored_mod(x: f64, m: f64) -> f64 {
    ((x % m) + m) % m
}
