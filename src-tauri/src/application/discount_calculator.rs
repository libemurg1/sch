use crate::domain::entities::{Discount, StudentDiscount};
use crate::domain::money::Money;

/// A positive charge that discounts can be applied to.
#[derive(Debug, Clone)]
pub struct ChargeLine {
    /// Set when the charge originates from a fee item.
    pub fee_item_id: Option<String>,
    /// Set when the charge originates from an ad-hoc student fee.
    pub student_fee_id: Option<String>,
    pub description: String,
    pub amount_cents: i64,
    pub discountable: bool,
}

/// A discount resolved into a concrete negative line.
#[derive(Debug, Clone)]
pub struct DiscountLine {
    pub discount_id: String,
    pub name: String,
    /// Always zero or negative.
    pub amount_cents: i64,
}

/// Resolves the discounts granted to a student into negative invoice lines.
///
/// Rules (see README "Discount Rules"):
/// - `PERCENT` discounts use basis points (`1000` = 10.00%) and apply only to
///   the discountable part of the matching charges.
/// - Percentage discounts do not compound: each one is rounded half away from
///   zero on its own and the values are then added together.
/// - Percentages that share a target are capped at 100% of that target's base.
/// - `FIXED` discounts come off the whole invoice.
/// - Everything stacks; the only overall limit is that the invoice total never
///   drops below zero.
pub fn calculate(
    charges: &[ChargeLine],
    student_discounts: &[StudentDiscount],
    discount_definitions: &[Discount],
) -> Vec<DiscountLine> {
    let subtotal = charges
        .iter()
        .map(|charge| Money::from_cents(charge.amount_cents))
        .fold(Money::zero(), Money::add);
    if !subtotal.is_positive() {
        return Vec::new();
    }

    let mut percentage_lines: Vec<(String, DiscountLine)> = Vec::new();
    let mut fixed_lines: Vec<DiscountLine> = Vec::new();

    for granted in student_discounts {
        let Some(definition) = discount_definitions
            .iter()
            .find(|definition| definition.id == granted.discount_id)
        else {
            continue;
        };
        if !definition.is_active {
            continue;
        }

        match definition.kind.as_str() {
            "PERCENT" => {
                let base = discountable_base(charges, granted);
                if base.is_zero() {
                    continue;
                }
                let amount = base.mul_percent(definition.value).cents();
                if amount <= 0 {
                    continue;
                }
                percentage_lines.push((
                    target_key(granted),
                    DiscountLine {
                        discount_id: definition.id.clone(),
                        name: definition.name.clone(),
                        amount_cents: -amount,
                    },
                ));
            }
            "FIXED" => {
                if definition.value <= 0 {
                    continue;
                }
                fixed_lines.push(DiscountLine {
                    discount_id: definition.id.clone(),
                    name: definition.name.clone(),
                    amount_cents: -definition.value,
                });
            }
            _ => {}
        }
    }

    // Percentages sharing a target are capped at 100% of that target's base.
    let mut capped_percentage_lines: Vec<DiscountLine> = Vec::new();
    let mut scopes: Vec<String> = Vec::new();
    for (scope, _) in &percentage_lines {
        if !scopes.contains(scope) {
            scopes.push(scope.clone());
        }
    }
    for scope in scopes {
        let base = discountable_base_for_scope(charges, &scope);
        let mut group: Vec<DiscountLine> = percentage_lines
            .iter()
            .filter(|(group_scope, _)| group_scope == &scope)
            .map(|(_, line)| line.clone())
            .collect();

        let group_total = group
            .iter()
            .map(|line| Money::from_cents(-line.amount_cents))
            .fold(Money::zero(), Money::add);
        if group_total > base {
            let mut magnitudes: Vec<i64> = group.iter().map(|line| -line.amount_cents).collect();
            scale_to_fit(&mut magnitudes, base.cents());
            for (line, magnitude) in group.iter_mut().zip(magnitudes) {
                line.amount_cents = -magnitude;
            }
        }

        capped_percentage_lines.extend(group);
    }

    let mut lines = capped_percentage_lines;
    lines.extend(fixed_lines);

    // Final clamp: the total discount can never exceed the invoice subtotal.
    let total_discount = lines
        .iter()
        .map(|line| Money::from_cents(-line.amount_cents))
        .fold(Money::zero(), Money::add);
    let capped = total_discount.min(subtotal);
    if total_discount > capped {
        let mut magnitudes: Vec<i64> = lines.iter().map(|line| -line.amount_cents).collect();
        scale_to_fit(&mut magnitudes, capped.cents());
        for (line, magnitude) in lines.iter_mut().zip(magnitudes) {
            line.amount_cents = -magnitude;
        }
    }

    lines
}

/// The discountable base a percentage discount applies to.
fn discountable_base(charges: &[ChargeLine], granted: &StudentDiscount) -> Money {
    discountable_base_for_scope(charges, &target_key(granted))
}

/// The discountable base for a target scope key.
fn discountable_base_for_scope(charges: &[ChargeLine], scope: &str) -> Money {
    charges
        .iter()
        .filter(|charge| charge.discountable)
        .filter(|charge| scope_matches(charge, scope))
        .map(|charge| Money::from_cents(charge.amount_cents))
        .fold(Money::zero(), Money::add)
}

