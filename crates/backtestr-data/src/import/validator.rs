use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid value for field {field}: {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("Invalid timestamp format: {0}")]
    InvalidTimestamp(String),

    #[error("Negative price: bid={bid}, ask={ask}")]
    NegativePrice { bid: f64, ask: f64 },

    #[error("Invalid spread: bid ({bid}) > ask ({ask})")]
    InvalidSpread { bid: f64, ask: f64 },
}

pub fn validate_tick_data(
    symbol: Option<&str>,
    timestamp: Option<&str>,
    bid: Option<f64>,
    ask: Option<f64>,
) -> Result<(), ValidationError> {
    // Validate required fields
    if symbol.is_none() || symbol == Some("") {
        return Err(ValidationError::MissingField("symbol".to_string()));
    }

    if timestamp.is_none() || timestamp == Some("") {
        return Err(ValidationError::MissingField("timestamp".to_string()));
    }

    let bid_val = bid.ok_or_else(|| ValidationError::MissingField("bid".to_string()))?;
    let ask_val = ask.ok_or_else(|| ValidationError::MissingField("ask".to_string()))?;

    // Validate positive prices
    if bid_val <= 0.0 || ask_val <= 0.0 {
        return Err(ValidationError::NegativePrice {
            bid: bid_val,
            ask: ask_val,
        });
    }

    // Validate spread (bid should be less than ask in normal conditions)
    // Note: We allow bid >= ask for crossed markets, but flag extreme cases
    if bid_val > ask_val * 1.1 {
        // More than 10% crossed is likely an error
        return Err(ValidationError::InvalidSpread {
            bid: bid_val,
            ask: ask_val,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tick_data() {
        let result = validate_tick_data(
            Some("EURUSD"),
            Some("2024-01-01T00:00:00Z"),
            Some(1.0921),
            Some(1.0923),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_symbol() {
        let result = validate_tick_data(
            None,
            Some("2024-01-01T00:00:00Z"),
            Some(1.0921),
            Some(1.0923),
        );
        assert!(matches!(result, Err(ValidationError::MissingField(_))));
    }

    #[test]
    fn test_negative_prices() {
        let result = validate_tick_data(
            Some("EURUSD"),
            Some("2024-01-01T00:00:00Z"),
            Some(-1.0921),
            Some(1.0923),
        );
        assert!(matches!(result, Err(ValidationError::NegativePrice { .. })));
    }

    #[test]
    fn test_invalid_spread() {
        let result = validate_tick_data(
            Some("EURUSD"),
            Some("2024-01-01T00:00:00Z"),
            Some(2.0),
            Some(1.0),
        );
        assert!(matches!(result, Err(ValidationError::InvalidSpread { .. })));
    }
}
