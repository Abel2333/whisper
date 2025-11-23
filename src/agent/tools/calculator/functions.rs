//! Math helpers used by the calculator parser.

use super::CalculatorError;
use std::f64::consts::{E, PI};

/// Supported single-argument functions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum FunctionKind {
    Sqrt,
    Ln,
    Log10,
    Exp,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Abs,
}

impl FunctionKind {
    /// Map a lowercase identifier to a known function.
    pub(super) fn from_name(name: &str) -> Option<Self> {
        match name {
            "sqrt" => Some(FunctionKind::Sqrt),
            "ln" => Some(FunctionKind::Ln),
            "log" => Some(FunctionKind::Log10),
            "exp" => Some(FunctionKind::Exp),
            "sin" => Some(FunctionKind::Sin),
            "cos" => Some(FunctionKind::Cos),
            "tan" => Some(FunctionKind::Tan),
            "asin" => Some(FunctionKind::Asin),
            "acos" => Some(FunctionKind::Acos),
            "atan" => Some(FunctionKind::Atan),
            "abs" => Some(FunctionKind::Abs),
            _ => None,
        }
    }

    /// Apply the function to its argument, enforcing domain constraints.
    pub(super) fn apply(self, input: f64) -> Result<f64, CalculatorError> {
        let result = match self {
            FunctionKind::Sqrt => {
                if input < 0.0 {
                    return Err(CalculatorError::DomainViolation(
                        "sqrt() expects a non-negative input".into(),
                    ));
                }
                input.sqrt()
            }
            FunctionKind::Ln => {
                if input <= 0.0 {
                    return Err(CalculatorError::DomainViolation(
                        "ln() expects input greater than 0".into(),
                    ));
                }
                input.ln()
            }
            FunctionKind::Log10 => {
                if input <= 0.0 {
                    return Err(CalculatorError::DomainViolation(
                        "log() expects input greater than 0".into(),
                    ));
                }
                input.log10()
            }
            FunctionKind::Exp => input.exp(),
            FunctionKind::Sin => input.sin(),
            FunctionKind::Cos => input.cos(),
            FunctionKind::Tan => input.tan(),
            FunctionKind::Asin => {
                if !(-1.0..=1.0).contains(&input) {
                    return Err(CalculatorError::DomainViolation(
                        "asin() expects input between -1 and 1".into(),
                    ));
                }
                input.asin()
            }
            FunctionKind::Acos => {
                if !(-1.0..=1.0).contains(&input) {
                    return Err(CalculatorError::DomainViolation(
                        "acos() expects input between -1 and 1".into(),
                    ));
                }
                input.acos()
            }
            FunctionKind::Atan => input.atan(),
            FunctionKind::Abs => input.abs(),
        };
        Ok(result)
    }
}

/// Resolve named mathematical constants.
pub(super) fn constant_value(name: &str) -> Option<f64> {
    match name {
        "pi" => Some(PI),
        "tau" => Some(2.0 * PI),
        "e" => Some(E),
        _ => None,
    }
}
