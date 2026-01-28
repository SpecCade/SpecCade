//! CLI command implementations

pub mod analyze;
mod analyze_csv;
pub mod audit;
pub mod cache;
pub mod compare;
pub mod coverage;
pub mod doctor;
pub mod eval;
pub mod expand;
pub mod fmt;
pub mod generate; // Directory module: human, json, quality, variations, tests
pub mod generate_all;
pub mod inspect;
pub mod json_output; // Directory module: analysis, convert, manifest, records
pub mod lint;
pub mod migrate;
pub mod preview;
pub mod preview_grid;
#[cfg(feature = "serve")]
pub mod serve;
pub mod stdlib;
pub mod template;
pub mod validate;
pub mod verify;

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
        let _ = lint::run;
        let _ = preview::run;
        let _ = preview_grid::run;
        let _ = template::list;
        let _ = validate::run;
        let _ = verify::run;
        let _ = coverage::run_generate;
        let _ = coverage::run_report;
    }

    #[cfg(feature = "serve")]
    #[test]
    fn serve_module_exports_entrypoints() {
        let _ = serve::run;
        let _ = serve::analyze_path;
        let _ = serve::analyze_data;
    }
}
