use thiserror::Error;

#[derive(Debug, Error)]
pub enum BillingError {
    #[error("invoice {id} has a non-positive amount: {amount}")]
    InvalidAmount { id: u32, amount: f64 },

    #[error("invoice {id} has an empty customer region")]
    MissingRegion { id: u32 },

    #[error("invoice {id} has an unknown customer region: {region}")]
    UnknownRegion { id: u32, region: String },
}
