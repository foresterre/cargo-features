use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

pub fn run(
    skip_empty: bool,
    feature_filter: Vec<String>,
    manifest_path: Option<PathBuf>,
    cargo_command: Vec<String>,
) -> Result<()> {
    let mut cmd = MetadataCommand::new();

    if let Some(path) = &manifest_path {
        cmd.manifest_path(path);
    }

    let metadata = cmd.exec().context("Failed to get cargo metadata")?;
    let root_package = metadata.root_package().context("No root package found")?;

    let features: Vec<String> = if feature_filter.is_empty() {
        root_package.features.keys().cloned().collect()
    } else {
        feature_filter
    };

    println!(
        "{} powerset for {} features: {}\n",
        "Generating".bright_blue().bold(),
        features.len(),
        features.join(", ").bright_yellow()
    );

    let powerset = powerset(&features);

    let total = if skip_empty {
        powerset.len() - 1
    } else {
        powerset.len()
    };

    println!(
        "{} {} feature combinations\n",
        "Testing".bright_green().bold(),
        total
    );

    let mut passed = 0;
    let mut failed = 0;

    for (idx, feature_set) in powerset.iter().enumerate() {
        if skip_empty && feature_set.is_empty() {
            continue;
        }

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

fn powerset(features: &[String]) -> Vec<Vec<String>> {
    let n = features.len();
    let mut result = Vec::new();

    // Iterate through all possible combinations (2^n)
    for i in 0..(1 << n) {
        let mut subset = Vec::new();

        for (j, feature) in features.iter().enumerate() {
            if i & (1 << j) != 0 {
                subset.push(feature.clone());
            }
        }

        result.push(subset);
    }

    result
}

fn run_cargo_command(
    cargo_command: &[String],
    features: &[String],
    manifest_path: Option<&PathBuf>,
) -> Result<bool> {
    let mut cmd = Command::new("cargo");

    for arg in cargo_command {
        cmd.arg(arg);
    }

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
