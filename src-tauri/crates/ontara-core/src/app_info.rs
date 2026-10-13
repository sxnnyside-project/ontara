/// Identity of the running product, reported to every adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}

/// Returns the product name and the version this crate was built with.
#[must_use]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "Ontara",
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_product_name_and_package_version() {
        let info = app_info();

        assert_eq!(info.name, "Ontara");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    }
}
