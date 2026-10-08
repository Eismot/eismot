use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::ops::Index;

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

pub const PRIZES: [u32; 15] = [
    100, 200, 300, 500, 1_000, 2_000, 4_000, 8_000, 16_000, 32_000, 64_000, 125_000, 250_000,
    500_000, 1_000_000,
];

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Answer {
    A,
    B,
    C,
    D,
}

impl Answer {
    pub const ALL: [Self; 4] = [Self::A, Self::B, Self::C, Self::D];

    pub const fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
        }
    }
}

impl fmt::Display for Answer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Options {
    #[serde(rename = "A")]
    pub a: String,
    #[serde(rename = "B")]
    pub b: String,
    #[serde(rename = "C")]
    pub c: String,
    #[serde(rename = "D")]
    pub d: String,
}

impl Index<Answer> for Options {
    type Output = str;

    fn index(&self, answer: Answer) -> &Self::Output {
        match answer {
            Answer::A => &self.a,
            Answer::B => &self.b,
            Answer::C => &self.c,
            Answer::D => &self.d,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "u8")]
pub struct Difficulty(u8);

impl TryFrom<u8> for Difficulty {
    type Error = &'static str;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        if (1..=5).contains(&value) {
            Ok(Self(value))
        } else {
            Err("difficulty must be an integer from 1 to 5")
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub id: String,
    pub difficulty: Difficulty,
    pub topic: String,
    pub category: String,
    pub question: String,
    pub options: Options,
    pub correct: Answer,
    pub hint: String,
    pub explanation: String,
    pub reference: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub id: String,
    pub questions: [String; 15],
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub seed: String,
    pub runs: Vec<Run>,
}

pub struct Bank {
    questions: Vec<Question>,
    indices: BTreeMap<String, usize>,
}

impl Bank {
    pub fn new(questions: Vec<Question>) -> Result<Self> {
        ensure!(
            !questions.is_empty(),
            "Question bank must be a nonempty array"
        );
        let mut indices = BTreeMap::new();
        for (index, question) in questions.iter().enumerate() {
            ensure!(
                valid_id(&question.id),
                "Invalid question ID: {}",
                question.id
            );
            ensure!(
                indices.insert(question.id.clone(), index).is_none(),
                "Duplicate question ID: {}",
                question.id
            );
            let mut unique = BTreeSet::new();
            for answer in Answer::ALL {
                let option = &question.options[answer];
                ensure!(
                    !option.trim().is_empty(),
                    "{}: empty option {answer}",
                    question.id
                );
                ensure!(
                    unique.insert(option.trim()),
                    "{}: duplicate options",
                    question.id
                );
                validate_text(option)
                    .with_context(|| format!("{}: option {answer}", question.id))?;
            }
            for (field, text) in [
                ("topic", &question.topic),
                ("category", &question.category),
                ("question", &question.question),
                ("hint", &question.hint),
                ("explanation", &question.explanation),
            ] {
                validate_text(text).with_context(|| format!("{}: {field}", question.id))?;
            }
            validate_source_url(&question.reference)
                .with_context(|| format!("{}: reference", question.id))?;
        }
        for tier in 1..=5 {
            ensure!(
                questions
                    .iter()
                    .filter(|question| question.difficulty.0 == tier)
                    .count()
                    >= 3,
                "Tier {tier} needs at least three questions"
            );
        }
        Ok(Self { questions, indices })
    }

    pub fn questions(&self) -> &[Question] {
        &self.questions
    }

    pub fn question(&self, id: &str) -> Result<&Question> {
        self.indices
            .get(id)
            .map(|index| &self.questions[*index])
            .with_context(|| format!("Unknown question: {id}"))
    }

    pub fn validate_manifest(&self, manifest: &Manifest) -> Result<()> {
        ensure!(!manifest.seed.trim().is_empty(), "Manifest seed required");
        ensure!(!manifest.runs.is_empty(), "At least one run required");
        let mut run_ids = BTreeSet::new();
        let mut used = BTreeSet::new();
        for run in &manifest.runs {
            ensure!(
                run.id.len() == 2
                    && run.id.bytes().all(|byte| byte.is_ascii_digit())
                    && run.id != "00",
                "Run ID must contain two digits; 00 is reserved"
            );
            ensure!(
                run_ids.insert(run.id.as_str()),
                "Duplicate run ID: {}",
                run.id
            );
            ensure!(
                run.questions.iter().collect::<BTreeSet<_>>().len() == 15,
                "Run {}: repeated question",
                run.id
            );
            for (index, id) in run.questions.iter().enumerate() {
                let question = self.question(id)?;
                ensure!(
                    usize::from(question.difficulty.0) == index / 3 + 1,
                    "Run {}: wrong tier at round {}",
                    run.id,
                    index + 1
                );
                used.insert(id);
            }
        }
        ensure!(run_ids.contains("01"), "README's opening Run 01 required");
        ensure!(
            used.len() == self.questions.len(),
            "Every bank question must appear in at least one run"
        );
        Ok(())
    }

