#![no_std]

use core::cmp::min;

use glam::UVec3;
use spirv_std::{glam, spirv};

//fill the words up? the padding rn is 64
const MAX: usize = 64;

/// Возвращает метрику, разность по модулю между двумя последовательностями символов
pub fn levenshtein(words: &[u32]) -> Option<usize> {
    // String - UTF-8–encoded
    // problems: multi-byte characters in utf-8 
    // this only works for one byte per one character
    // to note: doesn't support slicing

    let n = MAX;

    let mut prev: [usize; MAX + 1] = [0usize; MAX + 1];
    let mut curr: [usize; MAX + 1] = [0usize; MAX + 1];

    for j in 0usize..=n { 
        prev[j] = j as usize; 
    }

    let mut i: usize = 0;
    let mut j: usize = 0;

    // базовый алгоритм левенштейна из википедии
    for a_byte in 0usize..=MAX {
        curr[0] = (i + 1usize) as usize;
        i += 1usize;
        for j in 0usize..=MAX {
            let cost  = (words[a_byte] != words[j]) as usize;
            // curr[j + 1usize] = min(
            //     // error: `i8` without `OpCapability Int8`
            //     min(curr[j] + 1usize,
            //         prev[j + 1usize] + 1usize),
            //     prev[j] + cost
            // );
        }
        for j in 0usize..=n { prev[j] = curr[j]; }
    }
    Some(prev[n])
}

#[spirv(compute(threads(1)))]
pub fn main_cs(
    //leave that out for now
    #[spirv(global_invocation_id)] _id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] words: & mut [u32], // слова единым array
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] output: & mut usize, // метрика левенштейна для пар слов
) {
    //current invocation index
    // let index = id.x as usize;

    *output = levenshtein(words).unwrap_or_default();
}