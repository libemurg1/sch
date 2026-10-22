use serde::{Deserialize, Serialize};
use std::fmt;

/// Money is always stored as an integer number of cents. No floating point is
/// used for amounts, totals or balances.
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

    /// Applies a basis-point percentage (`1000` = 10.00%) and rounds half away
    /// from zero. The intermediate product is computed in 128 bits so large
    /// amounts cannot overflow.
    pub fn mul_percent(self, basis_points: i64) -> Money {
        let product = (self.0 as i128) * (basis_points as i128);
        let magnitude = product.abs() + 5000;
        let rounded = magnitude / 10000;
        let signed = if product < 0 { -rounded } else { rounded };
        Money(signed as i64)
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
        let sign = if self.is_negative() { "-" } else { "" };
        let abs_cents = self.abs().cents();
        format!("{}{}.{:02}", sign, abs_cents / 100, abs_cents % 100)
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

#[cfg(test)]
mod tests {
    use super::Money;

    #[test]
    fn rounds_half_away_from_zero() {
        assert_eq!(Money::from_cents(1005).mul_percent(5000).cents(), 503);
        assert_eq!(Money::from_cents(1050).mul_percent(5000).cents(), 525);
        assert_eq!(Money::from_cents(-1005).mul_percent(5000).cents(), -503);
    }

    #[test]
    fn applies_basis_points() {
        assert_eq!(Money::from_cents(10_000).mul_percent(1000).cents(), 1_000);
        assert_eq!(Money::from_cents(10_000).mul_percent(10_000).cents(), 10_000);
        assert_eq!(Money::from_cents(10_000).mul_percent(0).cents(), 0);
    }

    #[test]
    fn formats_as_decimal() {
        assert_eq!(Money::from_cents(123456).format(), "1234.56");
        assert_eq!(Money::from_cents(5).format(), "0.05");
        assert_eq!(Money::from_cents(-500).format(), "-5.00");
    }
}
