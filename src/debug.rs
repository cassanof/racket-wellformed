use std::{io::BufRead, vec};

use racket_wellformed::sexpr_parser::SexprParser;

pub fn main() {
    // repl
    let mut buf = String::new();
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        if line.starts_with("!RUN") {
            let output = SexprParser::parse_all(&buf).unwrap_or_else(|e| {
                println!("Error: {}", e);
                vec![]
            });
            for expr in output {
                println!("{}", expr);
            }
            buf.clear();
        } else {
            buf.push_str(&line);
            buf.push('\n');
        }
    }
}
