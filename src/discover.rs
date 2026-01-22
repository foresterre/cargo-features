use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub fn run(enabled_only: bool, manifest_path: Option<PathBuf>) -> Result<()> {
    let mut cmd = MetadataCommand::new();

    if let Some(path) = manifest_path {
        cmd.manifest_path(path);
    }

    let metadata = cmd.exec().context("Failed to get cargo metadata")?;
    let root_package = metadata.root_package().context("No root package found")?;

    println!(
        "{} {} ({})\n",
        "Analyzing".bright_blue().bold(),
        root_package.name.bright_white().bold(),
        root_package.version
    );

    let enabled_features = get_enabled_features(&metadata)?;

    println!("{}", "Root package features:\n".bright_green().bold());

    let mut root_features: Vec<_> = root_package.features.keys().collect();
    root_features.sort();

    for feature in root_features {
        let deps = &root_package.features[feature];
        let is_enabled = enabled_features
            .get(&root_package.id)
            .map(|f| f.contains(feature))
            .unwrap_or(false);

        if enabled_only && !is_enabled {
            continue;
        }

        let (marker, style) = if is_enabled {
            ("✓", feature.bright_green())
        } else {
            ("○", feature.dimmed())
        };

        println!(
            "  {} {} {}",
            marker,
            style,
            if deps.is_empty() {
                String::new()
            } else {
                format!("→ {}", deps.join(", ")).dimmed().to_string()
            }
        );
    }

    println!("\n{}", "Features by dependency:".bright_green().bold());

    let mut dep_features: Vec<_> = metadata
        .packages
        .iter()
        .filter(|p| p.id != root_package.id && !p.features.is_empty())
        .collect();

    dep_features.sort_by(|a, b| a.name.cmp(&b.name));

    for package in dep_features.as_slice() {
        let package_enabled = enabled_features
            .get(&package.id)
            .cloned()
            .unwrap_or_default();

        let mut features: Vec<_> = package.features.keys().collect();
        features.sort();

        let features_to_show: Vec<_> = if enabled_only {
            features
                .iter()
                .filter(|f| package_enabled.contains(f.as_str()))
                .copied()
                .collect()
        } else {
            features
        };

        if enabled_only && features_to_show.is_empty() {
            continue;
        }

        println!(
            "\n{} ({})",
            package.name.bright_cyan().bold(),
            package.version
        );

        let enabled_count = features_to_show
            .iter()
            .filter(|f| package_enabled.contains(f.as_str()))
            .count();

        if !enabled_only && enabled_count > 0 {
            println!(
                "  {} {}/{}",
                "Enabled:".bright_white(),
                enabled_count.to_string().bright_green(),
                features_to_show.len()
            );
        }

        for feature in features_to_show {
            let is_enabled = package_enabled.contains(feature);
            let (marker, style) = if is_enabled {
                ("✓", feature.bright_green())
            } else {
                ("○", feature.dimmed())
            };

            println!("  {} {}", marker, style);
        }
    }

    if dep_features.is_empty() {
        println!("  {}", "No dependency features found".dimmed());
    }

    let total_enabled: usize = enabled_features.values().map(|f| f.len()).sum();

    println!(
        "\n{} {} features enabled across {} crates",
        "Summary:".bright_blue().bold(),
        total_enabled.to_string().bright_green(),
        enabled_features.len()
    );

    Ok(())
}

fn get_enabled_features(
    metadata: &cargo_metadata::Metadata,
) -> Result<HashMap<cargo_metadata::PackageId, HashSet<String>>> {
    let mut enabled = HashMap::new();

    if let Some(resolve) = &metadata.resolve {
        for node in &resolve.nodes {
            let mut features = HashSet::new();

            for feature in &node.features {
                features.insert(feature.to_string());
            }

            enabled.insert(node.id.clone(), features);
        }
    }

    Ok(enabled)
}
