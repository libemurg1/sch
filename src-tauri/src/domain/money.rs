use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money(i64);

impl Money {
    pub fn zero() -> Self {
        Money(0)
    }

    pub fn from_cents(cents: i64) -> Self {
        Money(cents)
    }

    pub fn cents(&self) -> i64 {
        self.0
    }

    pub fn add(self, other: Money) -> Money {
        Money(self.0 + other.0)
    }

    pub fn sub(self, other: Money) -> Money {
        Money(self.0 - other.0)
    }

    pub fn mul_percent(self, basis_points: i64) -> Money {
        // basis points * cents / 10000, round half up
        let num = self.0 * basis_points;
        let result = if num >= 0 { (num + 5000) / 10000 } else { (num - 5000) / 10000 };
        Money(result)
    }

    pub fn is_positive(self) -> bool {
        self.0 > 0
    }

    pub fn is_negative(self) -> bool {
        self.0 < 0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn min(self, other: Money) -> Money {
        if self.0 < other.0 { self } else { other }
    }

    pub fn max(self, other: Money) -> Money {
        if self.0 > other.0 { self } else { other }
    }

    pub fn abs(self) -> Money {
        Money(self.0.abs())
    }

    pub fn format(&self) -> String {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs_cents = self.0.abs();
        let dollars = abs_cents / 100;
        let cents = abs_cents % 100;
        format!("{}{}.{:02}", sign, dollars, cents)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl Default for Money {
    fn default() -> Self {
        Money::zero()
    }
}
