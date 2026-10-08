use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{ensure, Context, Result};
use regex::Regex;
use roxmltree::Document;
use serde_yaml_ng::{Mapping, Value};

use crate::build::load;

const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
const SVG_ELEMENTS: &[&str] = &[
    "svg", "g", "path", "text", "title", "desc", "defs", "pattern",
];
const SVG_ATTRIBUTES: &[&str] = &[
    "width",
    "height",
    "viewBox",
    "role",
    "aria-label",
    "aria-labelledby",
    "id",
    "x",
    "y",
    "d",
    "fill",
    "fill-opacity",
    "stroke",
    "stroke-opacity",
    "font-family",
    "font-size",
    "textLength",
    "lengthAdjust",
    "text-anchor",
    "patternUnits",
];
static PAINT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i-u)^(?:#[a-f0-9]{3,8}|none|currentColor|url\(#[a-z][a-z0-9_-]*\))$")
        .expect("constant paint pattern")
});
static ACTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z0-9_-]+/[A-Za-z0-9_-]+(?:/[A-Za-z0-9_-]+)*@[a-f0-9]{40}$")
        .expect("constant action pattern")
});
static EVENT_DATA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\{\{[^}]*github\.event\.").expect("constant event pattern"));

pub fn validate_svg(source: &str) -> Result<()> {
    ensure!(
        !source.trim_start().starts_with("<?xml"),
        "XML declarations are forbidden"
    );
    let document = Document::parse(source).context("Invalid or unsafe SVG XML")?;
    let root = document.root_element();
    ensure!(
        root.tag_name().name() == "svg" && root.tag_name().namespace() == Some(SVG_NAMESPACE),
        "SVG root required"
    );
    ensure!(
        root.attribute("role") == Some("img"),
        "Accessible image role required"
    );
    for dimension in ["width", "height"] {
        let value = root
            .attribute(dimension)
            .context("SVG dimensions required")?
            .parse::<f64>()?;
        ensure!(
            value.is_finite() && value > 0.0,
            "Positive finite dimensions required"
        );
    }
    let view_box = root
        .attribute("viewBox")
        .context("viewBox required")?
        .split_whitespace()
        .map(str::parse::<f64>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        view_box.len() == 4
            && view_box.iter().all(|value| value.is_finite())
            && view_box[2] > 0.0
            && view_box[3] > 0.0,
        "Valid viewBox required"
    );
    let mut ids = BTreeMap::new();
    let mut references = Vec::new();
    for node in document.descendants() {
        ensure!(!node.is_pi(), "Processing instructions are forbidden");
        if !node.is_element() {
            continue;
        }
        ensure!(
            node.tag_name().namespace() == Some(SVG_NAMESPACE)
                && SVG_ELEMENTS.contains(&node.tag_name().name()),
            "Unsafe SVG element: {}",
            node.tag_name().name()
        );
        ensure!(
            node.namespaces()
                .all(|namespace| namespace.name().is_none() && namespace.uri() == SVG_NAMESPACE),
            "Additional XML namespaces are forbidden"
        );
        if let Some(id) = node.attribute("id") {
            ensure!(ids.insert(id, node).is_none(), "Duplicate SVG ID: {id}");
        }
        for attribute in node.attributes() {
            ensure!(
                attribute.namespace().is_none() && SVG_ATTRIBUTES.contains(&attribute.name()),
                "Unsafe SVG attribute: {}",
                attribute.name()
            );
            if ["fill", "stroke"].contains(&attribute.name()) {
                let value = attribute.value();
                ensure!(PAINT.is_match(value), "Unsafe paint value: {value}");
                if let Some(id) = value
                    .strip_prefix("url(#")
                    .and_then(|value| value.strip_suffix(')'))
                {
                    references.push(id);
                }
            }
        }
    }
    for id in references {
        ensure!(ids.contains_key(id), "Missing paint reference: {id}");
    }
    let label = root
        .attribute("aria-label")
        .is_some_and(|label| !label.trim().is_empty());
    let labelled_by = root.attribute("aria-labelledby").is_some_and(|attribute| {
        let references: Vec<_> = attribute.split_whitespace().collect();
        !references.is_empty()
            && references.iter().all(|id| {
                ids.get(id).is_some_and(|node| {
                    node.descendants()
                        .filter_map(|child| child.text())
                        .any(|text| !text.trim().is_empty())
                })
            })
    });
    ensure!(label || labelled_by, "Accessible SVG name required");
    Ok(())
}

fn mapping<'value>(value: &'value Value, label: &str) -> Result<&'value Mapping> {
    value
        .as_mapping()
        .with_context(|| format!("{label} must be a mapping"))
}

