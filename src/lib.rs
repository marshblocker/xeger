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
    match_indices: Vec<usize>,
}

#[derive(Debug, PartialEq)]
pub enum MatchResult {
    Found(Vec<LineMatch>),
    NotFound,
    Error(String),
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
        let mut match_indices: Vec<usize> = Vec::new();
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
                match_indices.push(indx_l - indx_p);
                indx_p = 0;
            }
        }

        if !match_indices.is_empty() {
            Some(LineMatch {
                line: line.to_string(),
                lineno,
                match_indices,
            })
        } else {
            None
        }
    }
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
                match_indices: vec![1]
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
                    match_indices: vec![0, 4]
                },
                LineMatch {
                    line: buf.split("\n").nth(1).unwrap().to_string(),
                    lineno: 2,
                    match_indices: vec![4]
                },
                LineMatch {
                    line: buf.split("\n").last().unwrap().to_string(),
                    lineno: 3,
                    match_indices: vec![2, 5, 8]
                }
            ])
        );
    }
}
