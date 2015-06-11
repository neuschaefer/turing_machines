#![feature(io)]

extern crate turing;

use turing::{TMDesc, TM};
use std::io::{Read, BufRead, BufReader};
use std::fs::File;
use std::path::Path;

fn main() {
    let file = File::open(&Path::new("../jn.tm"))
        .unwrap_or_else(|e| panic!("can't open file: {}", e));
    let reader = BufReader::new(file);

    let mut tmd = TMDesc::new();

    for line_res in reader.lines() {
        if let Ok(line) = line_res {
            if let Ok(words) = TMDesc::parse_line(line.trim_right()) {
                tmd.handle_line(&words);
            }
        }
    }

    tmd.resolve_all_state_indices();

    //println!("The TM is now configured as follows:");
    //println!("");
    //println!("{}", json::encode(tm));
    //println!("{}", tmd);

    let input_file = File::open(&Path::new("../big-troll.txt"))
        .unwrap_or_else(|e| panic!("can't open input: {}", e));
    let input_reader = BufReader::new(input_file);

    let filtered: String =
        input_reader.chars().map(|c| c.unwrap())
        .filter(|ch| !ch.is_whitespace()).collect();
    let mut tm = TM::new(&tmd, &filtered);

    let mut steps: u64 = 0;
    while !tm.has_finished() {
        //println!("{}, {} ≤ {} < {}: {}", tm.state.name, tm.tape.min(), tm.head, tm.tape.max(), tm.tape.to_string());
        tm.run_step();
        steps += 1;
    }

    println!("Outpoot: {}", tm.get_tape_output());
    println!("Has finished? {} in {} steps!", tm.has_finished(), steps);
}