    pub fn plan(&self, seed: &str) -> Result<Vec<Run>> {
        ensure!(!seed.trim().is_empty(), "A nonempty run seed is required");
        let mut decks = Vec::new();
        for tier in 1..=5 {
            let mut ranked: Vec<_> = self
                .questions
                .iter()
                .filter(|question| question.difficulty.0 == tier)
                .map(|question| (Sha256::digest(format!("{seed}:{}", question.id)), question))
                .collect();
            ranked.sort_by(|left, right| {
                left.0
                    .cmp(&right.0)
                    .then_with(|| left.1.id.cmp(&right.1.id))
            });
            let mut deck: Vec<&Question> = Vec::new();
            while !ranked.is_empty() {
                let next = ranked
                    .iter()
                    .position(|entry| deck.last().map(|last| &last.topic) != Some(&entry.1.topic))
                    .unwrap_or(0);
                deck.push(ranked.remove(next).1);
            }
            decks.push(deck);
        }
        let count = decks
            .iter()
            .map(|deck| deck.len().div_ceil(3))
            .max()
            .context("Missing tiers")?;
        ensure!(count <= 99, "Planning requires more than 99 runs");
        Ok((0..count)
            .map(|run_index| Run {
                id: format!("{:02}", run_index + 1),
                questions: std::array::from_fn(|index| {
                    let deck = &decks[index / 3];
                    deck[(run_index * 3 + index % 3) % deck.len()].id.clone()
                }),
            })
            .collect())
    }
}

fn valid_id(id: &str) -> bool {
    id.split('-').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

fn validate_text(text: &str) -> Result<()> {
    ensure!(!text.trim().is_empty(), "Nonempty text required");
    ensure!(
        !text.contains(['\r', '\n', '<', '>', '[', ']', '|']),
        "Unsupported Markdown characters"
    );
    Ok(())
}

pub fn validate_source_url(reference: &str) -> Result<()> {
    let url = Url::parse(reference).context("Invalid source URL")?;
    ensure!(
        url.scheme() == "https" && url.username().is_empty() && url.password().is_none(),
        "Public HTTPS source without credentials required"
    );
    let host = url.host_str().context("Source hostname required")?;
    ensure!(
        matches!(url.host(), Some(url::Host::Domain(_)))
            && host != "localhost"
            && ![".local", ".localhost", ".internal", ".invalid"]
                .iter()
                .any(|suffix| host.ends_with(suffix)),
        "Public source hostname required"
    );
    Ok(())
}

pub fn money(amount: u32) -> String {
    let digits = amount.to_string();
    let mut formatted = String::from("$");
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(digit);
    }
    formatted
}

pub fn question_file(index: usize) -> String {
    format!("q{:02}.md", index + 1)
}

pub const fn safety_net(completed: usize) -> u32 {
    if completed >= 10 {
        32_000
    } else if completed >= 5 {
        1_000
    } else {
        0
    }
}

pub fn fifty_fifty(correct: Answer) -> [Answer; 2] {
    let other = Answer::ALL[(correct.index() + 2) % 4];
    if correct < other {
        [correct, other]
    } else {
        [other, correct]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_invalid_bank(change: impl FnOnce(&mut Vec<Question>)) {
        let (bank, _) = fixture();
        let mut questions = bank.questions().to_vec();
        change(&mut questions);
        assert!(Bank::new(questions).is_err());
    }

    fn assert_invalid_manifest(change: impl FnOnce(&mut Manifest)) {
        let (bank, mut manifest) = fixture();
        change(&mut manifest);
        assert!(bank.validate_manifest(&manifest).is_err());
    }

    #[test]
    fn rejects_duplicate_ids_sparse_tiers_unsafe_text_and_empty_options() {
        assert_invalid_bank(|questions| questions.push(questions[0].clone()));
        assert_invalid_bank(|questions| questions.retain(|question| question.difficulty.0 != 5));
        assert_invalid_bank(|questions| questions[0].id = "Invalid ID".to_owned());
        assert_invalid_bank(|questions| questions[0].options.a = " ".to_owned());
        assert_invalid_bank(|questions| questions[0].topic.clear());
        assert_invalid_bank(|questions| questions[0].hint = "Unsafe [Markdown]".to_owned());
        assert_invalid_bank(|questions| questions[0].question.push('\n'));
    }

    #[test]
    fn rejects_repeated_unknown_uncovered_and_reserved_routes() {
        assert_invalid_manifest(|manifest| {
            manifest.runs[0].questions[1] = manifest.runs[0].questions[0].clone()
        });
        assert_invalid_manifest(|manifest| manifest.runs[0].questions[0] = "missing".to_owned());
        assert_invalid_manifest(|manifest| manifest.runs.truncate(1));
        assert_invalid_manifest(|manifest| manifest.runs.push(manifest.runs[0].clone()));
        assert_invalid_manifest(|manifest| manifest.runs[0].id = "00".to_owned());
        assert_invalid_manifest(|manifest| manifest.runs[0].id = "06".to_owned());
        assert_invalid_manifest(|manifest| manifest.seed.clear());
    }

    #[test]
    fn schema_rejects_question_level_prizes_and_wrong_round_counts() {
        let mut bank: serde_json::Value =
            serde_json::from_str(include_str!("../millionaire/questions.json")).unwrap();
        bank[0]["prize"] = serde_json::json!(100);
        assert!(serde_json::from_value::<Vec<Question>>(bank).is_err());
        let mut manifest: serde_json::Value =
            serde_json::from_str(include_str!("../millionaire/runs.json")).unwrap();
        manifest["runs"][0]["questions"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(serde_json::from_value::<Manifest>(manifest).is_err());
        let (bank, manifest) = fixture();
        assert_ne!(
            bank.plan(&manifest.seed).unwrap(),
            bank.plan("other-seed").unwrap()
        );
        assert!(bank.questions().len() >= 47);
        assert!(manifest.runs.len() >= 5);
        let prohibited = regex::Regex::new("(?i)muratori|which year|when did").unwrap();
        assert!(bank
            .questions()
            .iter()
            .all(|question| !prohibited.is_match(&question.question)));
    }

    fn fixture() -> (Bank, Manifest) {
        let questions =
            serde_json::from_str(include_str!("../millionaire/questions.json")).unwrap();
        let manifest = serde_json::from_str(include_str!("../millionaire/runs.json")).unwrap();
        (Bank::new(questions).unwrap(), manifest)
    }

    #[test]
    fn published_manifest_is_valid_and_planning_is_order_independent() {
        let (bank, manifest) = fixture();
        bank.validate_manifest(&manifest).unwrap();
        let planned = bank.plan(&manifest.seed).unwrap();
        bank.validate_manifest(&Manifest {
            seed: manifest.seed.clone(),
            runs: planned.clone(),
        })
        .unwrap();
        let mut reversed = bank.questions().to_vec();
        reversed.reverse();
        assert_eq!(
            planned,
            Bank::new(reversed).unwrap().plan(&manifest.seed).unwrap()
        );
    }

    #[test]
    fn rejects_invalid_models_during_deserialization() {
        assert!(serde_json::from_str::<Answer>("\"E\"").is_err());
        for value in ["0", "6", "1.5", "\"1\""] {
            assert!(serde_json::from_str::<Difficulty>(value).is_err());
        }
        assert!(serde_json::from_str::<Options>(r#"{"A":"a","B":"b","C":"c"}"#).is_err());
        assert!(
            serde_json::from_str::<Options>(r#"{"A":"a","B":"b","C":"c","D":"d","E":"e"}"#)
                .is_err()
        );
    }

    #[test]
    fn rules_preserve_ladder_checkpoints_and_jokers() {
        assert_eq!(money(1_000_000), "$1,000,000");
        assert_eq!(question_file(14), "q15.md");
        for completed in 0..=15 {
            let expected = if completed >= 10 {
                32_000
            } else if completed >= 5 {
                1_000
            } else {
                0
            };
            assert_eq!(safety_net(completed), expected);
        }
        for correct in Answer::ALL {
            let remaining = fifty_fifty(correct);
            assert!(remaining.contains(&correct));
            assert!(remaining[0] < remaining[1]);
        }
    }

    #[test]
    fn invalid_banks_and_routes_are_rejected() {
        let (bank, mut manifest) = fixture();
        let mut questions = bank.questions().to_vec();
        questions[0].options.b = questions[0].options.a.clone();
        assert!(Bank::new(questions).is_err());
        manifest.runs[0].questions.swap(0, 14);
        assert!(bank.validate_manifest(&manifest).is_err());
        assert!(bank.plan(" ").is_err());
    }

    #[test]
    fn sources_require_public_https_without_credentials() {
        validate_source_url("https://example.com/source#section").unwrap();
        for source in [
            "http://example.com",
            "https://user:secret@example.com",
            "https://127.0.0.1",
            "https://[::1]",
            "https://localhost",
            "https://host.internal",
        ] {
            assert!(validate_source_url(source).is_err(), "{source}");
        }
    }
}
