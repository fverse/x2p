use std::path::Path;

use anyhow::{Context, Result};
use x2p_core::{render, Bundle, RenderConfig};

pub fn run(bundle_path: &Path, budget: Option<usize>) -> Result<()> {
    let json = std::fs::read_to_string(bundle_path)
        .with_context(|| format!("reading {}", bundle_path.display()))?;
    let bundle: Bundle = serde_json::from_str(&json).context("parsing bundle JSON")?;
    let out = render(&bundle, &RenderConfig { budget_tokens: budget });
    print!("{out}");
    Ok(())
}
