use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};
use crate::utils::calculator::StrCalc;
use crate::verilog::parameter::Param;

#[derive(Debug, Clone)]
pub enum Width {
    RawWidth(usize),
    LiteralWidth(String, usize),
}

impl Default for Width {
    fn default() -> Self {
        Width::RawWidth(0)
    }
}

impl From<&str> for Width {
    fn from(value: &str) -> Self {
        Self::LiteralWidth(value.into(), 0)
    }
}

impl From<String> for Width {
    fn from(value: String) -> Self {
        Self::LiteralWidth(value, 0)
    }
}

impl From<usize> for Width {
    fn from(value: usize) -> Self {
        Self::RawWidth(value)
    }
}

impl Display for Width {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Width::RawWidth(x) => {write!(f, "{}", x)}
            Width::LiteralWidth(x, _) => {write!(f, "{}", x)}
        }
    }
}

impl Width {
    pub fn width_from(&self, param: &Vec<Param>) -> Self {
        match self {
            Width::RawWidth(x) => {Width::RawWidth(*x)}
            Width::LiteralWidth(s, _) => {
                let mut temp = s.clone();
                for p in param {
                    temp = temp.replace(
                        &p.name,
                        &format!("{}", p.value),
                    )
                }
                let res = temp.calculate();
                let t = if res.is_ok() {
                    res.unwrap()
                } else {
                    log::warn!("Failed to calculate width: {} in literal {}", res.err().unwrap(), s);
                    0
                };
                Width::LiteralWidth(s.clone(), t)
            }
        }
    }

    pub fn width(&self) -> usize {
        match self {
            Width::RawWidth(x) => *x,
            Width::LiteralWidth(_, v) => *v
        }
    }

    pub fn is_literal(&self) -> bool {
        match self {
            Width::RawWidth(_) => {false}
            Width::LiteralWidth(_, _) => {true}
        }
    }
}

impl Add for Width {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use Width::*;
        match (self,rhs) {
            (RawWidth(x), RawWidth(y)) => RawWidth(x + y),
            (LiteralWidth(x, _), RawWidth(y)) => LiteralWidth(format!("{} + {}", x, y), 0),
            (RawWidth(x), LiteralWidth(y, _)) => LiteralWidth(format!("{} + {}", x, y), 0),
            (LiteralWidth(x, _), LiteralWidth(y, _)) => LiteralWidth(format!("{} + {}", x, y), 0),
        }
    }
}

impl Sub for Width {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        use Width::*;
        match (self,rhs) {
            (RawWidth(x), RawWidth(y)) => RawWidth(x - y),
            (LiteralWidth(x, _), RawWidth(y)) => LiteralWidth(format!("{} - {}", x, y), 0),
            (RawWidth(x), LiteralWidth(y, _)) => LiteralWidth(format!("{} - {}", x, y), 0),
            (LiteralWidth(x, _), LiteralWidth(y, _)) => LiteralWidth(format!("{} - {}", x, y), 0),
        }
    }
}

impl Add<usize> for Width {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        use Width::*;
        match self {
            RawWidth(x) => RawWidth(x + rhs),
            LiteralWidth(x, _) => LiteralWidth(format!("{} + {}", x, rhs), 0),
        }
    }
}

impl Sub<usize> for Width {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        use Width::*;
        match self {
            RawWidth(x) => RawWidth(x - rhs),
            LiteralWidth(x, _) => LiteralWidth(format!("{} - {}", x, rhs), 0),
        }
    }
}

impl Add<Width> for usize {
    type Output = Width;
    fn add(self, rhs: Width) -> Self::Output {
        use Width::*;
        match rhs {
            RawWidth(x) => RawWidth(x + self),
            LiteralWidth(x, _) => LiteralWidth(format!("{} + {}", x, self), 0),
        }
    }
}

impl Sub<Width> for usize {
    type Output = Width;
    fn sub(self, rhs: Width) -> Self::Output {
        use Width::*;
        match rhs {
            RawWidth(x) => RawWidth(x - self),
            LiteralWidth(x, _) => LiteralWidth(format!("{} - {}", x, self), 0),
        }
    }
}