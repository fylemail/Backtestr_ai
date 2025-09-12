pub mod handlers;
pub mod protocol;
pub mod streaming;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_initialization() {
        assert_eq!(2 + 2, 4);
    }
}
