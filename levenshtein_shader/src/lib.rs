#![no_std]

use core::cmp::min;

use glam::UVec3;
use spirv_std::{glam, spirv};

const MAX: usize = 64;

pub fn levenshtein(w1: &[u8], w2: &[u8]) -> Option<u32> {
    // возвращает метрику, разность по модулю между двумя последовательностями символов
    // String - UTF-8–encoded
    // problems: multi-byte characters in utf-8 
    // this only works for one byte per one character

    if w1 == w2 {
        return 0;
    }

    if w1.len() == 0 {
        return Some(w2.len() as u32);
    }

    if w2.len() == 0 {
        return Some(w1.len() as u32);
    }

    let n = w2.len();
    if n > MAX {
        return None;
    }

    let mut prev: [u32; MAX + 1] = [0; MAX + 1];
    let mut curr: [u32; MAX + 1] = [0; MAX + 1];

    for j in 0..=n { 
        prev[j] = j as u32; 
    }

    // базовый алгоритм левенштейна из википедии
    for (i, &a_byte) in w1.iter().enumerate() {
        curr[0] = (i + 1) as u32;
        for (j, &b_byte) in w2.iter().enumerate() {
            let cost  = (a_byte != b_byte) as u32;
            curr[j + 1] = min(
                min(curr[j] + 1,
                    prev[j + 1] + 1),
                prev[j] + cost
            );
        }
        for j in 0..=n { prev[j] = curr[j]; }
    }
    Some(prev[n])
}

// #[spirv(compute(threads(64)))]
// pub fn main_cs(
//     #[spirv(global_invocation_id)] id: UVec3,
//     #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] prime_indices: &mut [u32],
// ) {
//     let index = id.x as usize;
//     prime_indices[index] = levenshtein(prime_indices[index]).unwrap_or(u32::MAX);
// }