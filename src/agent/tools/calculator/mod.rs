mod functions;
mod parser;

use parser::parse_expression;

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use thiserror::Error;

/// Runtime configuration knobs for the calculator tool.
#[derive(Clone, Debug)]
pub struct CalculatorConfig {
    /// Maximum length (characters) accepted per expression.
    pub max_expression_len: usize,
    /// Maximum number of operators/functions evaluated per request.
    pub max_operations: usize,
    /// Default decimal precision when the caller does not request one.
    pub default_precision: u8,
    /// Upper bound on requested precision to keep output deterministic.
    pub max_precision: u8,
}

impl Default for CalculatorConfig {
    fn default() -> Self {
        Self {
            max_expression_len: 512,
            max_operations: 128,
            default_precision: 6,
            max_precision: 12,
        }
    }
}

/// LLM-facing calculator tool that evaluates deterministic expressions.
#[derive(Debug, Default, Clone)]
pub struct CalculatorTool {
    config: CalculatorConfig,
}

impl CalculatorTool {
    /// Construct a tool with custom limits.
    pub fn new(config: CalculatorConfig) -> Self {
        Self { config }
    }

    /// Parse and evaluate the provided expression string.
    fn evaluate_expression(&self, expr: &str) -> Result<f64, CalculatorError> {
        let expression = expr.trim();
        if expression.is_empty() {
            return Err(CalculatorError::EmptyExpression);
        }

        if expression.len() > self.config.max_expression_len {
            return Err(CalculatorError::ExpressionTooLong {
                len: expression.len(),
                max: self.config.max_expression_len,
            });
        }

        parse_expression(expression, self.config.max_operations)
    }

    /// Resolve either the caller-provided precision or the default.
    fn clamp_precision(&self, requested: Option<u8>) -> Result<u8, CalculatorError> {
        let precision = requested.unwrap_or(self.config.default_precision);
        if precision > self.config.max_precision {
            return Err(CalculatorError::PrecisionTooHigh {
                requested: precision,
                max: self.config.max_precision,
            });
        }
        Ok(precision)
    }

    /// Render a value according to the caller's formatting preferences.
    fn format_value(&self, value: f64, precision: u8, scientific: bool) -> String {
        if scientific {
            format!("{value:.precision$e}", precision = precision as usize)
        } else {
            format!("{value:.precision$}", precision = precision as usize)
        }
    }
}

/// Input payload for the calculator tool.
#[derive(Debug, Deserialize)]
pub struct CalculatorArgs {
    pub expression: String,
    #[serde(default)]
    pub precision: Option<u8>,
    #[serde(default)]
    pub scientific: Option<bool>,
}

/// Structured response from the calculator tool.
#[derive(Debug, Serialize)]
pub struct CalculatorResult {
    pub expression: String,
    pub value: f64,
    pub display: String,
}

impl Tool for CalculatorTool {
    const NAME: &'static str = "calculator";

    type Error = CalculatorError;
    type Args = CalculatorArgs;
    type Output = CalculatorResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        let max_precision = self.config.max_precision;
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Evaluate arithmetic expressions with support for +, -, *, /, ^, parentheses, and functions like sqrt(), log(), sin().".into(),
            parameters: json!({
                "type": "object",
                "required": ["expression"],
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Mathematical expression to evaluate. Supported constants: pi, tau, e."
                    },
                    "precision": {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": max_precision,
                        "description": "Optional number of decimal places (defaults to 6)."
                    },
                    "scientific": {
                        "type": "boolean",
                        "description": "Format the result using scientific notation when true."
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let precision = self.clamp_precision(args.precision)?;
        let value = self.evaluate_expression(&args.expression)?;

        if !value.is_finite() {
            return Err(CalculatorError::NonFiniteResult(value));
        }

        let display = self.format_value(value, precision, args.scientific.unwrap_or(false));
        Ok(CalculatorResult {
            expression: args.expression,
            value,
            display,
        })
    }
}

/// Unified error type for the calculator tool and parser.
#[derive(Debug, Error)]
pub enum CalculatorError {
    #[error("expression is empty")]
    EmptyExpression,
    #[error("expression length {len} exceeds limit {max}")]
    ExpressionTooLong { len: usize, max: usize },
    #[error("invalid token '{token}' at position {position}")]
    InvalidToken { token: char, position: usize },
    #[error("failed to parse number near position {position}")]
    NumberParseError { position: usize },
    #[error("unknown identifier '{identifier}'")]
    UnknownIdentifier { identifier: String },
    #[error("mismatched parentheses")]
    MismatchedParentheses,
    #[error("too many operations (limit {limit})")]
    OperationLimitExceeded { limit: usize },
    #[error("precision {requested} exceeds maximum {max}")]
    PrecisionTooHigh { requested: u8, max: u8 },
    #[error("calculation resulted in a non-finite value ({0})")]
    NonFiniteResult(f64),
    #[error("could not fully evaluate expression: {0}")]
    EvaluationFailed(&'static str),
    #[error("unexpected token: {found}")]
    UnexpectedToken { found: String },
    #[error("unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("{0}")]
    DomainViolation(String),
}

impl fmt::Display for CalculatorResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.expression, self.display)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_basic_expression() {
        let tool = CalculatorTool::default();
        let value = tool.evaluate_expression("1 + 2 * 3").unwrap();
        assert!((value - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn handles_parentheses_and_functions() {
        let tool = CalculatorTool::default();
        let value = tool.evaluate_expression("sqrt(16) + sin(pi / 2)").unwrap();
        assert!((value - 5.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_unknown_identifier() {
        let tool = CalculatorTool::default();
        let err = tool.evaluate_expression("foo(2)").unwrap_err();
        assert!(matches!(err, CalculatorError::UnknownIdentifier { .. }));
    }

    #[test]
    fn clamps_precision() {
        let tool = CalculatorTool::default();
        let err = tool.clamp_precision(Some(64)).unwrap_err();
        assert!(matches!(err, CalculatorError::PrecisionTooHigh { .. }));
    }
}
