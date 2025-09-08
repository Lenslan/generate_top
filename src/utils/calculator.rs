use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::Chars;
use rust_decimal::{Decimal, MathematicalOps};

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Add(Box<Node>, Box<Node>),
    Subtract(Box<Node>, Box<Node>),
    Multiply(Box<Node>, Box<Node>),
    Divide(Box<Node>, Box<Node>),
    Power(Box<Node>, Box<Node>),
    Negative(Box<Node>),
    Number(Decimal),
}

impl Node {
    pub fn eval(&self) -> Decimal {
        use Node::*;

        match self {
            Add(left, right) => left.eval() + right.eval(),
            Subtract(left, right) => left.eval() - right.eval(),
            Multiply(left, right) => left.eval() * right.eval(),
            Divide(left, right) => left.eval() / right.eval(),
            Power(left, right) => left.eval().powd(right.eval()),
            Negative(expr) => -expr.eval(),
            Number(n) => *n,
        }
    }
}

pub type CalcResult<T> = Result<T, CalcError>;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum CalcError {
    #[error("非法字符: {0}")]
    UnexpectedChar(char),
    #[error("无效运算符: {0}")]
    InvalidOperator(String)
}

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(expression: &'a str) -> CalcResult<Self> {
        let mut tokenizer = Tokenizer::new(expression);
        let current_token = tokenizer.next().ok_or_else(|| {
            CalcError::UnexpectedChar(tokenizer.get_unexpected_char().unwrap())
        })?;

        Ok(Parser {
            tokenizer,
            current_token,
        })
    }

    pub fn parse(&mut self) -> CalcResult<Node> {
        self.parse_expression(OperatorPrecedence::Default)
    }
}

impl<'a> Parser<'a> {
    fn next_token(&mut self) -> CalcResult<()> {
        self.current_token = self.tokenizer.next().ok_or_else(|| {
            CalcError::UnexpectedChar(self.tokenizer.get_unexpected_char().unwrap())
        })?;

        Ok(())
    }

    fn parse_expression(&mut self, precedence: OperatorPrecedence) -> CalcResult<Node> {
        let mut expr = self.parse_number_or_expression()?;

        while precedence < self.current_token.get_precedence() {
            expr = self.parse_binary_expression(expr)?;
        }

        Ok(expr)
    }

    fn parse_binary_expression(&mut self, left_expr: Node) -> CalcResult<Node> {
        match self.current_token {
            Token::Add => {
                self.next_token()?;
                let right_expr = self.parse_expression(OperatorPrecedence::AddOrSubtract)?;
                Ok(Node::Add(Box::new(left_expr), Box::new(right_expr)))
            }
            Token::Subtract => {
                self.next_token()?;
                let right_expr = self.parse_expression(OperatorPrecedence::AddOrSubtract)?;
                Ok(Node::Subtract(Box::new(left_expr), Box::new(right_expr)))
            }
            Token::Multiply => {
                self.next_token()?;
                let right_expr = self.parse_expression(OperatorPrecedence::MultiplyOrDivide)?;
                Ok(Node::Multiply(Box::new(left_expr), Box::new(right_expr)))
            }
            Token::Divide => {
                self.next_token()?;
                let right_expr = self.parse_expression(OperatorPrecedence::MultiplyOrDivide)?;
                Ok(Node::Divide(Box::new(left_expr), Box::new(right_expr)))
            }
            Token::Caret => {
                self.next_token()?;
                let right_expr = self.parse_expression(OperatorPrecedence::Power)?;
                Ok(Node::Power(Box::new(left_expr), Box::new(right_expr)))
            }
            _ => unreachable!()
        }
    }

