use uuid::Uuid;

pub fn create_uuid() -> String {
    Uuid::new_v4().to_string()
}

// CATEGORY KEYS
pub fn categories_version_key(user_id: Uuid) -> String {
    format!("user:{}:categories:version", user_id)
}

// EXPENSE KEYS
pub fn all_expenses_version_key(user_id: Uuid) -> String {
    format!("user:{}:expenses:version", user_id)
}

pub fn category_filter_expenses_version_key(category_id: Uuid, user_id: Uuid) -> String {
    format!(
        "user:{}:filter:category:{}:expenses:version",
        user_id, category_id
    )
}

pub fn category_filter_total_expense_key(category_id: Uuid, user_id: Uuid) -> String {
    format!(
        "user:{}:filter:category:{}:total:expenses",
        user_id, category_id
    )
}

pub fn single_expense_key(expense_id: Uuid, user_id: Uuid) -> String {
    format!("user:{}:expense:{}", user_id, expense_id)
}

pub fn total_expense_key(user_id: Uuid) -> String {
    format!("user:{}:total:expenses", user_id)
}
