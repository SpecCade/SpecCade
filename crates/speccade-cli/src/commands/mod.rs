//! CLI command implementations

pub mod analyze;
mod analyze_csv;
pub mod audit;
pub mod cache;
pub mod compare;
pub mod doctor;
pub mod eval;
pub mod expand;
pub mod fmt;
pub mod generate;
pub mod generate_all;
pub mod inspect;
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
        let _ = audit::run;
        let _ = compare::run;
        let _ = doctor::run;
        let _ = eval::run;
        let _ = expand::run;
        let _ = fmt::run;
        let _ = generate::run;
        let _ = generate_all::run;
        let _ = inspect::run;
        let _ = preview::run;
        let _ = template::list;
        let _ = validate::run;
    }
}
