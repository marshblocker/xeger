use std::fs;

/// Currently supports concatenation.
/// TODO: Implement alternation.
pub struct RegExpr {
    pattern: Vec<char>,
}

#[derive(PartialEq, Debug)]
pub struct LineMatch {
    line: String,
    lineno: u32,
    matches: Vec<MatchInstance>,
}

/// [start_indx, end_indx]
#[derive(PartialEq, Debug)]
pub struct MatchInstance {
    start_indx: usize,
    end_indx: usize,
}

#[derive(Debug, PartialEq)]
pub enum MatchResult {
    Found(Vec<LineMatch>),
    NotFound,
    Error(String),
}

impl MatchInstance {
    pub fn new(start_indx: usize, end_indx: usize) -> Self {
        Self {
            start_indx,
            end_indx,
        }
    }
}

impl RegExpr {
    pub fn new(pattern: &str) -> Self {
        let pattern = pattern.chars().collect::<Vec<char>>();
        // let options = pattern
        //     .split("|")
        //     .map(|s| s.to_string())
        //     .collect::<Vec<String>>();

        Self { pattern }
    }

    pub fn match_buf(&self, buf: &str) -> MatchResult {
        let mut matched_lines: Vec<LineMatch> = Vec::new();
        let lines = buf.lines().map(|l| l.to_string()).collect::<Vec<String>>();

        for (i, line) in lines.into_iter().enumerate() {
            let lineno = (i + 1).try_into().unwrap();

            if let Some(matched_line) = self.match_line(&line, lineno) {
                matched_lines.push(matched_line);
            }
        }

        if !matched_lines.is_empty() {
            MatchResult::Found(matched_lines)
        } else {
            MatchResult::NotFound
        }
    }

    pub fn match_file(&self, path: &str) -> MatchResult {
        let mut matched_lines: Vec<LineMatch> = Vec::new();
        let buf = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => return MatchResult::Error(e.to_string()),
        };
        let lines = buf.lines().map(|l| l.to_string()).collect::<Vec<String>>();

        for (i, line) in lines.into_iter().enumerate() {
            let lineno = (i + 1).try_into().unwrap();

            if let Some(matched_line) = self.match_line(&line, lineno) {
                matched_lines.push(matched_line);
            }
        }

        if !matched_lines.is_empty() {
            MatchResult::Found(matched_lines)
        } else {
            MatchResult::NotFound
        }
    }

    fn match_line(&self, line: &str, lineno: u32) -> Option<LineMatch> {
        let mut matches: Vec<MatchInstance> = Vec::new();
        let line_chars = line.chars().collect::<Vec<char>>();

        let mut indx_l: usize = 0; // line index
        let mut indx_p: usize = 0; // pattern index

        while indx_l < line.len() {
            if line_chars[indx_l] == self.pattern[indx_p] {
                indx_l += 1;
                indx_p += 1;
            } else {
                // Restart matching but move one character to the
                // right from the starting character of the previous
                // (failed) match.
                indx_l -= indx_p;
                indx_l += 1;
                indx_p = 0;
            }

            if indx_p >= self.pattern.len() {
                // Successfully matched pattern to a substring of
                // the line. Take note of the matched word starting index and continue
                // matching.
                let match_start = indx_l - indx_p;
                matches.push(MatchInstance::new(
                    match_start,
                    match_start + self.pattern.len() - 1,
                ));
                indx_p = 0;
            }
        }

        if !matches.is_empty() {
            Some(LineMatch {
                line: line.to_string(),
                lineno,
                matches,
            })
        } else {
            None
        }
    }
}

pub fn print_matched_lines(matched_lines: &Vec<LineMatch>) {
    let matched_lines_display = get_matched_lines_display(matched_lines);
    matched_lines_display
        .into_iter()
        .for_each(|s| println!("{}", s));
}

fn get_matched_lines_display(matched_lines: &Vec<LineMatch>) -> Vec<String> {
    let mut matched_lines_display: Vec<String> = Vec::new();

    for matched_line in matched_lines {
        let mut matched_line_display: Vec<String> = Vec::new();

        let mut start: usize;
        let mut end = 0;
        let mut prev_end: usize;
        for i in 0..matched_line.matches.len() {
            prev_end = if i == 0 { 0 } else { end };

            start = matched_line.matches[i].start_indx;
            end = matched_line.matches[i].end_indx;

            if i == 0 {
                matched_line_display.push(format!("{}", &matched_line.line[prev_end..start]));
                matched_line_display.push(format!("<{}>", &matched_line.line[start..=end]));
            } else {
                matched_line_display.push(format!("{}", &matched_line.line[prev_end + 1..start]));
                matched_line_display.push(format!("<{}>", &matched_line.line[start..=end]));
            }
        }

        matched_line_display.push(format!("{}", &matched_line.line[end + 1..]));
        matched_lines_display.push(matched_line_display.join(""));
    }

    matched_lines_display
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_empty_buf() {
        let buf = "";
        let pattern = "foo";

        let re = RegExpr::new(pattern);
        assert_eq!(re.match_buf(buf), MatchResult::NotFound);
    }

    #[test]
    fn match_one() {
        let buf = "ffoof";
        let pattern = "foo";

        let re = RegExpr::new(pattern);
        assert_eq!(
            re.match_buf(buf),
            MatchResult::Found(vec![LineMatch {
                line: buf.to_string(),
                lineno: 1,
                matches: vec![MatchInstance::new(1, 3)]
            }])
        );
    }

    #[test]
    fn match_many_lines() {
        let buf = "fooffoo\nabfofooF\nngfoofoofooff";
        let pattern = "foo";

        let re = RegExpr::new(pattern);
        assert_eq!(
            re.match_buf(buf),
            MatchResult::Found(vec![
                LineMatch {
                    line: buf.split("\n").next().unwrap().to_string(),
                    lineno: 1,
                    matches: vec![MatchInstance::new(0, 2), MatchInstance::new(4, 6)]
                },
                LineMatch {
                    line: buf.split("\n").nth(1).unwrap().to_string(),
                    lineno: 2,
                    matches: vec![MatchInstance::new(4, 6)]
                },
                LineMatch {
                    line: buf.split("\n").last().unwrap().to_string(),
                    lineno: 3,
                    matches: vec![
                        MatchInstance::new(2, 4),
                        MatchInstance::new(5, 7),
                        MatchInstance::new(8, 10)
                    ]
                }
            ])
        );
    }

    #[test]
    fn get_matched_lines_display_one_line() {
        let buf = "ffoof";
        let pattern = "foo";

        let re = RegExpr::new(pattern);
        let res = re.match_buf(buf);
        if let MatchResult::Found(matched_lines) = res {
            assert_eq!(get_matched_lines_display(&matched_lines), vec!["f<foo>f"]);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn get_matched_lines_display_one_line_many() {
        let buf = "fooffoofofoo";
        let pattern = "foo";

        let re = RegExpr::new(pattern);
        let res = re.match_buf(buf);
        if let MatchResult::Found(matched_lines) = res {
            assert_eq!(
                get_matched_lines_display(&matched_lines),
                vec!["<foo>f<foo>fo<foo>"]
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn get_matched_lines_display_many_lines_match_some_only() {
        let buf = "bar\nfoo";
        let pattern = "foo";

        let re = RegExpr::new(pattern);
        let res = re.match_buf(buf);
        if let MatchResult::Found(matched_lines) = res {
            assert_eq!(get_matched_lines_display(&matched_lines), vec!["<foo>"]);
        } else {
            assert!(false);
        }
    }
}
