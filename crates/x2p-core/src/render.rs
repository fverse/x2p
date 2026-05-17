use std::fmt::Write as _;

use crate::model::{Block, Bundle};

#[derive(Debug, Clone, Default)]
pub struct RenderConfig {
    pub budget_tokens: Option<usize>,
}

pub fn render(bundle: &Bundle, cfg: &RenderConfig) -> String {
    let mut b = bundle.clone();
    if let Some(budget) = cfg.budget_tokens {
        crate::prune::shrink(&mut b, budget);
    }
    render_markdown(&b)
}

pub(crate) fn render_markdown(b: &Bundle) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {}", b.title);
    let _ = writeln!(out, "_Source: {} (captured {})_\n", b.url, b.captured_at);

    for block in &b.blocks {
        match block {
            Block::Heading { level, text } => {
                let hashes = "#".repeat((*level).clamp(1, 6) as usize);
                let _ = writeln!(out, "\n{hashes} {text}\n");
            }
            Block::Paragraph { text } => {
                let _ = writeln!(out, "{text}\n");
            }
            Block::List { ordered, items } => {
                for (i, item) in items.iter().enumerate() {
                    if *ordered {
                        let _ = writeln!(out, "{}. {}", i + 1, item);
                    } else {
                        let _ = writeln!(out, "- {item}");
                    }
                }
                out.push('\n');
            }
            Block::Code { lang, text } => {
                let _ = writeln!(out, "```{}", lang.as_deref().unwrap_or(""));
                out.push_str(text);
                if !text.ends_with('\n') {
                    out.push('\n');
                }
                let _ = writeln!(out, "```\n");
            }
            Block::Table { headers, rows } => {
                if headers.is_empty() {
                    continue;
                }
                let _ = writeln!(out, "| {} |", headers.join(" | "));
                let sep = headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | ");
                let _ = writeln!(out, "| {sep} |");
                for row in rows {
                    let _ = writeln!(out, "| {} |", row.join(" | "));
                }
                out.push('\n');
            }
            Block::Link { text, href } => {
                let _ = writeln!(out, "[{text}]({href})\n");
            }
            Block::Form { fields } => {
                let _ = writeln!(out, "**Form**");
                for (label, value) in fields {
                    let _ = writeln!(out, "- {label}: `{value}`");
                }
                out.push('\n');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_basic_markdown() {
        let b = Bundle {
            url: "u".into(),
            title: "T".into(),
            captured_at: 0,
            blocks: vec![
                Block::Heading { level: 2, text: "H".into() },
                Block::Paragraph { text: "P".into() },
            ],
        };
        let s = render(&b, &RenderConfig::default());
        assert!(s.contains("# T"));
        assert!(s.contains("## H"));
        assert!(s.contains("P"));
    }
}
