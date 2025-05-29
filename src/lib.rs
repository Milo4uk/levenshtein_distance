use levenshtein_shader::levenshtein;

pub mod runners;
pub use runners::wgpu_runner::levenshtein_gpu;

pub const WORDS_PADDING: usize = 64;
pub const SHADER: &[u8] = include_bytes!(env!("levenshtein_shader.spv"));

// take usual levenshtein for comparison and run it on CPU
pub fn levenshtein_distance(words: &[&str]) -> Vec<u32> {
    let number_of_words = words.len();
    let mut words_byted: Vec<u32> = Vec::with_capacity(number_of_words * WORDS_PADDING);
    let mut result = vec![0; number_of_words * number_of_words];

    // convert words to fixed-length u32 arrays with padding
    for w in words {
        assert!(w.len() <= WORDS_PADDING, "word too long");
        words_byted.extend(w.chars().map(|c| c as u32));
        words_byted.extend(std::iter::repeat(0).take(WORDS_PADDING - w.len()));
    }

    // calculate distances for all pairs
    for pair_idx in 0..number_of_words {
        let start = pair_idx * WORDS_PADDING;
        for compared_word_index in 0..number_of_words {
            let compared_word_start = compared_word_index * WORDS_PADDING;
            let dist = levenshtein(&words_byted, start, compared_word_start);
            result[pair_idx * number_of_words + compared_word_index] = dist;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_levenshtein() {
        let words = ["kitten", "sitting"];
        let distances = levenshtein_distance(&words);
        assert_eq!(distances, vec![0, 3, 3, 0]);
    }

    #[test]
    fn test_empty_words() {
        let words = ["", "test"];
        let distances = levenshtein_distance(&words);
        assert_eq!(distances, vec![0, 4, 4, 0]);
    }
}
