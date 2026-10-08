use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{ensure, Context, Result};
use regex::Regex;

use crate::render::{
    badge, escape_html, render_answers, render_jokers, render_template, themed_image, Theme,
};
use crate::trivia::{
    fifty_fifty, money, question_file, safety_net, Answer, Bank, Manifest, PRIZES,
};

static OPENING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)<!-- trivia:start -->.*?<!-- trivia:end -->").expect("constant README pattern")
});
static LINKS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\]\(\s*([^\s)]+)\s*\)|(?:href|src|srcset)="([^"]+)""#)
        .expect("constant link pattern")
});

pub fn load(root: &Path) -> Result<(Bank, Manifest)> {
    let bank = Bank::new(
        serde_json::from_str(&read_text(&root.join("millionaire/questions.json"))?)
            .context("Invalid question bank JSON")?,
    )?;
    let manifest = serde_json::from_str(&read_text(&root.join("millionaire/runs.json"))?)
        .context("Invalid run manifest JSON")?;
    Ok((bank, manifest))
}

fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Read {}", path.display()))
}

pub fn run_directory(id: &str) -> String {
    match id {
        "01" => "millionaire".to_owned(),
        "02" => "millionaire/history".to_owned(),
        _ => format!("millionaire/runs/{id}"),
    }
}

#[derive(Debug)]
pub struct Generated {
    files: BTreeMap<PathBuf, String>,
    pub runs: usize,
    pub questions: usize,
    pub pages: usize,
    pub hud_images: usize,
    pub local_links: usize,
    directories: Vec<PathBuf>,
}

