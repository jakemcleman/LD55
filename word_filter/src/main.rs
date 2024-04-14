use std::collections::BTreeMap;
use std::fs;
use std::env;
use std::io::BufWriter;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_file = &args[1];
    let output_file = &args[2];

    let contents = fs::read_to_string(input_file).expect("input file should be readable");

    let mut out_words = BTreeMap::new();

    for word in contents.split_whitespace() {
        if is_valid_word(word) {
            let length = word.len();

            if !out_words.contains_key(&length) {
                out_words.insert(length, Vec::new());
            }

            out_words.get_mut(&length).unwrap().push(String::from(word));
        }
    }

    

    for (length, list) in out_words.iter() {
        let file = fs::File::create(format!("{}{}", length, output_file)).expect("output file should be available");
        let mut writer = BufWriter::new(file);

        for word in list.iter() {
            writer.write_all(word.as_bytes()).expect("write should be possible");
            writer.write_all("\n".as_bytes()).expect("write should be possible");
        }
    }
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

