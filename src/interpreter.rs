#![feature(slicing_syntax)]

extern crate turing;

use turing::{TMDesc, TM};
use std::io::{File, BufferedReader};

fn main() {
    let file = File::open(&Path::new("../jn.tm"))
        .unwrap_or_else(|e| panic!("can't open file: {}", e));
    let mut reader = BufferedReader::new(file);

    let mut tmd = TMDesc::new();

    for line_res in reader.lines() {
        if let Ok(line) = line_res {
            if let Ok(words) = TMDesc::parse_line(line.trim_right_chars('\n')) {
                tmd.handle_line(words[]);
            }
        }
    }

    //println!("The TM is now configured as follows:");
    //println!("");
    //println!("{}", json::encode(tm));
    //println!("{}", tmd);

    let input_file = File::open(&Path::new("../big-troll.txt"))
        .unwrap_or_else(|e| panic!("can't open input: {}", e));
    let mut input_reader = BufferedReader::new(input_file);
    let input = input_reader.read_to_string()
        .unwrap_or_else(|e| panic!("input is ded: {}", e));

    let mut filtered = String::new();
    for c in input.chars().filter(|ch| !ch.is_whitespace()) {
        filtered.push(c)
    }
    let mut tm = TM::new(&tmd, filtered[]);

    let mut steps = 0u;
    while !tm.has_finished() {
        //println!("{}, {} ≤ {} < {}: {}", tm.state.name, tm.tape.min(), tm.head, tm.tape.max(), tm.tape.to_string());
        tm.run_step();
        steps += 1;
    }

    println!("Outpoot: {}", tm.get_tape_output());
    println!("Has finished? {} in {} steps!", tm.has_finished(), steps);
}