impl Generated {
    pub fn create(root: &Path, selected_id: Option<&str>) -> Result<Self> {
        let (bank, manifest) = load(root)?;
        bank.validate_manifest(&manifest)?;
        let selected: Vec<_> = manifest
            .runs
            .iter()
            .filter(|run| selected_id.map_or(true, |id| id == run.id))
            .collect();
        ensure!(
            !selected.is_empty(),
            "Unknown run: {}",
            selected_id.unwrap_or_default()
        );
        let question_template = read_text(&root.join("millionaire/templates/question.md.tmpl"))?;
        let hud_template = read_text(&root.join("millionaire/templates/hud.svg"))?;
        let header_template = read_text(&root.join("millionaire/templates/header.svg"))?;
        let mut generated = Self {
            files: BTreeMap::new(),
            runs: selected.len(),
            questions: bank.questions().len(),
            pages: 0,
            hud_images: 0,
            local_links: 0,
            directories: Vec::new(),
        };
        for theme in Theme::ALL {
            let palette = theme.palette();
            let values = BTreeMap::from([
                ("GREEN", palette.green.to_owned()),
                ("AMBER", palette.amber.to_owned()),
                ("MUTED", palette.muted.to_owned()),
            ]);
            generated.files.insert(
                PathBuf::from(theme.path("assets/arcade.svg")),
                render_template(&header_template, &values, &["GREEN", "AMBER", "MUTED"])?,
            );
            for answer in Answer::ALL {
                let name = answer.to_string().to_ascii_lowercase();
                generated.files.insert(
                    PathBuf::from(theme.path(&format!("assets/trivia-{name}.svg"))),
                    badge(&name, &format!("> {answer}"), palette.green),
                );
            }
            for (name, label, color) in [
                ("question", "> QUESTION", palette.amber),
                ("fifty", "50:50", palette.green),
                ("hint", "[?]", palette.amber),
                ("cash", "[$]", palette.green),
            ] {
                generated.files.insert(
                    PathBuf::from(theme.path(&format!("assets/trivia-{name}.svg"))),
                    badge(name, label, color),
                );
            }
        }
        for run in selected {
            let directory = PathBuf::from(run_directory(&run.id));
            generated.directories.push(directory.clone());
            let depth = directory.components().count();
            let prefix = "../".repeat(depth);
            let arcade = format!("{prefix}README.md");
            let assets = format!("{prefix}assets/");
            let template = question_template.replace("../README.md", &arcade);
            let questions = run
                .questions
                .iter()
                .map(|id| bank.question(id))
                .collect::<Result<Vec<_>>>()?;
            for (index, question) in questions.iter().enumerate() {
                let prize = PRIZES[index];
                let current = if index == 0 { 0 } else { PRIZES[index - 1] };
                let floor = safety_net(index);
                let correct = if index == 14 {
                    "win.md".to_owned()
                } else {
                    question_file(index + 1)
                };
                let wrong = format!("game-over-{floor}.md");
                let targets = std::array::from_fn(|option| {
                    if option == question.correct.index() {
                        correct.clone()
                    } else {
                        wrong.clone()
                    }
                });
                let answers = render_answers(question, &targets, &assets)?;
                let jokers = render_jokers(question, current, index, &assets, &arcade, "q01.md")?;
                let remaining = fifty_fifty(question.correct);
                let previous = if index == 0 {
                    String::new()
                } else {
                    let last = questions[index - 1];
                    format!("<details>\n<summary>Previous answer</summary>\n\n## Q{index}: {} / {}\n\n{}\n\n[Source]({})\n\n</details>", last.correct, &last.options[last.correct], last.explanation, last.reference)
                };
                let checkpoint = if index == 5 || index == 10 {
                    format!("**Safety net secured: {}**", money(floor))
                } else {
                    String::new()
                };
                let ladder = PRIZES
                    .iter()
                    .enumerate()
                    .rev()
                    .map(|(step, prize)| {
                        let state = match step.cmp(&index) {
                            Ordering::Equal => "Current",
                            Ordering::Less => "Answered",
                            Ordering::Greater => "Next",
                        };
                        let checkpoint = if step == 4 || step == 9 {
                            " / Safety net"
                        } else {
                            ""
                        };
                        format!("| {} | {} | {state}{checkpoint} |", step + 1, money(*prize))
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let hud_path = format!("hud/q{:02}.svg", index + 1);
                let round = format!("{:02}", index + 1);
                let values = BTreeMap::from([
                    ("REFERENCE_LABEL", "Source".to_owned()),
                    ("HUD_PATH", hud_path.clone()),
                    (
                        "HUD",
                        themed_image(
                            &hud_path,
                            &format!(
                                "Round {round} of 15. Playing for {}. Bank {}. Safety net {}.",
                                money(prize),
                                money(current),
                                money(floor)
                            ),
                            None,
                            None,
                        )?,
                    ),
                    ("ROUND", round),
                    ("PRIZE", money(prize)),
                    ("BANK", money(current)),
                    ("SAFETY_NET", money(floor)),
                    ("COMPLETED", index.to_string()),
                    ("CHECKPOINT", checkpoint),
                    ("CATEGORY", question.category.clone()),
                    (
                        "QUESTION_BADGE",
                        themed_image(
                            &format!("{assets}trivia-question.svg"),
                            "Question",
                            None,
                            None,
                        )?,
                    ),
                    ("QUESTION", escape_html(&question.question)),
                    ("ANSWERS", answers.clone()),
                    ("JOKERS", jokers.clone()),
                    (
                        "FIFTY_FIFTY",
                        format!("{} and {}", remaining[0], remaining[1]),
                    ),
                    ("HINT", question.hint.clone()),
                    ("PREVIOUS_DEBRIEF", previous),
                    ("LADDER", ladder),
                    (
                        "ANSWER",
                        format!(
                            "{} / {}",
                            question.correct, &question.options[question.correct]
                        ),
                    ),
                    ("EXPLANATION", question.explanation.clone()),
                    ("REFERENCE", question.reference.clone()),
                ]);
                let page = format!(
                    "<!-- Run {}; question {}. -->\n{}",
                    run.id,
                    question.id,
                    render_template(
                        &template,
                        &values,
                        &["HUD", "QUESTION_BADGE", "QUESTION", "ANSWERS", "JOKERS"]
                    )?
                );
                ensure!(
                    page.contains(&answers) && page.contains(&jokers),
                    "Template must preserve answers and jokers"
                );
                ensure!(
                    page.matches("<picture>").count() == 9,
                    "Every terminal image must support both themes"
                );
                generated
                    .files
                    .insert(directory.join(question_file(index)), page);
                generated.pages += 1;
                for theme in Theme::ALL {
                    let palette = theme.palette();
                    let lights = (0..15)
                        .map(|step| {
                            let color = match step.cmp(&index) {
                                Ordering::Less => palette.green,
                                Ordering::Equal => palette.amber,
                                Ordering::Greater => palette.inactive,
                            };
                            format!(
                                "<path fill=\"{color}\" d=\"M{} 88h40v4h-40z\"/>",
                                32 + step * 44
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n  ");
                    let mut hud_values = values.clone();
                    hud_values.extend([
                        ("GREEN", palette.green.to_owned()),
                        ("MUTED", palette.muted.to_owned()),
                        ("PROGRESS_LIGHTS", lights),
                    ]);
                    generated.files.insert(
                        directory.join(theme.path(&hud_path)),
                        render_template(
                            &hud_template,
                            &hud_values,
                            &[
                                "ROUND",
                                "PRIZE",
                                "BANK",
                                "SAFETY_NET",
                                "PROGRESS_LIGHTS",
                                "GREEN",
                                "MUTED",
                            ],
                        )?,
                    );
                    generated.hud_images += 1;
                }
            }
            for floor in [0, 1_000, 32_000] {
                generated.files.insert(directory.join(format!("game-over-{floor}.md")), format!("# Game over\n\n**Final prize: {}**\n\nGo back for the answer and source.\n\n[Restart](q01.md) / [Games]({arcade})\n", money(floor)));
            }
            let last = questions[14];
            generated.files.insert(directory.join("win.md"), format!("# Complete\n\n**15 / 15. Final prize: $1,000,000**\n\n## Final answer: {} / {}\n\n{}\n\n[Source]({})\n\n[Restart](q01.md) / [Games]({arcade})\n", last.correct, &last.options[last.correct], last.explanation, last.reference));
            generated.pages += 4;
            if run.id == "01" {
                let first = questions[0];
                let opening_directory = run_directory(&run.id);
                let targets = std::array::from_fn(|index| {
                    format!(
                        "{opening_directory}/{}",
                        if index == first.correct.index() {
                            "q02.md"
                        } else {
                            "game-over-0.md"
                        }
                    )
                });
                let answers = render_answers(first, &targets, "assets/")?;
                let jokers = render_jokers(
                    first,
                    0,
                    0,
                    "assets/",
                    "README.md",
                    &format!("{opening_directory}/q01.md"),
                )?;
                let header = themed_image(
                    "assets/arcade.svg",
                    "Run 01. Question 1 of 15. Playing for $100. Bank $0. Safety net $0.",
                    None,
                    None,
                )?;
                let question_badge =
                    themed_image("assets/trivia-question.svg", "Question", None, None)?;
                let runs = manifest
                    .runs
                    .iter()
                    .map(|entry| format!("[Run {}]({}/q01.md)", entry.id, run_directory(&entry.id)))
                    .collect::<Vec<_>>()
                    .join(" / ");
                let opening = format!("<!-- trivia:start -->\n\n<a href=\"{opening_directory}/q01.md\">{header}</a>\n\n> {question_badge}\n>\n> **<samp>{}</samp>**\n\n  {answers}\n\n  {jokers}\n\n  <details>\n  <summary><samp>[+] Answer & source (spoiler)</samp></summary>\n\n**{}.** {}\n\n{}\n\n[Source]({})\n\n  </details>\n\n---\n\n{runs}\n\n  <!-- trivia:end -->", escape_html(&first.question), first.correct, &first.options[first.correct], first.explanation, first.reference);
                let readme = read_text(&root.join("README.md"))?;
                ensure!(
                    OPENING.find_iter(&readme).count() == 1,
                    "README must contain one trivia block"
                );
                generated.files.insert(
                    PathBuf::from("README.md"),
                    OPENING
                        .replace(&readme, regex::NoExpand(&opening))
                        .into_owned(),
                );
            }
        }
        generated.validate_existing_pages(root)?;
        generated.local_links = generated.validate_links(root)?;
        Ok(generated)
    }

    fn validate_existing_pages(&self, root: &Path) -> Result<()> {
        for directory in &self.directories {
            let full = root.join(directory);
            if !full.exists() {
                continue;
            }
            for entry in fs::read_dir(&full).with_context(|| format!("List {}", full.display()))? {
                let entry = entry?;
                if entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "md")
                {
                    ensure!(
                        self.files.contains_key(&directory.join(entry.file_name())),
                        "Unexpected game page: {}; remove stale generated pages explicitly",
                        entry.path().display()
                    );
                }
            }
        }
        Ok(())
    }

    fn validate_links(&self, root: &Path) -> Result<usize> {
        let mut pages: Vec<_> = self
            .files
            .iter()
            .filter(|(path, _)| {
                path.extension()
                    .is_some_and(|extension| extension == "md" || extension == "svg")
                    && !path.starts_with("assets")
            })
            .collect();
        let readme_path = PathBuf::from("README.md");
        let readme = read_text(&root.join(&readme_path))?;
        if !self.files.contains_key(&readme_path) {
            pages.push((&readme_path, &readme));
        }
        let mut count = 0;
        for (filename, content) in pages {
            ensure!(
                content.matches("<details>").count() == content.matches("</details>").count(),
                "Unbalanced details in {}",
                filename.display()
            );
            for capture in LINKS.captures_iter(content) {
                let target = capture
                    .get(1)
                    .or_else(|| capture.get(2))
                    .context("Missing link target")?
                    .as_str();
                if target.starts_with("https://") {
                    continue;
                }
                ensure!(
                    !target.contains([':', '#', '?', '\\']) && !target.starts_with('/'),
                    "Unsupported link in {}: {target}",
                    filename.display()
                );
                let resolved = resolve_local(filename, target)?;
                if !self.files.contains_key(&resolved) {
                    let canonical_root = root.canonicalize()?;
                    let canonical = root.join(&resolved).canonicalize().with_context(|| {
                        format!("Missing link in {}: {target}", filename.display())
                    })?;
                    ensure!(
                        canonical.starts_with(canonical_root) && canonical.is_file(),
                        "Link escapes repository or is not a file: {target}"
                    );
                }
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn verify(&self, root: &Path) -> Result<()> {
        for (filename, content) in &self.files {
            ensure!(
                read_text(&root.join(filename))? == *content,
                "{} is stale; run cargo run --locked -- build",
                filename.display()
            );
        }
        Ok(())
    }

    pub fn write(&self, root: &Path) -> Result<()> {
        for filename in self.files.keys() {
            let mut destination = root.to_path_buf();
            for component in filename.components() {
                ensure!(
                    matches!(component, Component::Normal(_)),
                    "Output paths must be repository-relative"
                );
                destination.push(component);
                match fs::symlink_metadata(&destination) {
                    Ok(metadata) => ensure!(
                        !metadata.is_symlink(),
                        "Refusing to write through symlink: {}",
                        destination.display()
                    ),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                    Err(error) => {
                        return Err(error)
                            .with_context(|| format!("Inspect {}", destination.display()))
                    }
                }
            }
        }
        for (filename, content) in &self.files {
            let destination = root.join(filename);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&destination, content)
                .with_context(|| format!("Write {}", destination.display()))?;
        }
        Ok(())
    }
}

pub fn resolve_local(filename: &Path, target: &str) -> Result<PathBuf> {
    let mut resolved = filename
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_path_buf();
    for component in Path::new(target).components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                ensure!(resolved.pop(), "Link escapes repository: {target}");
            }
            _ => anyhow::bail!("Absolute link forbidden: {target}"),
        }
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_directory() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR"));
        for filename in [
            "README.md",
            "millionaire/questions.json",
            "millionaire/runs.json",
            "millionaire/templates/question.md.tmpl",
            "millionaire/templates/hud.svg",
            "millionaire/templates/header.svg",
        ] {
            let target = directory.path().join(filename);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::copy(source.join(filename), target).unwrap();
        }
        directory
    }

    #[test]
    fn build_writes_a_fresh_repository_and_check_detects_stale_files_without_writing() {
        let directory = fixture_directory();
        let root = directory.path();
        let generated = Generated::create(root, None).unwrap();
        assert!(generated.verify(root).is_err());
        assert!(!root.join("assets").exists());
        generated.write(root).unwrap();
        generated.verify(root).unwrap();
        let stale = root.join("millionaire/q01.md");
        fs::write(&stale, "stale\n").unwrap();
        assert!(generated.verify(root).is_err());
        assert_eq!(fs::read_to_string(&stale).unwrap(), "stale\n");
        fs::write(root.join("millionaire/old.md"), "old\n").unwrap();
        assert!(Generated::create(root, None).is_err());
    }

    #[test]
    fn malformed_templates_and_readme_markers_fail_before_writes() {
        for (filename, content) in [
            ("millionaire/templates/question.md.tmpl", "{{UNKNOWN}}"),
            ("README.md", "# No trivia markers\n"),
        ] {
            let directory = fixture_directory();
            fs::write(directory.path().join(filename), content).unwrap();
            assert!(Generated::create(directory.path(), None).is_err());
            assert!(!directory.path().join("assets").exists());
            assert!(!directory.path().join("millionaire/q01.md").exists());
        }
    }

    #[cfg(unix)]
    #[test]
    fn output_symlinks_are_rejected_before_any_writes() {
        let directory = fixture_directory();
        let outside = tempfile::tempdir().unwrap();
        let generated = Generated::create(directory.path(), None).unwrap();
        std::os::unix::fs::symlink(outside.path(), directory.path().join("assets")).unwrap();
        assert!(generated.write(directory.path()).is_err());
        assert!(!directory.path().join("millionaire/q01.md").exists());
        assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
    }

    #[test]
    fn planning_accepts_new_questions_before_the_manifest_is_updated() {
        let directory = fixture_directory();
        let filename = directory.path().join("millionaire/questions.json");
        let mut questions: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&filename).unwrap()).unwrap();
        let mut added = questions[0].clone();
        added["id"] = serde_json::json!("new-question");
        questions.as_array_mut().unwrap().push(added);
        fs::write(filename, serde_json::to_string(&questions).unwrap()).unwrap();
        let (bank, manifest) = load(directory.path()).unwrap();
        assert!(bank.validate_manifest(&manifest).is_err());
        let proposed = Manifest {
            seed: manifest.seed.clone(),
            runs: bank.plan(&manifest.seed).unwrap(),
        };
        bank.validate_manifest(&proposed).unwrap();
        assert!(Generated::create(directory.path(), None).is_err());
        assert!(!directory.path().join("assets").exists());
    }

    #[test]
    fn all_published_outputs_are_byte_for_byte_compatible() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let generated = Generated::create(root, None).unwrap();
        assert_eq!(generated.pages, generated.runs * 19);
        assert_eq!(generated.hud_images, generated.runs * 30);
        generated.verify(root).unwrap();
    }

    #[test]
    fn selections_preserve_legacy_paths_and_reject_unknown_runs() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let generated = Generated::create(root, Some("02")).unwrap();
        assert_eq!(generated.runs, 1);
        assert!(generated
            .files
            .contains_key(Path::new("millionaire/history/q01.md")));
        assert!(!generated.files.contains_key(Path::new("README.md")));
        assert!(Generated::create(root, Some("99")).is_err());
        assert!(resolve_local(Path::new("README.md"), "../secret").is_err());
    }
}
