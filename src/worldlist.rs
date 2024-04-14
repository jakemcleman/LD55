use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub struct WordList {
    pub all_valid_words: HashMap<usize, Vec<String>>,
}

fn alphabet_index(letter: u8) -> usize {
    if letter >= 'A' as u8 && letter <= 'Z' as u8 {
        letter as usize - 'A' as usize
    }
    else if letter >= 'a' as u8 && letter <= 'z' as u8 {
        letter as usize - 'a' as usize   
    }
    else {
        panic!("not a letter");
    }
}

pub fn word_to_bits(word: &str) -> u32 {
    let mut val = 0;

    for c in word.as_bytes() {
        val |= 1 << alphabet_index(*c);
    }

    val
}

pub fn bits_to_letters(bits: u32) -> String {
    let mut s = String::new();

    for c in 'A'..'Z' {
        if bits & 1 << alphabet_index(c as u8) != 0 {
            s.push(c);
        }
    }

    s
}

fn is_valid_word(word: &str) -> bool {
    if word.len() < 4 {
        return false;
    }
    
    for c in word.as_bytes() {
        if !c.is_ascii_alphabetic() {
            return false;
        }
    }

    let distinct_letters = word_to_bits(word).count_ones() as usize;

    if distinct_letters < 4 { return false }

    let first_matches_last = word.as_bytes()[0] == word.as_bytes()[word.len() - 1];
    return first_matches_last && distinct_letters == (word.len() - 1);
}

impl Default for WordList {
    fn default() -> Self {
        let file_contents = include_str!("words/dict_words.txt");

        let mut list = WordList {
            all_valid_words: HashMap::new(),
        };

        for word in file_contents.split_whitespace() {
            if is_valid_word(word) {
                let distinct_letters = word_to_bits(word).count_ones() as usize;

                if !list.all_valid_words.contains_key(&distinct_letters) {
                    list.all_valid_words.insert(distinct_letters, Vec::new());
                }
                
                list.all_valid_words.get_mut(&distinct_letters).unwrap().push(String::from(word));
            }
        }

        for (length, list) in list.all_valid_words.iter() {
            println!("{} words of length {}", list.len(), length);
        }

        list
    }
}