    fn parse_number_or_expression(&mut self) -> CalcResult<Node> {
        match self.current_token {
            Token::Number(n) => {
                self.next_token()?;
                Ok(Node::Number(n))
            }
            Token::Subtract => {
                self.next_token()?;
                let expr = self.parse_expression(OperatorPrecedence::Negative)?;
                Ok(Node::Negative(Box::new(expr)))
            }
            Token::LeftParen => {
                self.next_token()?;
                let expr = self.parse_expression(OperatorPrecedence::Default)?;

                if self.current_token != Token::RightParen {
                    if self.current_token == Token::EOF {
                        return Err(
                            CalcError::InvalidOperator(
                                String::from("不完整的运算表达式")
                            )
                        );
                    }

                    return Err(
                        CalcError::InvalidOperator(
                            format!("期望 ')', 但是遇到 '{}'", self.current_token)
                        )
                    );
                }

                self.next_token()?;
                Ok(expr)
            },
            _ => {
                if self.current_token == Token::EOF {
                    return Err(
                        CalcError::InvalidOperator(
                            String::from("不完整的运算表达式")
                        )
                    );
                }

                Err(
                    CalcError::InvalidOperator(
                        format!("期望数字或表达式, 但是遇到 '{}'", self.current_token)
                    )
                )
            }
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token {
    Add,
    Subtract,
    Multiply,
    Divide,
    Caret,
    LeftParen,
    RightParen,
    Number(Decimal),
    EOF
}

impl Token {
    pub fn get_precedence(&self) -> OperatorPrecedence {
        use Token::*;
        use OperatorPrecedence::*;

        match self {
            Add | Subtract => AddOrSubtract,
            Multiply | Divide => MultiplyOrDivide,
            Caret => Power,
            _ => Default
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Token::*;

        match self {
            Add => write!(f, "+"),
            Subtract => write!(f, "-"),
            Multiply => write!(f, "*"),
            Divide => write!(f, "/"),
            Caret => write!(f, "^"),
            LeftParen => write!(f, "("),
            RightParen => write!(f, ")"),
            Number(n) => write!(f, "{}", n),
            EOF => write!(f, "EOF")
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum OperatorPrecedence {
    Default,
    AddOrSubtract,
    MultiplyOrDivide,
    Power,
    Negative
}

pub struct Tokenizer<'a> {
    expression: Peekable<Chars<'a>>,
    reached_end: bool,
    unexpected_char: Option<char>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(expression: &'a str) -> Self {
        Self {
            expression: expression.chars().peekable(),
            reached_end: false,
            unexpected_char: None,
        }
    }

    pub fn get_unexpected_char(&self) -> Option<char> {
        self.unexpected_char
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reached_end {
            return None;
        }

        let next_chr = self.expression.next();
        match next_chr {
            Some(chr) if chr.is_numeric() => {
                let mut number = String::from(chr);

                // while let Some(next) = self.expression.peek() {
                //     if next.is_numeric() {
                //         number.push(self.expression.next().unwrap());
                //     } else {
                //         break;
                //     }
                // }

                while let Some(next) = self.expression.next_if(|c| c.is_numeric()) {
                    number.push(next);
                }

                Some(Token::Number(number.parse().unwrap()))
            }
            Some(chr) if chr.is_whitespace() => {
                while let Some(_) = self.expression.next_if(|c| c.is_whitespace()) {}

                self.next()
            }
            Some('+') => Some(Token::Add),
            Some('-') => Some(Token::Subtract),
            Some('*') => Some(Token::Multiply),
            Some('/') => Some(Token::Divide),
            Some('^') => Some(Token::Caret),
            Some('(') => Some(Token::LeftParen),
            Some(')') => Some(Token::RightParen),
            None => {
                self.reached_end = true;
                Some(Token::EOF)
            }
            Some(chr) => {
                self.unexpected_char = Some(chr);
                None
            }
        }
    }
}

pub trait StrCalc {
    fn calculate(&self) ->usize;
}

impl StrCalc for String {
    fn calculate(&self) -> usize {
        let mut parser = Parser::new(self).unwrap_or_else(|e| panic!("{}", e));
        let ast = parser.parse().unwrap_or_else(|e| panic!("{}", e));

        usize::try_from(ast.eval()).unwrap()
    }

}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;
    use super::*;

    #[test]
    fn test_calculate() {
        // assert_eq!(String::from("1 + 2").calculate().unwrap(), dec!(3));
        // assert_eq!(String::from("1 - 2").calculate().unwrap(), dec!(-1));
        // assert_eq!(String::from("1 * 2").calculate().unwrap(), dec!(2));
        // assert_eq!(String::from("1 / 2").calculate().unwrap(), dec!(0.5));
        // assert_eq!(String::from("1 ^ 2").calculate().unwrap(), dec!(1));
        // assert_eq!(String::from("-1").calculate().unwrap(), dec!(-1));
        // assert_eq!(String::from("-1 + 2").calculate().unwrap(), dec!(1));
        // assert_eq!(String::from("-1 - 2").calculate().unwrap(), dec!(-3));
        // assert_eq!(String::from("-1 * 2").calculate().unwrap(), dec!(-2));
        // assert_eq!(String::from("-1 / 2").calculate().unwrap(), dec!(-0.5));
        // assert_eq!(String::from("-1 ^ 2").calculate().unwrap(), dec!(1));
        // assert_eq!(String::from("3 - (2+3) * 2 - 1 * (-3 *3)").calculate().unwrap(), dec!(2));
    }
}
