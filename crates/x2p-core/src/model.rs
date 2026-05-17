use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub url: String,
    pub title: String,
    pub captured_at: i64,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Block {
    Heading { level: u8, text: String },
    Paragraph { text: String },
    List { ordered: bool, items: Vec<String> },
    Code { lang: Option<String>, text: String },
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    Link { text: String, href: String },
    Form { fields: Vec<(String, String)> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_round_trips_through_json() {
        let b = Bundle {
            url: "https://example.com".into(),
            title: "Example".into(),
            captured_at: 1_700_000_000_000,
            blocks: vec![
                Block::Heading { level: 1, text: "Hello".into() },
                Block::Paragraph { text: "World.".into() },
                Block::List {
                    ordered: false,
                    items: vec!["a".into(), "b".into()],
                },
                Block::Code {
                    lang: Some("rust".into()),
                    text: "fn main() {}".into(),
                },
                Block::Table {
                    headers: vec!["k".into(), "v".into()],
                    rows: vec![vec!["x".into(), "1".into()]],
                },
                Block::Link {
                    text: "home".into(),
                    href: "/".into(),
                },
                Block::Form {
                    fields: vec![("name".into(), "".into())],
                },
            ],
        };
        let json = serde_json::to_string(&b).unwrap();
        let back: Bundle = serde_json::from_str(&json).unwrap();
        assert_eq!(back.blocks.len(), b.blocks.len());
        assert_eq!(back.title, b.title);
    }
}
