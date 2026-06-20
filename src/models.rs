#[derive(Debug, Clone)]
pub struct Invoice {
    pub id: u32,
    pub customer_region: String,
    pub amount: f64,
}
