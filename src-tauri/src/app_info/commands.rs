use serde::Serialize;

/// Frontend contract of `get_app_info`; mirrored by `src/features/app-info/api/get-app-info.ts`.
#[derive(Debug, Serialize)]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
}

#[tauri::command]
#[must_use]
pub fn get_app_info() -> AppInfoDto {
    let info = ontara_core::app_info();

    AppInfoDto {
        name: info.name.to_owned(),
        version: info.version.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_the_domain_info_to_the_frontend_contract() {
        let dto = get_app_info();

        assert_eq!(dto.name, "Ontara");
        assert_eq!(dto.version, env!("CARGO_PKG_VERSION"));
    }
}
