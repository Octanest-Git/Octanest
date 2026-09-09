//! Shared domain types for Octanest.

pub fn crate_name() -> &'static str {
    "octanest-core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name() {
        assert_eq!(crate_name(), "octanest-core");
    }
}