fn field<'value>(value: &'value Mapping, key: &str) -> Result<&'value Value> {
    value
        .get(Value::String(key.to_owned()))
        .with_context(|| format!("Missing workflow field: {key}"))
}

fn validate_permissions(value: &Value) -> Result<()> {
    let permissions = mapping(value, "permissions")?;
    ensure!(
        permissions.len() == 1 && field(permissions, "contents")?.as_str() == Some("read"),
        "Read-only contents permission required"
    );
    Ok(())
}

pub fn validate_workflow(source: &str) -> Result<()> {
    let workflow: Value = serde_yaml_ng::from_str(source).context("Invalid workflow YAML")?;
    let workflow = mapping(&workflow, "workflow")?;
    validate_permissions(field(workflow, "permissions")?)?;
    let events: Vec<&str> = match field(workflow, "on")? {
        Value::String(event) => vec![event],
        Value::Sequence(events) => events
            .iter()
            .map(|event| event.as_str().context("Invalid workflow trigger"))
            .collect::<Result<_>>()?,
        Value::Mapping(events) => events
            .keys()
            .map(|event| event.as_str().context("Invalid workflow trigger"))
            .collect::<Result<_>>()?,
        _ => anyhow::bail!("Invalid workflow triggers"),
    };
    ensure!(
        !events.is_empty()
            && events.iter().all(
                |event| ["push", "pull_request", "schedule", "workflow_dispatch"].contains(event)
            ),
        "Privileged or unsupported trigger"
    );
    let jobs = mapping(field(workflow, "jobs")?, "jobs")?;
    ensure!(!jobs.is_empty(), "At least one CI job required");
    for job in jobs.values() {
        let job = mapping(job, "job")?;
        ensure!(
            field(job, "runs-on")?.as_str() == Some("ubuntu-latest"),
            "GitHub-hosted runner required"
        );
        let timeout = field(job, "timeout-minutes")?
            .as_u64()
            .context("Integer job timeout required")?;
        ensure!((1..=20).contains(&timeout), "Bounded job timeout required");
        if let Some(permissions) = job.get("permissions") {
            validate_permissions(permissions)?;
        }
        let steps = field(job, "steps")?
            .as_sequence()
            .context("Job steps required")?;
        ensure!(!steps.is_empty(), "At least one job step required");
        for step in steps {
            let step = mapping(step, "step")?;
            if let Some(action) = step.get("uses") {
                let action = action.as_str().context("Action must be a string")?;
                ensure!(
                    ACTION.is_match(action),
                    "Action must use a full commit SHA: {action}"
                );
                if action.starts_with("actions/checkout@") {
                    let settings = mapping(field(step, "with")?, "checkout settings")?;
                    ensure!(
                        field(settings, "persist-credentials")?.as_bool() == Some(false),
                        "Checkout must not save credentials"
                    );
                }
            }
            if let Some(script) = step.get("run") {
                let script = script.as_str().context("Run script must be a string")?;
                ensure!(
                    !EVENT_DATA.is_match(script),
                    "Do not interpolate event data into shell scripts"
                );
            }
        }
    }
    Ok(())
}

