use std::ops::{Add, Sub};
use crate::utils::calculator::StrCalc;
use crate::verilog::parameter::Param;

#[derive(Debug)]
pub enum Width {
    RawWidth(usize),
    LiteralWidth(String),
}

impl Default for Width {
    fn default() -> Self {
        Width::RawWidth(0)
    }
}

impl Width {
    fn width(&self, param: &Vec<Param>) -> usize {
        match self {
            Width::RawWidth(x) => {*x}
            Width::LiteralWidth(s) => {
                let mut temp = s.clone();
                for p in param {
                    temp = temp.replace(
                        &p.name,
                        &format!("{}", p.value),
                    )
                }
                let res = temp.calculate();
                if res.is_ok() {
                    res.unwrap()
                } else {
                    log::warn!("Failed to calculate width: {} in literal {}", res.err().unwrap(), s);
                    0
                }

            }
        }
    }
}

impl Add for Width {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use Width::*;
        match (self,rhs) {
            (RawWidth(x), RawWidth(y)) => RawWidth(x + y),
            (LiteralWidth(x), RawWidth(y)) => LiteralWidth(format!("{} + {}", x, y)),
            (RawWidth(x), LiteralWidth(y)) => LiteralWidth(format!("{} + {}", x, y)),
            (LiteralWidth(x), LiteralWidth(y)) => LiteralWidth(format!("{} + {}", x, y)),
        }
    }
}

impl Sub for Width {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        use Width::*;
        match (self,rhs) {
            (RawWidth(x), RawWidth(y)) => RawWidth(x - y),
            (LiteralWidth(x), RawWidth(y)) => LiteralWidth(format!("{} - {}", x, y)),
            (RawWidth(x), LiteralWidth(y)) => LiteralWidth(format!("{} - {}", x, y)),
            (LiteralWidth(x), LiteralWidth(y)) => LiteralWidth(format!("{} - {}", x, y)),
        }
    }
}

impl Add<usize> for Width {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        use Width::*;
        match self {
            RawWidth(x) => RawWidth(x + rhs),
            LiteralWidth(x) => LiteralWidth(format!("{} + {}", x, rhs)),
        }
    }
}

impl Sub<usize> for Width {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        use Width::*;
        match self {
            RawWidth(x) => RawWidth(x - rhs),
            LiteralWidth(x) => LiteralWidth(format!("{} - {}", x, rhs)),
        }
    }
}

impl Add<Width> for usize {
    type Output = Width;
    fn add(self, rhs: Width) -> Self::Output {
        use Width::*;
        match rhs {
            RawWidth(x) => RawWidth(x + self),
            LiteralWidth(x) => LiteralWidth(format!("{} + {}", x, self)),
        }
    }
}

impl Sub<Width> for usize {
    type Output = Width;
    fn sub(self, rhs: Width) -> Self::Output {
        use Width::*;
        match rhs {
            RawWidth(x) => RawWidth(x - self),
            LiteralWidth(x) => LiteralWidth(format!("{} - {}", x, self)),
        }
    }
}