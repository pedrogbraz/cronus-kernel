//! `env { … }` schema, checked when `cronus run` starts (LANGUAGE.md §3.8).
//!
//! ```cronus
//! env {
//!   APP_STRIPE_KEY string! sensitive
//!   APP_FEATURE_X  boolean default:false
//! }
//! ```
//!
//! A required variable with no value and no `default:`, or a value of the
//! wrong type, stops the server before it opens the database. The report
//! names variables and expected types only: values, sensitive or not, are
//! never printed.

use crate::parser::{AstNode, EnvType, EnvVarSpec};

#[derive(Debug, Clone, PartialEq)]
pub enum EnvProblem {
    Missing(String),
    WrongType { name: String, expected: EnvType },
}

/// Every declared variable across all `env` blocks, in source order.
pub fn declared(nodes: &[AstNode]) -> Vec<&EnvVarSpec> {
    nodes
        .iter()
        .flat_map(|n| match n {
            AstNode::Env(env) => env.schema.iter().collect::<Vec<_>>(),
            _ => Vec::new(),
        })
        .collect()
}

/// Checks `specs` against `lookup` (the process environment in `cronus run`).
/// An empty value counts as unset.
pub fn check(specs: &[&EnvVarSpec], lookup: impl Fn(&str) -> Option<String>) -> Vec<EnvProblem> {
    specs
        .iter()
        .filter_map(
            |spec| match lookup(&spec.name).filter(|v| !v.trim().is_empty()) {
                None if spec.required && spec.default.is_none() => {
                    Some(EnvProblem::Missing(spec.name.clone()))
                }
                Some(value) if !spec.env_type.accepts(&value) => Some(EnvProblem::WrongType {
                    name: spec.name.clone(),
                    expected: spec.env_type,
                }),
                _ => None,
            },
        )
        .collect()
}

fn describe(ty: EnvType) -> &'static str {
    match ty {
        EnvType::String => "a string",
        EnvType::Number => "a number",
        EnvType::Boolean => "a boolean (true, false, 1 or 0)",
        EnvType::Url => "an http(s) URL",
        EnvType::Email => "an email address",
    }
}

/// One line for all missing variables, one per mistyped variable.
pub fn report(problems: &[EnvProblem]) -> String {
    let missing: Vec<&str> = problems
        .iter()
        .filter_map(|p| match p {
            EnvProblem::Missing(name) => Some(name.as_str()),
            _ => None,
        })
        .collect();
    let mut lines = Vec::new();
    if !missing.is_empty() {
        lines.push(format!(
            "missing required environment variables: {}",
            missing.join(", ")
        ));
    }
    for problem in problems {
        if let EnvProblem::WrongType { name, expected } = problem {
            lines.push(format!(
                "environment variable {} must be {} (value not shown)",
                name,
                describe(*expected)
            ));
        }
    }
    lines.join("\n")
}

/// Startup gate for `cronus run`: `Ok(declared count)` or the report.
pub fn enforce(
    nodes: &[AstNode],
    lookup: impl Fn(&str) -> Option<String>,
) -> Result<usize, String> {
    let specs = declared(nodes);
    let problems = check(&specs, lookup);
    if problems.is_empty() {
        Ok(specs.len())
    } else {
        Err(report(&problems))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use std::collections::HashMap;

    const SRC: &str = "app \"E\" { port 5175 }\nenv {\n  APP_STRIPE_KEY string! sensitive  APP_FEATURE_X boolean default:false\n  APP_PORT number\n  APP_HOOK url!\n  APP_ADMIN email default:admin@example.com\n}\n";

    fn run(values: &[(&str, &str)]) -> Result<usize, String> {
        let nodes = parse(SRC).expect("parse");
        let env: HashMap<String, String> = values
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        enforce(&nodes, |name| env.get(name).cloned())
    }

    #[test]
    fn missing_required_variables_are_listed_together() {
        let err = run(&[]).unwrap_err();
        assert_eq!(
            err,
            "missing required environment variables: APP_STRIPE_KEY, APP_HOOK"
        );
        let err = run(&[("APP_STRIPE_KEY", "  "), ("APP_HOOK", "https://h.example")]).unwrap_err();
        assert_eq!(
            err,
            "missing required environment variables: APP_STRIPE_KEY"
        );
    }

    #[test]
    fn wrong_types_are_errors_and_values_are_never_printed() {
        let err = run(&[
            ("APP_STRIPE_KEY", "sk_live_SECRET"),
            ("APP_HOOK", "not-a-url-SECRET"),
            ("APP_FEATURE_X", "maybe-SECRET"),
            ("APP_PORT", "eighty-SECRET"),
        ])
        .unwrap_err();
        assert!(!err.contains("SECRET"), "{err}");
        assert_eq!(
            err.lines().collect::<Vec<_>>(),
            [
                "environment variable APP_FEATURE_X must be a boolean (true, false, 1 or 0) (value not shown)",
                "environment variable APP_PORT must be a number (value not shown)",
                "environment variable APP_HOOK must be an http(s) URL (value not shown)",
            ]
        );
    }

    #[test]
    fn valid_environment_passes_and_defaults_cover_optional_variables() {
        assert_eq!(
            run(&[
                ("APP_STRIPE_KEY", "sk_test_1"),
                ("APP_HOOK", "http://hooks.internal"),
                ("APP_PORT", "8080"),
            ]),
            Ok(5)
        );
        let none = parse("app \"E\" { port 5175 }\n").unwrap();
        assert_eq!(enforce(&none, |_| None), Ok(0));
    }
}
