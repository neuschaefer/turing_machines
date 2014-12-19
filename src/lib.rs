#![feature(slicing_syntax)]

extern crate serialize;

use std::io::{BufferedReader, File};
use std::default::Default;

#[deriving(Show, Encodable)]
struct Transition {
    state: String,
    symbol: char,
    movement: Movement
}

impl Transition {
    fn from_str(s: &str) -> Transition {
        let v: Vec<_> = s.split(',').collect();
        assert_eq!(v.len(), 3);
        assert_eq!(v[1].char_len(), 1);
        assert_eq!(v[2].char_len(), 1);

        Transition {
            state: v[0].to_string(),
            symbol: v[1].char_at(0),
            movement: Movement::from_char(v[2].char_at(0))
        }
    }
}

#[deriving(Show, Encodable)]
enum Movement {
    Left,
    Right,
    None
}

impl Movement {
    fn from_char(c: char) -> Movement {
        match c {
            'L' => Movement::Left,
            'R' => Movement::Right,
            'N' => Movement::None,
            bad => panic!("`{}' is not a valid movement!", bad)
        }
    }

    fn to_delta(&self) -> int {
        match *self {
            Movement::Left => -1,
            Movement::None => 0,
            Movement::Right => 1
        }
    }
}


#[deriving(Show, Encodable)]
struct State {
    name: String,
    transitions: Vec<Option<Transition>>
}

impl State {
    fn new(name: String, words: &[&str]) -> State {
        let trans = words.iter().map(|&w| {
            match w {
                "-" => None,
                w => Some(Transition::from_str(w))
            }
        }).collect();

        State {
            name: name,
            transitions: trans
        }
    }
}

// file format:
// - comments and empty lines everywhere.
//
// input symbols        A                       B   C
// transition table     state,symbol,movement ...
#[deriving(Default, Show, Encodable)]
pub struct TMDesc {
    input_symbols: Vec<char>,
    states: Vec<State>,
}

impl TMDesc {
    pub fn new() -> TMDesc {
        Default::default()
    }

    pub fn parse_line(line: &str) -> Result<Vec<&str>, ()> {
        if line.starts_with("#") {
            // this line is a comment
            return Err(());
        }

        if line.is_empty() {
            return Err(());
        }

        let words: Vec<_> = line.split('\t').collect();

        Ok(words)
    }

    pub fn handle_line(&mut self, words: &[&str]) {
        if self.input_symbols.is_empty() {
            for word in words.iter().skip(1) {
                assert_eq!(word.char_len(), 1);
                self.input_symbols.push(word.char_at(0));
            }
        } else { // a new state
            let name = words[0];
            let state = State::new(name.to_string(),
                    words.slice_from(1));
            self.states.push(state);
        }
    }
}

pub struct Tape {
    left: Vec<char>,
    right: Vec<char>
}

impl Tape {
    pub fn from_str(s: &str) -> Tape {
        Tape {
            left: vec![],
            right: s.chars().collect()
        }
    }

    /// maximum index plus one. the name is slightly misleading.
    pub fn max(&self) -> int {
        self.right.len() as int
    }

    /// minimum index.
    fn min(&self) -> int {
        -(self.left.len() as int)
    }

    /// add blanks to ensure that the given index is valid.
    pub fn ensure_space(&mut self, i: int) {
        if i >= self.max() {
            let delta = i - self.max() + 1;
            self.right.grow(delta as uint, 'B')
        } else if i < self.min() {
            let delta = self.min() - i;
            self.left.grow(delta as uint, 'B')
        }
    }
}

impl ToString for Tape {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for &c in self.left.iter().rev().chain(self.right.iter()) {
            s.push(c)
        }
        s
    }
}

impl IndexMut<int, char> for Tape {
    fn index_mut<'a>(&'a mut self, i: &int) -> &'a mut char {
        if *i >= 0 {
            self.right.index_mut(&(*i as uint))
        } else {
            //  0 => right[0]
            // -1 => left[0]
            // -2 => left[1]
            self.left.index_mut(&((-*i - 1) as uint))
        }
    }
}

impl Index<int, char> for Tape {
    fn index<'a>(&'a self, i: &int) -> &'a char {
        if *i >= 0 {
            self.right.index(&(*i as uint))
        } else {
            self.left.index(&((-*i - 1) as uint))
        }
    }
}

/// a runnable turing machine instance
pub struct TM<'a> {
    desc: &'a TMDesc,
    head: int,
    tape: Tape,
    state: &'a State
}

impl<'a> TM<'a> {
    pub fn new<'a>(desc: &'a TMDesc, input: &str) -> TM<'a> {
        TM {
            desc: desc,
            head: 0,
            tape: Tape::from_str(input),
            state: &desc.states[0]
        }
    }

    pub fn run_step(&mut self) {
        if self.has_finished() {
            return;
        }

        let cur_sym = self.tape[self.head];
        let input_index = match self.desc.input_symbols.iter()
        .enumerate().find(|&(_i, &sym)| sym == cur_sym) {
            Some((index, _)) => index,
            None => panic!("Input character \'{}\' was not found", cur_sym)
        };

        let trans = match self.state.transitions[input_index] {
            Some(ref trans) => trans,
            None => panic!("No transition for {} on \'{}\'", self.state.name, input_index)
        };

        let next_state = match self.desc.states.iter().find(
            |s| s.name == trans.state
        ) {
            Some(s) => s,
            None => panic!("state \"{}\" not found", trans.state)
        };

        self.state = next_state;
        self.tape[self.head] = trans.symbol;
        self.head += trans.movement.to_delta();
        self.tape.ensure_space(self.head);
    }

    pub fn has_finished(&self) -> bool {
        self.state.name[] == "STOPP"
    }

    pub fn get_tape_output(&self) -> String {
        let mut s = String::new();
        for i in range(self.head, self.tape.max()) {
            s.push(self.tape[i]);
        }

        s
    }
}
