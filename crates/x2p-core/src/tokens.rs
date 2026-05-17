use std::sync::OnceLock;

use tiktoken_rs::{cl100k_base, CoreBPE};

fn bpe() -> &'static CoreBPE {
    static BPE: OnceLock<CoreBPE> = OnceLock::new();
    BPE.get_or_init(|| cl100k_base().expect("cl100k tokenizer loads"))
}

pub fn count_cl100k(text: &str) -> usize {
    bpe().encode_with_special_tokens(text).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_nonzero_for_words() {
        assert!(count_cl100k("hello world") > 0);
    }

    #[test]
    fn empty_string_is_zero() {
        assert_eq!(count_cl100k(""), 0);
    }
}