fn repository_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if [".git", "target", "node_modules", "lychee"]
            .iter()
            .any(|ignored| entry.file_name() == *ignored)
        {
            continue;
        }
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "Repository symlink requires review: {}",
            entry.path().display()
        );
        if kind.is_dir() {
            repository_files(&entry.path(), files)?;
        } else if kind.is_file() {
            files.push(entry.path());
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct SafetyReport {
    pub svg_files: usize,
    pub workflows: usize,
    pub source_urls: usize,
}

pub fn check_repository(root: &Path) -> Result<SafetyReport> {
    let mut files = Vec::new();
    repository_files(root, &mut files).context("Enumerate repository files")?;
    files.sort();
    let mut report = SafetyReport {
        svg_files: 0,
        workflows: 0,
        source_urls: 0,
    };
    for filename in files {
        let relative = filename.strip_prefix(root)?;
        if filename
            .extension()
            .is_some_and(|extension| extension == "svg")
            && !relative.starts_with("millionaire/templates")
        {
            validate_svg(&fs::read_to_string(&filename)?)
                .with_context(|| format!("Validate {}", relative.display()))?;
            report.svg_files += 1;
        }
        if relative.starts_with(".github/workflows")
            && filename
                .extension()
                .is_some_and(|extension| extension == "yml" || extension == "yaml")
        {
            validate_workflow(&fs::read_to_string(&filename)?)
                .with_context(|| format!("Validate {}", relative.display()))?;
            report.workflows += 1;
        }
    }
    ensure!(report.workflows > 0, "At least one CI workflow required");
    let (bank, manifest) = load(root)?;
    bank.validate_manifest(&manifest)?;
    report.source_urls = bank.questions().len();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20" role="img" aria-label="Sample"><path fill="#116329" d="M0 0h20v20H0z"/></svg>"##;

    fn workflow() -> String {
        format!("name: Example\non: [pull_request]\npermissions:\n  contents: read\njobs:\n  check:\n    runs-on: ubuntu-latest\n    timeout-minutes: 10\n    steps:\n      - uses: actions/checkout@{}\n        with:\n          persist-credentials: false\n      - run: cargo test\n", "a".repeat(40))
    }

    #[test]
    fn static_svg_and_local_paint_references_are_accepted() {
        validate_svg(SVG).unwrap();
        let painted = SVG.replace("<path", "<defs><pattern id=\"grid\" width=\"2\" height=\"2\" patternUnits=\"userSpaceOnUse\"/></defs><path").replace("fill=\"#116329\"", "fill=\"url(#grid)\"");
        validate_svg(&painted).unwrap();
        let labelled = SVG
            .replace("aria-label=\"Sample\"", "aria-labelledby=\"title\"")
            .replace("<path", "<title id=\"title\">Sample</title><path");
        validate_svg(&labelled).unwrap();
    }

    #[test]
    fn svg_policy_rejects_unsafe_content_and_invalid_xml() {
        let invalid = [
            SVG.replace("</svg>", "<script>alert(1)</script></svg>"),
            SVG.replace("</svg>", "<foreignObject><p>HTML</p></foreignObject></svg>"),
            SVG.replace(
                "</svg>",
                "<image href=\"https://example.com/tracker.png\"/></svg>",
            ),
            SVG.replace("<path", "<path onload=\"alert(1)\""),
            SVG.replace("<path", "<path style=\"fill:red\""),
            SVG.replace(
                "fill=\"#116329\"",
                "fill=\"url(https://example.com/paint.svg)\"",
            ),
            SVG.replace("fill=\"#116329\"", "fill=\"url(#missing)\""),
            SVG.replace("aria-label=\"Sample\"", ""),
            SVG.replace("</svg>", ""),
            format!("<!DOCTYPE svg SYSTEM \"https://example.com/entity.dtd\">{SVG}"),
            format!("<?xml-stylesheet href=\"https://example.com/style.css\"?>{SVG}"),
            SVG.replace("width=\"20\"", "width=\"NaN\""),
            SVG.replace("<path", "<path xmlns=\"https://example.com/namespace\""),
        ];
        for source in invalid {
            assert!(validate_svg(&source).is_err(), "{source}");
        }
    }

    #[test]
    fn workflow_policy_rejects_mutable_or_privileged_execution() {
        let workflow = workflow();
        validate_workflow(&workflow).unwrap();
        let invalid = [
            workflow.replace("contents: read", "contents: write"),
            workflow.replace("pull_request", "pull_request_target"),
            workflow.replace("ubuntu-latest", "self-hosted"),
            workflow.replace("timeout-minutes: 10", "timeout-minutes: 60"),
            workflow.replace(&"a".repeat(40), "v4"),
            workflow.replace("persist-credentials: false", "persist-credentials: true"),
            workflow.replace(
                "cargo test",
                "echo '${{ github.event.pull_request.title }}'",
            ),
            format!("{workflow}permissions: write-all\n"),
        ];
        for source in invalid {
            assert!(validate_workflow(&source).is_err(), "{source}");
        }
    }

    #[test]
    fn action_shas_and_paint_values_require_ascii_characters() {
        let workflow = workflow().replace(&"a".repeat(40), &"\u{0661}".repeat(40));
        assert!(validate_workflow(&workflow).is_err());
        assert!(validate_svg(&SVG.replace("#116329", "#\u{0661}\u{0661}\u{0661}")).is_err());
    }

    #[test]
    fn repository_artwork_and_workflows_satisfy_policy() {
        let report = check_repository(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert!(report.svg_files >= 168);
        assert_eq!(report.workflows, 2);
        assert!(report.source_urls >= 47);
    }
}
