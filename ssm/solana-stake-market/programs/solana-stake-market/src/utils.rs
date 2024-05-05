pub fn validate_rate(rate: u64) -> bool {
    rate >= 9000 && rate <= 10000 // Rates must be between 0.9 and 1.0
}
