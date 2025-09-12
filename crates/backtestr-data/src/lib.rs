pub mod migration;
pub mod query;
pub mod storage;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_initialization() {
        assert_eq!(2 + 2, 4);
    }
}
