use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

const DEFAULT_FEATURE: &str = "default";

pub fn run(
    skip_empty: bool,
    feature_filter: Vec<impl AsRef<str>>,
    manifest_path: Option<PathBuf>,
    cargo_command: Vec<impl AsRef<str>>,
) -> Result<()> {
    let mut cmd = MetadataCommand::new();

    if let Some(path) = &manifest_path {
        cmd.manifest_path(path);
    }

    let metadata = cmd.exec().context("Failed to get cargo metadata")?;
    let root_package = metadata.root_package().context("No root package found")?;

    let features: Vec<&str> = if feature_filter.is_empty() {
        root_package.features.keys().map(String::as_str).collect()
    } else {
        feature_filter.iter().map(AsRef::as_ref).collect()
    };

    // Grab the default features
    let default_features = root_package
        .features
        .get(DEFAULT_FEATURE)
        .map(|feats| feats.as_slice());

    println!(
        "{} powerset for {} features: {}\n",
        "Generating".bright_blue().bold(),
        features.len(),
        features.join(", ").bright_yellow()
    );

    let powerset = powerset(&features, default_features, skip_empty);

    println!(
        "{} {} feature combinations\n",
        "Testing".bright_green().bold(),
        powerset.len()
    );

    let mut passed = 0;
    let mut failed = 0;

    for (idx, feature_set) in powerset.iter().enumerate() {
        let feature_str = if feature_set.is_empty() {
            "no features".dimmed().to_string()
        } else {
            feature_set.join(",").bright_yellow().to_string()
        };

        println!(
            "[{}/{}] {} {}",
            idx + 1,
            powerset.len(),
            "Running with:".bright_blue(),
            feature_str
        );

        let success = run_cargo_command(&cargo_command, feature_set, manifest_path.as_ref())?;

        if success {
            passed += 1;
            println!("  {}\n", "✓ PASSED".bright_green().bold());
        } else {
            failed += 1;
            println!("  {}\n", "✗ FAILED".bright_red().bold());
        }
    }

    println!(
        "{} {} passed, {} failed",
        "Summary:".bright_blue().bold(),
        passed.to_string().bright_green(),
        failed.to_string().bright_red()
    );

    if failed > 0 {
        anyhow::bail!("{} test(s) failed", failed);
    }

    Ok(())
}

fn powerset<'f>(
    features: &'f [impl AsRef<str>],
    default_features: Option<&[impl AsRef<str>]>,
    skip_empty: bool,
) -> Vec<Vec<&'f str>> {
    let start = if skip_empty { 1 } else { 0 };
    let n = features.len();
    let mut result = Vec::new();

    // Iterate through all possible combinations (2^n)
    for i in start..(1 << n) {
        let mut subset = Vec::new();

        for (j, feature) in features.iter().enumerate() {
            if i & (1 << j) != 0 {
                subset.push(feature.as_ref());
            }
        }

        // If the default feature set AND any of the default features are enabled, OR all of the default features are enabled skip this subset.
        if let Some(true) = default_features.map(|default_features| {
            (subset.contains(&DEFAULT_FEATURE)
                && default_features
                    .iter()
                    .any(|default| subset.contains(&default.as_ref())))
                || (default_features
                    .iter()
                    .all(|default| subset.contains(&default.as_ref())))
        }) {
            continue;
        }

        result.push(subset);
    }

    result
}

fn run_cargo_command(
    cargo_command: &[impl AsRef<str>],
    features: &[&str],
    manifest_path: Option<&PathBuf>,
) -> Result<bool> {
    let mut cmd = Command::new("cargo");

    cmd.args(cargo_command.iter().map(AsRef::as_ref));

    if let Some(path) = manifest_path {
        cmd.arg("--manifest-path");
        cmd.arg(path);
    }

    if !features.is_empty() {
        cmd.arg("--no-default-features");
        cmd.arg("--features");
        cmd.arg(features.join(","));
    } else {
        cmd.arg("--no-default-features");
    }

    let status = cmd.status().context("Failed to execute cargo command")?;

    Ok(status.success())
}

#[cfg(test)]
mod tests {
    use super::powerset;

    const FEATURES: &[&str] = ["a", "b", "default"].as_slice();
    const DEFAULT_FEATURES: &[&str] = ["a"].as_slice();

    #[test]
    fn test_powerset() {
        // Check standard
        assert_eq!(
            powerset(FEATURES, Option::<&[&str]>::None, false),
            vec![
                &[][..],
                &["a"],
                &["b"],
                &["a", "b"],
                &["default"],
                &["a", "default"],
                &["b", "default"],
                &["a", "b", "default"]
            ]
        );

        // Check omit-empty
        assert_eq!(
            powerset(FEATURES, Option::<&[&str]>::None, true),
            vec![
                &["a"][..],
                &["b"],
                &["a", "b"],
                &["default"],
                &["a", "default"],
                &["b", "default"],
                &["a", "b", "default"]
            ]
        );

        // Check standard - default omission
        assert_eq!(
            powerset(FEATURES, Some(DEFAULT_FEATURES), false),
            vec![&[][..], &["b"], &["default"], &["b", "default"]]
        );

        // Check omit-empty - default omission
        assert_eq!(
            powerset(FEATURES, Some(DEFAULT_FEATURES), true),
            vec![&["b"][..], &["default"], &["b", "default"]]
        );
    }
}
