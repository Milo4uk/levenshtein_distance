#![no_std]

use core::cmp::min;
use spirv_std::spirv;

//fill the words up? the padding rn is 64
const MAX: usize = 64;

/// Возвращает метрику, разность по модулю между двумя последовательностями символов
pub fn levenshtein(words: &[u32]) -> Option<usize> {
    // String - UTF-8–encoded
    // problems: multi-byte characters in utf-8
    // this only works for one byte per one character
    // to note: doesn't support slicing
    if words.len() < MAX * 2 {
        return None;
    }

    let n = MAX;

    let mut prev: [usize; MAX + 1] = [0; MAX + 1];
    let mut curr: [usize; MAX + 1] = [0; MAX + 1];

    for j in 0..=MAX { 
        prev[j] = j; 
    }

    // базовый алгоритм левенштейна из википедии
    for i in 0..MAX {
        curr[0] = (i + 1) as usize;
        let a = words[i]; 
        for j in 0..MAX {
            //offset
            let b =  words[MAX + j];
            let cost = if a != b { 1 } else { 0 };
            curr[j + 1] = min(min(curr[j] + 1, prev[j + 1] + 1), prev[j] + cost);
        }
        for j in 0..=n {
            prev[j] = curr[j];
        }
    }
    Some(prev[n])
}

#[spirv(compute(threads(1)))]
pub fn main_cs(
    // #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] words: &mut [u32], // слова единым array
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] output: &mut usize, // метрика левенштейна для пар слов
) {
    //current invocation index
    // let index = id.x as usize;

    *output = levenshtein(words).unwrap_or(100);
}
