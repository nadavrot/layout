//! A collection of helper functions that are related to records. Records are
//! recursive data-structures that contain boxes and labels. This is where you
//! can find code for figuring out sizes and finding the location of a named
//! 'port'.

use crate::std_shapes::shapes::ShapeKind;
use crate::std_shapes::shapes::*;

pub fn print_record(rec: &RecordDef, indent: usize) {
    match rec {
        RecordDef::Text(label, port) => {
            println!("\"{}\"", label);
            if let Option::Some(port) = port {
                println!("\"{}\"", port);
            }
        }
        RecordDef::Array(arr) => {
            print!("{}", " ".repeat(indent));
            println!("[");
            for elem in arr {
                print_record(elem, indent + 1);
            }
            print!("{}", " ".repeat(indent));
            println!("]");
        }
    }
}

struct RecordParser {
    input: Vec<char>,
    pos: usize,
}

struct RecordParserFrame {
    label: String,
    arr: Vec<RecordDef>,
}

impl RecordParserFrame {
    pub fn new() -> Self {
        Self {
            label: String::new(),
            arr: Vec::new(),
        }
    }

    /// Split a label such as "<f0> XXX" into the port part "f0" and the text
    /// part "XXX".
    fn split_label_to_text_and_port(str: &str) -> (String, Option<String>) {
        let str = str.trim();
        if str.starts_with('<') {
            if let Option::Some(idx) = str.find('>') {
                let port = &str[1..idx];
                return (
                    str[idx + 1..].trim().to_string(),
                    Option::Some(port.to_string()),
                );
            }
        }
        (str.to_string(), Option::None)
    }

    pub fn finalize_label(&mut self) {
        if !self.label.trim().is_empty() {
            let ret = Self::split_label_to_text_and_port(&self.label);
            let text = RecordDef::Text(ret.0, ret.1);
            self.arr.push(text);
            self.label.clear();
        }
    }

    pub fn finalize_record(&mut self) -> RecordDef {
        self.finalize_label();
        match self.arr.len() {
            0 => RecordDef::Text(String::from(""), Option::None),
            _ => RecordDef::Array(self.arr.clone()),
        }
    }
}
impl RecordParser {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> RecordDef {
        let mut frame = RecordParserFrame::new();
        loop {
            // Read one char.
            let ch = self.input[self.pos] as char;

            match ch {
                '{' => {
                    self.pos += 1;
                    // Finalize the label.
                    frame.finalize_label();
                    // Parse the sub row:
                    let ret = self.parse();
                    frame.arr.push(ret);
                }
                '|' => {
                    // New record in the row.
                    self.pos += 1;
                    frame.finalize_label();
                }
                '}' => {
                    // Finish the row.
                    self.pos += 1;
                    // Finalize the row.
                    frame.finalize_label();
                    return frame.finalize_record();
                }
                _ => {
                    self.pos += 1;

                    // Handle regular chars. Add them to the current label.
                    frame.label.push(ch);
                }
            }
            // Are we at the end of the buffer?
            if self.pos == self.input.len() {
                return frame.finalize_record();
            }
        }
    }
}

pub fn parse_record_string(label: &str) -> RecordDef {
    RecordParser::new(label).parse()
}

// Construct a record from a description string.
pub fn record_builder(label: &str) -> ShapeKind {
    let res = parse_record_string(label);
    ShapeKind::Record(res)
}
