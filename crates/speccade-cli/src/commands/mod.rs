//! CLI command implementations

pub mod doctor;
pub mod expand;
pub mod fmt;
pub mod generate;
pub mod generate_all;
pub mod migrate;
pub mod preview;
pub mod template;
pub mod validate;

mod reporting;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_module_exports_entrypoints() {
        let _ = doctor::run;
        let _ = expand::run;
        let _ = fmt::run;
        let _ = generate::run;
        let _ = generate_all::run;
        let _ = preview::run;
        let _ = template::list;
        let _ = validate::run;
    }
}
