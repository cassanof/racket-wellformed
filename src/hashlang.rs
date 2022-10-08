/// Where the buffer is newline terminated
pub fn strip_hashlang(buf: &mut String) -> Option<String> {
    let mut lines: Vec<&str> = buf.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with('#') {
            let res = parse_hashlang(line);
            if res.is_some() {
                lines.remove(i);
                *buf = lines.join("\n");
                return res;
            }
        }
    }
    None
}

fn parse_hashlang(line: &str) -> Option<String> {
    if line.starts_with("#lang") {
        let stripped = line.strip_prefix("#lang")?.trim();
        Some(stripped.to_string())
    } else if line.starts_with("#reader") {
        let stripped = line.strip_prefix("#reader(lib ")?;
        let idx_end_lang = stripped[1..].find('\"')? + 1;
        Some(stripped[0..idx_end_lang].trim_matches('\"').to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod hashlang_tests {
    use super::{parse_hashlang, strip_hashlang};

    #[test]
    fn parse_reader_test() {
        let res = parse_hashlang("#reader(lib \"htdp-beginner-reader.ss\" \"lang\")((modname p2-code) (read-case-sensitive #t) (teachpacks ()) (htdp-settings #(#t constructor repeating-decimal #f #t none #f () #f)))");
        assert_eq!(res, Some("htdp-beginner-reader.ss".to_string()))
    }
    #[test]
    fn parse_hashlang_test() {
        let res = parse_hashlang("#lang htdp/bsl");
        assert_eq!(res, Some("htdp/bsl".to_string()))
    }

    #[test]
    fn parse_file_hashlang() {
        let mut file = r#"#lang racket/full

(+ 1 2 3)
; bla #bla #lang bla
(define (blah x)
    x)
"#
        .to_string();

        let res = strip_hashlang(&mut file);

        assert_eq!(file.split('\n').nth(0).unwrap(), "");

        assert_eq!(res, Some("racket/full".to_string()));
    }

    #[test]
    fn parse_file_reader() {
        let mut file = r#";; The first three lines of this file were inserted by DrRacket. They record metadata
;; about the language level of this file in a form that our tools can easily process.
#reader(lib "htdp-intermediate-lambda-reader.ss" "lang")((modname baaaaaa) (read-case-sensitive #t) (teachpacks ()) (htdp-settings #(#t constructor repeating-decimal #f #t none #f () #f)))


(+ 1 2 3)
; bla #bla #lang bla
(define (blah x)
    x)
"#
        .to_string();

        let res = strip_hashlang(&mut file);

        assert_eq!(file.split('\n').nth(2).unwrap(), "");

        assert_eq!(res, Some("htdp-intermediate-lambda-reader.ss".to_string()));
    }
}
