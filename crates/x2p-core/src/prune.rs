use crate::model::{Block, Bundle};
use crate::render::render_markdown;
use crate::tokens::count_cl100k;

pub fn shrink(bundle: &mut Bundle, budget: usize) {
    while count_cl100k(&render_markdown(bundle)) > budget {
        if !drop_one(bundle) {
            return;
        }
    }
}

fn drop_one(bundle: &mut Bundle) -> bool {
    let priority = |b: &Block| match b {
        Block::Heading { .. } => 100,
        Block::Code { .. } => 80,
        Block::Table { .. } => 70,
        Block::List { .. } => 60,
        Block::Form { .. } => 50,
        Block::Link { .. } => 40,
        Block::Paragraph { .. } => 30,
    };

    let target = bundle
        .blocks
        .iter()
        .enumerate()
        .min_by_key(|(i, b)| (priority(b), usize::MAX - i))
        .map(|(i, _)| i);

    match target {
        Some(i) if !matches!(bundle.blocks[i], Block::Heading { .. }) => {
            bundle.blocks.remove(i);
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::RenderConfig;

    #[test]
    fn shrink_reduces_token_count_to_budget() {
        let mut blocks = vec![Block::Heading { level: 1, text: "T".into() }];
        for i in 0..50 {
            blocks.push(Block::Paragraph {
                text: format!("This is paragraph number {i} with some filler content to consume tokens."),
            });
        }
        let bundle = Bundle {
            url: "u".into(),
            title: "T".into(),
            captured_at: 0,
            blocks,
        };
        let budget = 100;
        let rendered = crate::render::render(&bundle, &RenderConfig { budget_tokens: Some(budget) });
        assert!(count_cl100k(&rendered) <= budget, "{} > {budget}", count_cl100k(&rendered));
    }
}