/// Identifies which charges a discount targets. A scope of `"*"` means the
/// whole invoice.
fn target_key(granted: &StudentDiscount) -> String {
    if let Some(fee_item_id) = &granted.target_fee_item_id {
        format!("fee_item:{}", fee_item_id)
    } else if let Some(student_fee_id) = &granted.target_student_fee_id {
        format!("student_fee:{}", student_fee_id)
    } else {
        "*".to_string()
    }
}

fn scope_matches(charge: &ChargeLine, scope: &str) -> bool {
    if scope == "*" {
        return true;
    }
    if let Some(fee_item_id) = scope.strip_prefix("fee_item:") {
        return charge.fee_item_id.as_deref() == Some(fee_item_id);
    }
    if let Some(student_fee_id) = scope.strip_prefix("student_fee:") {
        return charge.student_fee_id.as_deref() == Some(student_fee_id);
    }
    false
}

/// Scales `amounts` down so that they sum to exactly `target`.
///
/// The sum of the floors of the proportional shares can never exceed the
/// target, so the final element absorbs the remainder without going negative.
fn scale_to_fit(amounts: &mut [i64], target: i64) {
    let total: i64 = amounts.iter().sum();
    if total <= target || total == 0 || amounts.is_empty() {
        return;
    }

    let last = amounts.len() - 1;
    let mut allocated = 0i64;
    for amount in amounts.iter_mut().take(last) {
        let scaled = ((*amount as i128) * (target as i128) / (total as i128)) as i64;
        *amount = scaled;
        allocated += scaled;
    }
    amounts[last] = target - allocated;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn charge(id: &str, amount: i64, discountable: bool) -> ChargeLine {
        ChargeLine {
            fee_item_id: Some(id.to_string()),
            student_fee_id: None,
            description: id.to_string(),
            amount_cents: amount,
            discountable,
        }
    }

    fn discount(id: &str, kind: &str, value: i64) -> Discount {
        Discount {
            id: id.to_string(),
            name: id.to_string(),
            kind: kind.to_string(),
            value,
            is_active: true,
        }
    }

    fn grant(discount_id: &str, target_fee_item_id: Option<&str>) -> StudentDiscount {
        StudentDiscount {
            id: format!("sd-{}", discount_id),
            student_id: "s1".to_string(),
            discount_id: discount_id.to_string(),
            term_id: None,
            target_fee_item_id: target_fee_item_id.map(|id| id.to_string()),
            target_student_fee_id: None,
            reason: None,
        }
    }

    #[test]
    fn percentage_applies_to_discountable_base_only() {
        let charges = vec![charge("tuition", 10_000, true), charge("bus", 5_000, false)];
        let lines = calculate(
            &charges,
            &[grant("d1", None)],
            &[discount("d1", "PERCENT", 1000)],
        );
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].amount_cents, -1_000);
    }

    #[test]
    fn percentages_do_not_compound() {
        let charges = vec![charge("tuition", 10_000, true)];
        let lines = calculate(
            &charges,
            &[grant("d1", None), grant("d2", None)],
            &[discount("d1", "PERCENT", 1000), discount("d2", "PERCENT", 1000)],
        );
        let total: i64 = lines.iter().map(|line| line.amount_cents).sum();
        assert_eq!(total, -2_000);
    }

    #[test]
    fn percentages_are_capped_at_one_hundred_percent_of_base() {
        let charges = vec![charge("tuition", 10_000, true)];
        let lines = calculate(
            &charges,
            &[grant("d1", None), grant("d2", None)],
            &[discount("d1", "PERCENT", 8000), discount("d2", "PERCENT", 8000)],
        );
        let total: i64 = lines.iter().map(|line| line.amount_cents).sum();
        assert_eq!(total, -10_000);
    }

    #[test]
    fn fixed_discount_comes_off_the_whole_invoice() {
        let charges = vec![charge("tuition", 10_000, true), charge("bus", 5_000, false)];
        let lines = calculate(
            &charges,
            &[grant("d1", None)],
            &[discount("d1", "FIXED", 2_500)],
        );
        assert_eq!(lines[0].amount_cents, -2_500);
    }

    #[test]
    fn total_discount_never_exceeds_the_subtotal() {
        let charges = vec![charge("tuition", 1_000, true)];
        let lines = calculate(
            &charges,
            &[grant("d1", None), grant("d2", None)],
            &[discount("d1", "FIXED", 5_000), discount("d2", "FIXED", 5_000)],
        );
        let total: i64 = lines.iter().map(|line| -line.amount_cents).sum();
        assert_eq!(total, 1_000);
    }

    #[test]
    fn targeted_percentage_only_touches_its_own_charge() {
        let charges = vec![charge("tuition", 10_000, true), charge("bus", 5_000, true)];
        let lines = calculate(
            &charges,
            &[grant("d1", Some("bus"))],
            &[discount("d1", "PERCENT", 1000)],
        );
        assert_eq!(lines[0].amount_cents, -500);
    }
}
