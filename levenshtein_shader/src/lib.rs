#![no_std]

use spirv_std::spirv;

//fill the words up? the padding rn is 64
const MAX: usize = 64;

/// Возвращает метрику, разность по модулю между двумя последовательностями символов
pub fn levenshtein(words: &[u32]) -> Option<usize> {
    // String - UTF-8–encoded
    // problems: multi-byte characters in utf-8
    // this only works for one byte per one character
    // to note: doesn't support slicing

    let mut prev: [usize; MAX + 1] = [0; MAX + 1];
    let mut curr: [usize; MAX + 1] = [0; MAX + 1];

    // базовый алгоритм левенштейна из википедии
    for i in 0..MAX {
        curr[0] = i as usize;
        let a = words[i]; 
        for j in 0..MAX {
            let b =  words[MAX + j];
            let cost = if a != b { 1 } else { 0 };

            let del = curr[j] + 1;
            let ins = prev[j + 1] + 1;
            let sub = prev[j] + cost;
             
            // we remove min, cause it causes InvalidTypeWidth(1) error
            // future note!!!
            // use THE simplest language, imagine u'r writing in C
            // imagine like you have one leg shot (u'r the crab) and disabled in means of using Rust
            let best_del_ins = if del < ins { del } else { ins };
            let best = if best_del_ins < sub { best_del_ins } else { sub };

            curr[j + 1] = best;
        }
        for j in 0..=MAX {
            prev[j] = curr[j];
        }
    }
    Some(prev[MAX])
}

#[spirv(compute(threads(1)))]
pub fn main_cs(
    // #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] words: &mut [u32], // слова единым array
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] output: &mut usize, // метрика левенштейна для пар слов
) {
    // we don't need index rn
    // let index = id.x as usize;

    *output = levenshtein(words).unwrap_or(100);
}
