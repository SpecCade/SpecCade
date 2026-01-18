//! CLI command implementations

pub mod analyze;
pub mod cache;
pub mod doctor;
pub mod eval;
pub mod expand;
pub mod fmt;
pub mod generate;
pub mod generate_all;
pub mod json_output;
pub mod migrate;
pub mod preview;
pub mod stdlib;
pub mod template;
pub mod validate;

mod reporting;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_module_exports_entrypoints() {
        let _ = analyze::run;
        let _ = doctor::run;
        let _ = eval::run;
        let _ = expand::run;
        let _ = fmt::run;
        let _ = generate::run;
        let _ = generate_all::run;
        let _ = preview::run;
        let _ = template::list;
        let _ = validate::run;
    }
}
