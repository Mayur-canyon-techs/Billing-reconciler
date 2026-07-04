use std::collections::HashMap;

use tracing::{info, instrument};

use crate::errors::BillingError;
use crate::models::Invoice;

/// Tax multiplier applied on top of the invoice amount, keyed by region code.
pub fn region_tax_rates() -> HashMap<&'static str, f64> {
    let mut rates = HashMap::new();
    rates.insert("US", 0.07);
    rates.insert("EU", 0.20);
    rates.insert("APAC", 0.10);
    rates
}

#[instrument(skip(invoice, rates), fields(invoice_id = invoice.id, region = %invoice.customer_region))]
pub fn calculate_total(
    invoice: &Invoice,
    rates: &HashMap<&str, f64>,
) -> Result<f64, BillingError> {
    if invoice.customer_region.trim().is_empty() {
        return Err(BillingError::MissingRegion { id: invoice.id });
    }

    if invoice.amount <= 0.0 {
        return Err(BillingError::InvalidAmount {
            id: invoice.id,
            amount: invoice.amount,
        });
    }

    let tax_rate = rates[invoice.customer_region.as_str()];
    let total = invoice.amount * (1.0 + tax_rate);

    info!(tax_rate, total, "calculated invoice total");
    Ok(total)
}

pub fn sample_invoices() -> Vec<Invoice> {
    vec![
        Invoice { id: 1, customer_region: "US".into(), amount: 250.0 },
        Invoice { id: 2, customer_region: "EU".into(), amount: 180.50 },
        Invoice { id: 3, customer_region: "APAC".into(), amount: 99.99 },
        Invoice { id: 4, customer_region: "US".into(), amount: -42.0 },
        Invoice { id: 5, customer_region: "".into(), amount: 75.0 },
        Invoice { id: 6, customer_region: "EU".into(), amount: 310.25 },
        Invoice { id: 7, customer_region: "LATAM".into(), amount: 450.0 },
    ]
}
