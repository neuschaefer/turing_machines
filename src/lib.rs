#![feature(collections)]

extern crate rustc_serialize;

mod errors;

use std::default::Default;
use std::ops::{Index, IndexMut, Deref};
use std::path::Path;
use std::fs::File;
use std::io::Read;

pub use errors::TMDescError;

#[derive(Debug, Clone, RustcEncodable)]
pub struct Transition {
    pub state: String,
    pub state_index: Option<usize>,
    pub symbol: char,
    pub movement: Movement
}

impl Transition {
    fn from_str(s: &str) -> Transition {
        let v: Vec<_> = s.split(',').collect();
        assert_eq!(v.len(), 3);
        assert_eq!(v[1].chars().count(), 1);
        assert_eq!(v[2].chars().count(), 1);

        Transition {
            state: v[0].into(),
            state_index: None,
            symbol: v[1].chars().next().unwrap(),
            movement: Movement::from_char(v[2].chars().next().unwrap())
        }
    }
}

#[derive(Debug, Clone, RustcEncodable)]
pub enum Movement {
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

    pub fn to_delta(&self) -> isize {
        match *self {
            Movement::Left => -1,
            Movement::None => 0,
            Movement::Right => 1
        }
    }
}


#[derive(Debug, Clone, RustcEncodable)]
pub struct State {
    pub name: String,
    pub transitions: Vec<Option<Transition>>
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

    pub fn is_final(&self) -> bool {
        self.name == "STOPP"
    }
}

// file format:
// - comments and empty lines everywhere.
//
// input symbols        A                       B   C
// transition table     state,symbol,movement ...
#[derive(Default, Debug, Clone, RustcEncodable)]
pub struct TMDesc {
    pub input_symbols: Vec<char>,
    pub states: Vec<State>,
}

impl TMDesc {
    pub fn new() -> TMDesc {
        Default::default()
    }

    pub fn parse_line(line: &str) -> Option<Vec<&str>> {
        // Comments and empty lines are ignored
        if line.starts_with("#") || line.is_empty() {
            return None;
        }

        Some(line.split('\t').collect())
    }

    pub fn handle_line(&mut self, words: &[&str]) {
        if self.input_symbols.is_empty() {
            for word in words.iter().skip(1) {
                assert_eq!(word.chars().count(), 1);
                self.input_symbols.push(word.chars().next().unwrap())
            }
        } else { // a new state
            let name = words[0];
            let state = State::new(name.into(), &words[1..]);
            self.states.push(state);
        }
    }

    pub fn resolve_state_index(&self, trans: &Transition) -> usize {
        if let Some(index) = trans.state_index {
            index
        } else {
            self.states.iter().enumerate()
                .find(|&(_, s)| s.name == trans.state)
                .map(|(i, _)| i)
                .unwrap_or_else(||
                        panic!("state \"{}\" not found", trans.state))
        }
    }

    /// Resolve all state names in transitions to simple index into the state
    /// tale, for faster lookup.
    pub fn resolve_all_state_indices(&mut self) {
        // This is a quick and dirty workaround for the borrow checker.
        // calling self.resolve_state_index would borrow self immutably while
        // it's already mutably borrowed. This wouldn't be a big deal in our
        // case, but the borrow checker doesn't allow it.
        let cloned: TMDesc = self.clone();

        for state in self.states.iter_mut() {
            for trans in state.transitions.iter_mut() {
                if let &mut Some(ref mut trans) = trans {
                    let index = cloned.resolve_state_index(trans);
                    trans.state_index = Some(index);
                }
            }
        }
    }

    pub fn blank_symbol(&self) -> char {
        *self.input_symbols.last().unwrap()
    }

    pub fn from_file(path: &Path) -> Result<TMDesc, TMDescError> {
        let mut file = try!(File::open(path));
        let mut string = String::new();

        try!(file.read_to_string(&mut string));
        Ok(Self::from_string(&string))
    }

    pub fn from_string(string: &str) -> TMDesc {
        let lines = string.lines().filter_map(Self::parse_line).collect::<Vec<_>>();
        Self::from_lines(lines)
    }

    pub fn from_lines<'a, I, S>(lines: I) -> TMDesc where I: IntoIterator<Item=S>, S: Deref<Target=[&'a str]> {
        let mut desc = Self::new();
        for line in lines {
            desc.handle_line(&*line)
        }
        desc
    }
}

#[derive(Debug)]
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
    pub fn max(&self) -> isize {
        self.right.len() as isize
    }

    /// minimum index.
    fn min(&self) -> isize {
        -(self.left.len() as isize)
    }

    /// add blanks to ensure that the given index is valid.
    pub fn ensure_space(&mut self, i: isize) {
        if i >= self.max() {
            self.right.resize(i as usize + 1, 'B')
        } else if i < self.min() {
            self.left.resize(-i as usize, 'B')
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

impl IndexMut<isize> for Tape {
    fn index_mut<'a>(&'a mut self, i: isize) -> &'a mut char {
        if i >= 0 {
            &mut self.right[i as usize]
        } else {
            //  0 => right[0]
            // -1 => left[0]
            // -2 => left[1]
            &mut self.left[(-i - 1) as usize]
        }
    }
}

impl Index<isize> for Tape {
    type Output = char;

    fn index<'a>(&'a self, i: isize) -> &'a char {
        if i >= 0 {
            &self.right[i as usize]
        } else {
            &self.left[(-i - 1) as usize]
        }
    }
}

/// a runnable turing machine instance
pub struct TM<'a> {
    desc: &'a TMDesc,
    head: isize,
    tape: Tape,
    state: &'a State
}

impl<'a> TM<'a> {
    pub fn new(desc: &'a TMDesc, input: &str) -> TM<'a> {
        TM {
            desc: desc,
            head: 0,
            tape: {
                let mut tape = Tape::from_str(input);
                tape.ensure_space(0);
                tape
            },
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

        let state_index = self.desc.resolve_state_index(trans);

        self.state = &self.desc.states[state_index];
        self.tape[self.head] = trans.symbol;
        self.head += trans.movement.to_delta();
        self.tape.ensure_space(self.head);
    }

    pub fn has_finished(&self) -> bool {
        self.state.is_final()
    }

    pub fn get_tape_output(&self) -> String {
        let mut s = String::new();
        for i in self.head..self.tape.max() {
            s.push(self.tape[i]);
        }

        s
    }
}
