use crate::domain::entities::*;
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::*;
use uuid::Uuid;

pub struct BillingService {
    enrollments: std::sync::Arc<dyn EnrollmentRepository>,
    fee_structures: std::sync::Arc<dyn FeeStructureRepository>,
    invoices: std::sync::Arc<dyn InvoiceRepository>,
    numbers: std::sync::Arc<crate::infrastructure::sqlite::number_generator::NumberGenerator>,
    student_fees: std::sync::Arc<dyn StudentFeeRepository>,
    discounts: std::sync::Arc<dyn DiscountRepository>,
    uow: std::sync::Arc<crate::infrastructure::sqlite::unit_of_work::SqliteUnitOfWork>,
}

impl BillingService {
    pub fn new(
        enrollments: std::sync::Arc<dyn EnrollmentRepository>,
        fee_structures: std::sync::Arc<dyn FeeStructureRepository>,
        invoices: std::sync::Arc<dyn InvoiceRepository>,
        numbers: std::sync::Arc<crate::infrastructure::sqlite::number_generator::NumberGenerator>,
        student_fees: std::sync::Arc<dyn StudentFeeRepository>,
        discounts: std::sync::Arc<dyn DiscountRepository>,
        uow: std::sync::Arc<crate::infrastructure::sqlite::unit_of_work::SqliteUnitOfWork>,
    ) -> Self {
        Self { enrollments, fee_structures, invoices, numbers, student_fees, discounts, uow }
    }
    
    pub fn generate_batch(&self, class_id: String, term_id: String, issue_date: String, due_date: String) -> AppResult<Vec<Invoice>> {
        let structure = self.fee_structures.find_by_class_term(&class_id, &term_id)?
            .ok_or_else(|| AppError { code: "NO_FEE_STRUCTURE".to_string(), message: "No fee structure for this class and term".to_string() })?;
        
        let enrollments = self.enrollments.list_by_class_term(&class_id, &term_id)?;
        let mut created = Vec::new();
        let year: i32 = chrono::Utc::now().format("%Y").to_string().parse().unwrap_or(2026);
        
        for enr in enrollments {
            if enr.status != "ACTIVE" { continue; }
            if self.invoices.find_live_by_student_term(&enr.student_id, &term_id)?.is_some() { continue; }
            
            let inv_num = self.numbers.next_formatted("INV", year, 3)?;
            let invoice = Invoice {
                id: Uuid::new_v4().to_string(),
                invoice_number: inv_num,
                kind: "TERM".to_string(),
                student_id: enr.student_id.clone(),
                term_id: term_id.clone(),
                fee_structure_id: Some(structure.id.clone()),
                issue_date: issue_date.clone(),
                due_date: due_date.clone(),
                status: "UNPAID".to_string(),
                subtotal_cents: 0,
                discount_cents: 0,
                total_cents: 0,
                paid_cents: 0,
            };
            // TODO: Calculate with fees, discounts, create items
            self.invoices.create(&invoice)?;
            created.push(invoice);
        }
        Ok(created)
    }
}
