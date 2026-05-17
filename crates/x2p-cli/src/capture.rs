use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use scraper::{ElementRef, Html, Selector};
use x2p_core::{Block, Bundle};

pub async fn run(url: &str, output: Option<&Path>) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("x2p/", env!("CARGO_PKG_VERSION"), " (+https://github.com/x2p-platform/x2p)"))
        .build()
        .context("building http client")?;
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("fetching {url}"))?
        .error_for_status()
        .with_context(|| format!("{url} returned a non-success status"))?;
    let html = resp.text().await.context("reading response body")?;
    let bundle = parse(url, &html);
    let json = serde_json::to_string_pretty(&bundle)?;
    match output {
        Some(p) => std::fs::write(p, json).with_context(|| format!("writing {}", p.display()))?,
        None => println!("{json}"),
    }
    Ok(())
}

fn parse(url: &str, html: &str) -> Bundle {
    let doc = Html::parse_document(html);
    let title = extract_title(&doc);
    let mut blocks = Vec::new();
    if let Ok(sel) = Selector::parse("body") {
        if let Some(body) = doc.select(&sel).next() {
            walk(body, &mut blocks);
        }
    }
    Bundle {
        url: url.into(),
        title,
        captured_at: now_millis(),
        blocks,
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn extract_title(doc: &Html) -> String {
    Selector::parse("title")
        .ok()
        .and_then(|s| doc.select(&s).next().map(|e| flatten_text(e.text())))
        .unwrap_or_default()
}

fn flatten_text<'a, I: Iterator<Item = &'a str>>(it: I) -> String {
    // Join fragments with a space first, so that adjacent inline elements
    // (e.g. <div>01</div><div>Burrata</div>) don't smash into "01Burrata".
    // Then normalize runs of whitespace into single spaces.
    let joined = it.collect::<Vec<&str>>().join(" ");
    joined.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn text_of(el: ElementRef<'_>) -> String {
    flatten_text(el.text())
}

const SKIP_TAGS: &[&str] = &["script", "style", "nav", "header", "footer", "aside", "noscript", "svg"];

fn walk(el: ElementRef<'_>, blocks: &mut Vec<Block>) {
    let tag = el.value().name();
    if SKIP_TAGS.contains(&tag) {
        return;
    }
    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level: u8 = tag[1..].parse().unwrap_or(1);
            let text = text_of(el);
            if !text.is_empty() {
                blocks.push(Block::Heading { level, text });
            }
        }
        "p" => {
            let text = text_of(el);
            if !text.is_empty() {
                blocks.push(Block::Paragraph { text });
            }
        }
        "ul" => emit_list(false, el, blocks),
        "ol" => emit_list(true, el, blocks),
        "pre" => emit_code(el, blocks),
        "table" => emit_table(el, blocks),
        _ => recurse(el, blocks),
    }
}

fn recurse(el: ElementRef<'_>, blocks: &mut Vec<Block>) {
    for child in el.children() {
        if let Some(child_el) = ElementRef::wrap(child) {
            walk(child_el, blocks);
        }
    }
}

fn emit_list(ordered: bool, el: ElementRef<'_>, blocks: &mut Vec<Block>) {
    let li = match Selector::parse(":scope > li") {
        Ok(s) => s,
        Err(_) => return,
    };
    let items: Vec<String> = el
        .select(&li)
        .map(text_of)
        .filter(|s| !s.is_empty())
        .collect();
    if !items.is_empty() {
        blocks.push(Block::List { ordered, items });
    }
}

fn emit_code(pre: ElementRef<'_>, blocks: &mut Vec<Block>) {
    let code_sel = match Selector::parse("code") {
        Ok(s) => s,
        Err(_) => return,
    };
    let (lang, text) = if let Some(code) = pre.select(&code_sel).next() {
        let lang = code
            .value()
            .attr("class")
            .and_then(|c| {
                c.split_whitespace()
                    .find(|tok| tok.starts_with("language-"))
                    .map(|tok| tok.trim_start_matches("language-").to_string())
            });
        let raw: String = code.text().collect();
        (lang, raw)
    } else {
        (None, pre.text().collect::<String>())
    };
    let text = text.trim_matches('\n').to_string();
    if !text.is_empty() {
        blocks.push(Block::Code { lang, text });
    }
}

fn emit_table(table: ElementRef<'_>, blocks: &mut Vec<Block>) {
    let (Ok(thead_th), Ok(tr_sel), Ok(cell_sel)) = (
        Selector::parse("thead tr th"),
        Selector::parse("tr"),
        Selector::parse("td, th"),
    ) else {
        return;
    };

    let mut headers: Vec<String> = table.select(&thead_th).map(text_of).collect();
    let mut rows: Vec<Vec<String>> = table
        .select(&tr_sel)
        .map(|r| r.select(&cell_sel).map(text_of).collect::<Vec<_>>())
        .filter(|r| !r.is_empty())
        .collect();

    if headers.is_empty() && !rows.is_empty() {
        headers = rows.remove(0);
    }

    if !headers.is_empty() {
        blocks.push(Block::Table { headers, rows });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_html_into_blocks() {
        let html = r#"
            <html><head><title>  My Title  </title></head>
            <body>
                <nav>skip me</nav>
                <h1>Hello</h1>
                <p>Some paragraph text.</p>
                <ul><li>one</li><li>two</li></ul>
                <pre><code class="language-rust">fn x() {}</code></pre>
            </body></html>
        "#;
        let b = parse("https://x", html);
        assert_eq!(b.title, "My Title");
        let kinds: Vec<&str> = b.blocks.iter().map(|b| match b {
            Block::Heading { .. } => "h",
            Block::Paragraph { .. } => "p",
            Block::List { .. } => "l",
            Block::Code { .. } => "c",
            _ => "?",
        }).collect();
        assert_eq!(kinds, vec!["h", "p", "l", "c"]);
    }
}
