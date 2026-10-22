use crate::domain::dto::SaveFeeStructureInput;
use crate::domain::entities::{FeeItem, FeeStructure};
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::FeeStructureRepository;
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::fee_structure_repository::SqliteFeeStructureRepository;
use crate::infrastructure::sqlite::unit_of_work::UnitOfWork;
use uuid::Uuid;

/// Use cases for the fee structures attached to a class and term.
pub struct FeeStructureService;

impl FeeStructureService {
    pub fn get(db: &Db, class_id: &str, term_id: &str) -> AppResult<Option<(FeeStructure, Vec<FeeItem>)>> {
        db.with_conn(|conn| {
            let repository = SqliteFeeStructureRepository::new(conn);
            let Some(structure) = repository.find_by_class_term(class_id, term_id)? else {
                return Ok(None);
            };
            let items = repository.list_items(&structure.id)?;
            Ok(Some((structure, items)))
        })
    }

    /// Creates or replaces the fee structure for a class and term, together
    /// with its items. The structure and its items are always written
    /// together.
    pub fn save(db: &Db, input: SaveFeeStructureInput) -> AppResult<FeeStructure> {
        if input.class_id.trim().is_empty() || input.term_id.trim().is_empty() {
            return Err(AppError::validation("Class and term are both required"));
        }
        if input.items.is_empty() {
            return Err(AppError::validation(
                "A fee structure needs at least one fee item",
            ));
        }

        let mut total_cents: i64 = 0;
        let mut items = Vec::with_capacity(input.items.len());
        for (index, item) in input.items.iter().enumerate() {
            if item.description.trim().is_empty() {
                return Err(AppError::validation("Fee item description is required"));
            }
            if item.amount_cents <= 0 {
                return Err(AppError::validation(format!(
                    "Fee item '{}' must have a positive amount",
                    item.description
                )));
            }
            total_cents += item.amount_cents;
            items.push(FeeItem {
                id: Uuid::new_v4().to_string(),
                fee_structure_id: String::new(),
                description: item.description.trim().to_string(),
                amount_cents: item.amount_cents,
                discountable: item.discountable,
                sort_order: if item.sort_order == 0 {
                    index as i32
                } else {
                    item.sort_order
                },
            });
        }

        let structure = FeeStructure {
            id: Uuid::new_v4().to_string(),
            class_id: input.class_id.trim().to_string(),
            term_id: input.term_id.trim().to_string(),
            total_cents,
        };

        db.execute(|conn| {
            let repository = SqliteFeeStructureRepository::new(conn);
            repository.save(&structure, &items)?;

            // The repository keeps the original id when replacing an existing
            // structure, so read it back to report the persisted value.
            match repository.find_by_class_term(&structure.class_id, &structure.term_id)? {
                Some(saved) => Ok(saved),
                None => Ok(structure),
            }
        })
    }
}
