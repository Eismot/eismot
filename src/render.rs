use std::collections::BTreeMap;
use std::sync::LazyLock;

use anyhow::{ensure, Context, Result};
use regex::Regex;

use crate::trivia::{fifty_fifty, money, Answer, Question};

static TOKENS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{([A-Z_]+)\}\}").expect("constant token pattern"));
static BLANK_LINES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{3,}").expect("constant whitespace pattern"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

pub struct Palette {
    pub green: &'static str,
    pub amber: &'static str,
    pub muted: &'static str,
    pub inactive: &'static str,
}

impl Theme {
    pub const ALL: [Self; 2] = [Self::Light, Self::Dark];

    pub const fn palette(self) -> Palette {
        match self {
            Self::Light => Palette {
                green: "#116329",
                amber: "#7d4e00",
                muted: "#57606a",
                inactive: "#d0d7de",
            },
            Self::Dark => Palette {
                green: "#9be9a8",
                amber: "#f3d889",
                muted: "#a2b6a8",
                inactive: "#304637",
            },
        }
    }

    pub fn path(self, filename: &str) -> String {
        match (self, filename.strip_suffix(".svg")) {
            (Self::Dark, Some(stem)) => format!("{stem}-dark.svg"),
            _ => filename.to_owned(),
        }
    }
}

pub fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

pub fn themed_image(
    filename: &str,
    alt: &str,
    width: Option<&str>,
    title: Option<&str>,
) -> Result<String> {
    ensure!(filename.ends_with(".svg"), "Theme images must be SVG files");
    let mut attributes = String::new();
    if let Some(width) = width {
        attributes.push_str(&format!(" width=\"{}\"", escape_html(width)));
    }
    if let Some(title) = title {
        attributes.push_str(&format!(" title=\"{}\"", escape_html(title)));
    }
    Ok(format!("<picture><source media=\"(prefers-color-scheme: dark)\" srcset=\"{}\"><source media=\"(prefers-color-scheme: light)\" srcset=\"{}\"><img src=\"{}\" alt=\"{}\"{attributes}></picture>",
        escape_html(&Theme::Dark.path(filename)), escape_html(filename), escape_html(filename), escape_html(alt)))
}

pub fn render_template(
    template: &str,
    values: &BTreeMap<&str, String>,
    required: &[&str],
) -> Result<String> {
    for token in required {
        ensure!(
            TOKENS
                .captures_iter(template)
                .any(|capture| &capture[1] == *token),
            "Template must include {{{{{token}}}}}"
        );
    }
    let mut rendered = String::with_capacity(template.len());
    let mut previous = 0;
    for capture in TOKENS.captures_iter(template) {
        let matched = capture.get(0).context("Missing template match")?;
        rendered.push_str(&template[previous..matched.start()]);
        rendered.push_str(
            values
                .get(&capture[1])
                .with_context(|| format!("Unknown template token: {{{{{}}}}}", &capture[1]))?,
        );
        previous = matched.end();
    }
    rendered.push_str(&template[previous..]);
    ensure!(
        !rendered.contains("{{") && !rendered.contains("}}"),
        "Unresolved or malformed template token"
    );
    let normalized = rendered.replace("\r\n", "\n");
    Ok(format!(
        "{}\n",
        BLANK_LINES.replace_all(&normalized, "\n\n").trim_end()
    ))
}

pub fn render_answers(question: &Question, targets: &[String; 4], prefix: &str) -> Result<String> {
    let mut cells = Vec::with_capacity(4);
    for answer in Answer::ALL {
        let image = themed_image(
            &format!(
                "{prefix}trivia-{}.svg",
                answer.to_string().to_ascii_lowercase()
            ),
            &answer.to_string(),
            Some("100%"),
            None,
        )?;
        cells.push(format!(
            "<td width=\"50%\"><a href=\"{}\">{image}<br><samp>{}</samp></a></td>",
            escape_html(&targets[answer.index()]),
            escape_html(&question.options[answer])
        ));
    }
    Ok(format!(
        "<table width=\"100%\">\n<tr>\n{}\n</tr>\n<tr>\n{}\n</tr>\n</table>",
        cells[..2].join("\n"),
        cells[2..].join("\n")
    ))
}

