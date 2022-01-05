//~ ## File parser

use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

const SPECIFICATION_INSTRUCTION: &str = "spec:";

/// Parse a file and return the specification-related content
pub fn parse_file(delimiter: &str, file_name: &str) -> String {
    match Path::new(file_name)
        .extension()
        .expect("cargo-specification can only parse files that have an extension")
        .to_str()
        .expect("couldn't convert the extension to a string")
    {
        "md" => {
            //~ - for markdown files, we retrieve the entire content
            std::fs::read_to_string(file_name).unwrap_or_else(|e| panic!("{}: {}", e, file_name))
        }
        _ => parse_code(delimiter, file_name),
    }
}

/// Parse code to return the specification-related content
/// (comments that start with a special delimiter, by default `~`)
pub fn parse_code(delimiter: &str, file_name: &str) -> String {
    // state
    let mut print_line = false; // indicates if we're between `//~ spec:startcode` and `//~spec:endcode` statements
    let mut result = String::new();

    // go over file line by line
    let mut previous_line_is_part_of_spec = None;
    let file = File::open(file_name).unwrap_or_else(|e| panic!("{}: {}", e, file_name));
    let lines = BufReader::new(file).lines();
    for line in lines {
        let line = line.unwrap();

        if !line.trim().starts_with(delimiter) {
            // only print a normal line if it is between `//~ spec:startcode` and `//~spec:endcode` statements
            // TODO: reset indentation
            if print_line {
                writeln!(&mut result, "{}", line).unwrap();
            }
            previous_line_is_part_of_spec = Some(false);
            continue;
        }

        // if the line starts with //~ parse it
        let comment = line.split_once(delimiter).unwrap().1.trim();
        if comment.starts_with(SPECIFICATION_INSTRUCTION) {
            // match on the instruction given in `//~ spec:instruction`
            match comment.split_once(SPECIFICATION_INSTRUCTION).unwrap().1 {
                // spec:startcode will print every line afterwards, up until a spec:endcode statement
                "startcode" if !print_line => {
                    writeln!(&mut result, "```rust").unwrap();
                    print_line = true;
                }
                "startcode" if print_line => panic!("cannot startcode when already started"),
                // spec:endcode ends spec:startcode
                "endcode" if print_line => {
                    writeln!(&mut result, "```").unwrap();
                    print_line = false;
                }
                "endcode" if !print_line => {
                    panic!("cannot endcode if haven't startcode before")
                }
                //
                _ => unimplemented!(),
            };
        } else {
            if let Some(false) = previous_line_is_part_of_spec {
                writeln!(&mut result, "\n").unwrap();
            }
            // if the specification comment is not an instruction, save it
            writeln!(&mut result, "{}", comment).unwrap();
            previous_line_is_part_of_spec = Some(true);
        }
    }

    // check state is consistent
    if print_line {
        panic!("a //~ spec:startcode was left open ended");
    }

    //
    result
}