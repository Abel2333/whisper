//! Pratt parser plus tokenizer for calculator expressions.

use super::{
    CalculatorError,
    functions::{FunctionKind, constant_value},
};

/// Public entry point for parsing raw expressions under an operation budget.
pub(super) fn parse_expression(expr: &str, max_operations: usize) -> Result<f64, CalculatorError> {
    let tokens = tokenize(expr)?;
    PrattParser::new(tokens, max_operations).parse()
}

/// Lightweight token representation from the lexer.
#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
}

/// Discriminated union of all supported token kinds.
#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Number(f64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    End,
}

/// Convert an expression string into a token stream.
fn tokenize(expr: &str) -> Result<Vec<Token>, CalculatorError> {
    let mut tokens = Vec::new();
    let mut chars = expr.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        match ch {
            c if c.is_whitespace() => continue,
            '+' => tokens.push(Token {
                kind: TokenKind::Plus,
            }),
            '-' => tokens.push(Token {
                kind: TokenKind::Minus,
            }),
            '*' => tokens.push(Token {
                kind: TokenKind::Star,
            }),
            '/' => tokens.push(Token {
                kind: TokenKind::Slash,
            }),
            '^' => tokens.push(Token {
                kind: TokenKind::Caret,
            }),
            '(' => tokens.push(Token {
                kind: TokenKind::LParen,
            }),
            ')' => tokens.push(Token {
                kind: TokenKind::RParen,
            }),
            c if c.is_ascii_digit() || c == '.' => {
                let mut buf = String::new();
                buf.push(c);
                while let Some((_, next_ch)) = chars.peek() {
                    if next_ch.is_ascii_digit() || *next_ch == '.' {
                        buf.push(*next_ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let value = buf
                    .parse::<f64>()
                    .map_err(|_| CalculatorError::NumberParseError { position: idx })?;
                tokens.push(Token {
                    kind: TokenKind::Number(value),
                });
            }
            c if c.is_ascii_alphabetic() => {
                let mut ident = String::new();
                ident.push(c.to_ascii_lowercase());
                while let Some((_, next_ch)) = chars.peek() {
                    if next_ch.is_ascii_alphabetic() {
                        ident.push(next_ch.to_ascii_lowercase());
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    kind: TokenKind::Identifier(ident),
                });
            }
            other => {
                return Err(CalculatorError::InvalidToken {
                    token: other,
                    position: idx,
                });
            }
        }
    }

    tokens.push(Token {
        kind: TokenKind::End,
    });
    Ok(tokens)
}

#[derive(Clone, Copy, Debug)]
enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

const PREFIX_BINDING_POWER: u8 = 9;

/// Pratt parser with explicit operation budget to guard against adversarial input.
struct PrattParser {
    tokens: Vec<Token>,
    pos: usize,
    op_count: usize,
    max_operations: usize,
}

impl PrattParser {
    fn new(tokens: Vec<Token>, max_operations: usize) -> Self {
        Self {
            tokens,
            pos: 0,
            op_count: 0,
            max_operations,
        }
    }

    /// Parse the entire stream and ensure no trailing garbage remains.
    fn parse(mut self) -> Result<f64, CalculatorError> {
        let value = self.parse_expression(0)?;
        if matches!(self.peek_kind(), TokenKind::End) {
            Ok(value)
        } else {
            Err(CalculatorError::UnexpectedToken {
                found: describe_token(self.peek_kind()),
            })
        }
    }

    /// Parse an expression given a minimum binding power.
    fn parse_expression(&mut self, min_bp: u8) -> Result<f64, CalculatorError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => Operator::Add,
                TokenKind::Minus => Operator::Sub,
                TokenKind::Star => Operator::Mul,
                TokenKind::Slash => Operator::Div,
                TokenKind::Caret => Operator::Pow,
                _ => break,
            };

            let (left_bp, right_bp) = infix_binding_power(op);
            if left_bp < min_bp {
                break;
            }

            self.bump();
            self.bump_operation()?;
            let rhs = self.parse_expression(right_bp)?;
            lhs = apply_operator(op, lhs, rhs)?;
        }

        Ok(lhs)
    }

    /// Parse prefix operators, literals, and grouped expressions.
    fn parse_prefix(&mut self) -> Result<f64, CalculatorError> {
        let token = self.bump();
        match token.kind {
            TokenKind::Number(value) => Ok(value),
            TokenKind::Identifier(name) => self.parse_identifier(name),
            TokenKind::Minus => {
                self.bump_operation()?;
                let value = self.parse_expression(PREFIX_BINDING_POWER)?;
                Ok(-value)
            }
            TokenKind::Plus => self.parse_expression(PREFIX_BINDING_POWER),
            TokenKind::LParen => {
                let value = self.parse_expression(0)?;
                self.expect_rparen()?;
                Ok(value)
            }
            TokenKind::RParen => Err(CalculatorError::MismatchedParentheses),
            TokenKind::End => Err(CalculatorError::UnexpectedEndOfInput),
            other => Err(CalculatorError::UnexpectedToken {
                found: describe_token(&other),
            }),
        }
    }

    /// Resolve identifiers as either constants or single-argument functions.
    fn parse_identifier(&mut self, name: String) -> Result<f64, CalculatorError> {
        if let Some(value) = constant_value(&name) {
            return Ok(value);
        }

        if let Some(func) = FunctionKind::from_name(&name) {
            self.expect_lparen()?;
            let arg = self.parse_expression(0)?;
            self.expect_rparen()?;
            self.bump_operation()?;
            return func.apply(arg);
        }

        Err(CalculatorError::UnknownIdentifier { identifier: name })
    }

    fn expect_lparen(&mut self) -> Result<(), CalculatorError> {
        match self.peek_kind() {
            TokenKind::LParen => {
                self.bump();
                Ok(())
            }
            TokenKind::End => Err(CalculatorError::MismatchedParentheses),
            other => Err(CalculatorError::UnexpectedToken {
                found: describe_token(other),
            }),
        }
    }

    fn expect_rparen(&mut self) -> Result<(), CalculatorError> {
        match self.peek_kind() {
            TokenKind::RParen => {
                self.bump();
                Ok(())
            }
            TokenKind::End => Err(CalculatorError::MismatchedParentheses),
            other => Err(CalculatorError::UnexpectedToken {
                found: describe_token(other),
            }),
        }
    }

    fn bump(&mut self) -> Token {
        let token = self
            .tokens
            .get(self.pos)
            .cloned()
            .unwrap_or_else(|| self.tokens.last().cloned().unwrap());
        self.pos += 1;
        token
    }

    fn peek_kind(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::End)
    }

    fn bump_operation(&mut self) -> Result<(), CalculatorError> {
        self.op_count += 1;
        if self.op_count > self.max_operations {
            Err(CalculatorError::OperationLimitExceeded {
                limit: self.max_operations,
            })
        } else {
            Ok(())
        }
    }
}

fn infix_binding_power(op: Operator) -> (u8, u8) {
    match op {
        Operator::Add | Operator::Sub => (1, 2),
        Operator::Mul | Operator::Div => (3, 4),
        Operator::Pow => (6, 5),
    }
}

fn apply_operator(op: Operator, left: f64, right: f64) -> Result<f64, CalculatorError> {
    let result = match op {
        Operator::Add => left + right,
        Operator::Sub => left - right,
        Operator::Mul => left * right,
        Operator::Div => {
            if right == 0.0 {
                return Err(CalculatorError::DomainViolation(
                    "division by zero is undefined".into(),
                ));
            }
            left / right
        }
        Operator::Pow => left.powf(right),
    };

    Ok(result)
}

fn describe_token(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Number(_) => "number".into(),
        TokenKind::Identifier(name) => format!("identifier '{name}'"),
        TokenKind::Plus => "'+'".into(),
        TokenKind::Minus => "'-'".into(),
        TokenKind::Star => "'*'".into(),
        TokenKind::Slash => "'/'".into(),
        TokenKind::Caret => "'^'".into(),
        TokenKind::LParen => "'('".into(),
        TokenKind::RParen => "')'".into(),
        TokenKind::End => "end of input".into(),
    }
}
