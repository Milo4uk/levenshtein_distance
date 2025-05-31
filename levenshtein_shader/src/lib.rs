#![no_std]

use glam::UVec3;
use spirv_std::{glam, spirv};

// words padding
const WORDS_PADDING: usize = 64;

/// Возвращает метрику, разность по модулю между двумя последовательностями символов
pub fn levenshtein(words: &[u32], start: usize, compared_word_start: usize) -> u32 {
    // problems: multi-byte characters in utf-8
    // this only works for ASCII characters
    // to note: doesn't support slicing

    let mut prev = [0u32; WORDS_PADDING + 1];
    let mut curr = [0u32; WORDS_PADDING + 1];

    // базовый алгоритм левенштейна из википедии
    for i in 0..WORDS_PADDING {
        // case when start = 0
        // we were reading garbage data all along
        // threads with global indices showed us that
        let a = if i == 0 { 0u32 } else { words[start + i - 1] };
        curr[0] = i as u32;
        for j in 0..WORDS_PADDING {
            let b = if j == 0 {
                0u32
            } else {
                words[compared_word_start + j - 1]
            };

            let cost: u32 = if a != b { 1 } else { 0 };

            let del: u32 = curr[j] + 1;
            let ins: u32 = prev[j + 1] + 1;
            let sub: u32 = prev[j] + cost;

            // we remove min, cause it causes InvalidTypeWidth(1) error
            // future note!!!
            // use THE simplest language, imagine u'r writing in C
            // imagine like you have one leg shot (u'r the crab) and disabled in means of using Rust
            let best_del_ins = if del < ins { del } else { ins };
            let best = if best_del_ins < sub {
                best_del_ins
            } else {
                sub
            };
            curr[j + 1] = best;
        }
        for j in 0..=WORDS_PADDING {
            prev[j] = curr[j];
        }
    }
    prev[WORDS_PADDING]
}

#[spirv(compute(threads(64)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    // слова в байтах единым array
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] words: &[u32],
    // метрика левенштейна для декартового произведения слов
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] output: &mut [u32],
) {
    // for each thread it's own id
    let pair_idx = id.x as usize;
    let start = pair_idx * WORDS_PADDING;

    // we will do cartesian product
    let number_of_words = words.len() / WORDS_PADDING;

    for compared_word_index in 0..number_of_words {
        let compared_word_start = compared_word_index * WORDS_PADDING;
        let dist = levenshtein(words, start, compared_word_start);
        output[pair_idx * number_of_words + compared_word_index] = dist;
    }
}