pub fn render_jokers(
    question: &Question,
    bank: u32,
    completed: usize,
    prefix: &str,
    games: &str,
    restart: &str,
) -> Result<String> {
    let choices = fifty_fifty(question.correct);
    let remaining = format!("{} and {}", choices[0], choices[1]);
    let fifty = themed_image(
        &format!("{prefix}trivia-fifty.svg"),
        "50:50",
        Some("100%"),
        Some("50:50: once per run"),
    )?;
    let hint = themed_image(
        &format!("{prefix}trivia-hint.svg"),
        "Hint",
        Some("100%"),
        Some("Hint: once per run"),
    )?;
    let cash = themed_image(
        &format!("{prefix}trivia-cash.svg"),
        "Cash out",
        Some("100%"),
        Some(&format!("Cash out: {}", money(bank))),
    )?;
    Ok(format!("<table width=\"100%\">
<tr>
<td width=\"33%\" valign=\"top\"><details><summary>{fifty}</summary><p><samp>{remaining}</samp></p></details></td>
<td width=\"34%\" valign=\"top\"><details><summary>{hint}</summary><p><samp>{}</samp></p></details></td>
<td width=\"33%\" valign=\"top\"><details><summary>{cash}</summary><p><samp>{} / {completed} of 15 answered.</samp></p><p><a href=\"{}\">Games</a> / <a href=\"{}\">Restart</a></p></details></td>
</tr>
</table>", escape_html(&question.hint), money(bank), escape_html(games), escape_html(restart)))
}

pub fn badge(name: &str, label: &str, color: &str) -> String {
    let width = if ["fifty", "hint", "cash"].contains(&name) {
        160
    } else {
        320
    };
    let font_size = if name == "question" { 20 } else { 26 };
    let border = "\u{2500}".repeat(if width == 160 { 16 } else { 36 });
    let label = escape_html(label);
    format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"56\" viewBox=\"0 0 {width} 56\" role=\"img\" aria-label=\"{label}\">
    <g font-family=\"Consolas, 'Liberation Mono', monospace\" fill=\"{color}\">
      <g font-size=\"14\" fill-opacity=\".6\">
        <text x=\"8\" y=\"14\" textLength=\"{}\" lengthAdjust=\"spacingAndGlyphs\">\u{250c}{border}\u{2510}</text>
        <text x=\"8\" y=\"34\">\u{2502}</text>
        <text x=\"{}\" y=\"34\">\u{2502}</text>
        <text x=\"8\" y=\"52\" textLength=\"{}\" lengthAdjust=\"spacingAndGlyphs\">\u{2514}{border}\u{2518}</text>
      </g>
      <text x=\"24\" y=\"36\" font-size=\"{font_size}\">{label}</text>
      <text x=\"{}\" y=\"36\" font-size=\"20\" fill-opacity=\".65\">\u{258c}</text>
    </g>
  </svg>\n", width - 16, width - 16, width - 16, width - 34)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_are_strict_and_preserve_literal_values() {
        let values = BTreeMap::from([("HINT", "$100 and $& stay literal".to_owned())]);
        assert_eq!(
            render_template("{{HINT}}\r\n\r\n\r\nEnd  ", &values, &["HINT"]).unwrap(),
            "$100 and $& stay literal\n\nEnd\n"
        );
        assert!(render_template("{{UNKNOWN}}", &values, &[]).is_err());
        assert!(render_template("No question", &values, &["QUESTION"]).is_err());
        assert!(render_template("{{wrong}}", &values, &[]).is_err());
        assert_eq!(escape_html("&<>\"'"), "&amp;&lt;&gt;&quot;&#39;");
    }

    #[test]
    fn images_have_accessible_light_and_dark_variants() {
        let image = themed_image(
            "../assets/trivia-a.svg",
            "A & B",
            Some("100%"),
            Some("\"Hint\""),
        )
        .unwrap();
        assert!(image.contains("srcset=\"../assets/trivia-a-dark.svg\""));
        assert!(image.contains("src=\"../assets/trivia-a.svg\" alt=\"A &amp; B\" width=\"100%\" title=\"&quot;Hint&quot;\""));
        assert!(themed_image("image.png", "Image", None, None).is_err());
    }

    #[test]
    fn answers_and_jokers_preserve_layout_and_navigation() {
        let questions: Vec<Question> =
            serde_json::from_str(include_str!("../millionaire/questions.json")).unwrap();
        let targets = std::array::from_fn(|index| format!("target-{index}.md"));
        let answers = render_answers(&questions[0], &targets, "../assets/").unwrap();
        assert_eq!(answers.matches("<tr>").count(), 2);
        assert_eq!(answers.matches("<td ").count(), 4);
        for target in targets {
            assert!(answers.contains(&format!("href=\"{target}\"")));
        }
        for correct in Answer::ALL {
            let mut question = questions[0].clone();
            question.correct = correct;
            question.hint = "A & B".to_owned();
            let jokers =
                render_jokers(&question, 1_000, 5, "../assets/", "../README.md", "q01.md").unwrap();
            assert_eq!(jokers.matches("<details>").count(), 3);
            assert_eq!(jokers.matches("<picture>").count(), 3);
            assert!(jokers.contains("<samp>A &amp; B</samp>"));
            assert!(jokers.contains("$1,000 / 5 of 15 answered."));
            assert!(jokers.contains("href=\"../README.md\""));
        }
    }
}
