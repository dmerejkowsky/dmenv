use crate::error::*;

use std::collections::HashMap;

const COMMENT: [char; 2] = ['#', ';'];
const SEPARATOR: [char; 2] = ['=', ':'];

#[derive(Debug)]
struct Section {
    name: String,
    contents: HashMap<String, Vec<String>>,
}

impl Section {
    fn new(name: &str) -> Self {
        Section {
            name: name.to_owned(),
            contents: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct SetupCfg {
    sections: HashMap<String, Section>,
}

impl SetupCfg {
    fn new() -> Self {
        SetupCfg {
            sections: HashMap::new(),
        }
    }

    pub fn read(path: &std::path::Path) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(|e| new_read_error(e, path))?;
        let res = parse(&contents);
        res.map_err(|e| Error::MalformedSetupCfg {
            path: path.to_path_buf(),
            message: format!("{}:{}: {}", path.display(), e.lineno, e.message),
        })
    }

    pub fn project_name(&self) -> Result<String, GetterError> {
        self.get_single("metadata", "name")
    }

    pub fn dependencies(&self) -> Result<Vec<String>, GetterError> {
        self.get_multi("options", "install_requires")
    }

    pub fn dev_dependencies(&self) -> Result<Vec<String>, GetterError> {
        self.get_multi("options.extras_require", "dev")
    }

    pub fn prod_dependencies(&self) -> Result<Vec<String>, GetterError> {
        self.get_multi("options.extras_require", "prod")
    }

    pub fn get_single(&self, section: &str, key: &str) -> Result<String, GetterError> {
        let values = self.get_multi(section, key)?;
        if values.len() > 1 {
            return Err(GetterError::MultipleValues {
                key: key.to_string(),
            });
        }
        if values.is_empty() {
            return Err(GetterError::EmptyValue {
                key: key.to_string(),
            });
        }
        Ok(values[0].to_string())
    }

    pub fn get_multi(&self, section: &str, key: &str) -> Result<Vec<String>, GetterError> {
        if !&self.sections.contains_key(section) {
            return Err(GetterError::SectionNotFound {
                name: section.to_string(),
            });
        }
        let section = &self.sections[section];
        let section_contents = &section.contents;
        if !section_contents.contains_key(key) {
            return Err(GetterError::KeyNotFound {
                name: key.to_string(),
            });
        }
        let values = &section_contents[key];
        Ok(values.to_vec())
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    lineno: usize,
    message: String,
}

impl ParseError {
    fn new(lineno: usize, message: &str) -> Self {
        ParseError {
            lineno,
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "line {}: {}", self.lineno, self.message)
    }
}

#[derive(Debug, PartialEq)]
pub enum GetterError {
    EmptyValue { key: String },
    MultipleValues { key: String },
    SectionNotFound { name: String },
    KeyNotFound { name: String },
}

impl std::fmt::Display for GetterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let message = match self {
            GetterError::MultipleValues { key } => {
                format!("Multiple values found for key '{}'", key)
            }
            GetterError::EmptyValue { key } => format!("No value found for key '{}'", key),
            GetterError::SectionNotFound { name } => {
                format!("No section found with name '{}'", name)
            }
            GetterError::KeyNotFound { name } => format!("No key found with name '{}'", name),
        };
        write!(f, "{}", message)
    }
}

fn parse(contents: &str) -> Result<SetupCfg, ParseError> {
    let mut parser = Parser::new();
    parser.parse(contents)?;
    Ok(parser.get())
}

#[derive(Debug, Copy, Clone)]
enum ParseState {
    Start,
    Key,
    Value,
    KeyValue,
    Section,
}

#[derive(Debug)]
struct Parser {
    key: String,
    value: String,
    section: String,
    state: ParseState,
    res: SetupCfg,
}

impl Parser {
    // Parsing is done line by line, in two steps:
    // First, in self.parse(), we store the parsing state and the potential values
    // for section, key, value in self.
    //
    // Then we call self.advance() to update hashmaps in self.sections
    // depending on the state and the values stored in the previous step
    //
    // We use debug_assert! to make sure code is consistent between two methods - for instance,
    // the parser should reject a value without key, meaning it's safe to assume that when
    // self.value is set, self.key exists and matches an existing key in the current section
    // contents map.
    //
    fn new() -> Self {
        Parser {
            key: String::new(),
            value: String::new(),
            section: String::new(),
            state: ParseState::Start,
            res: SetupCfg::new(),
        }
    }

    fn parse(&mut self, text: &str) -> Result<(), ParseError> {
        let lines_it = text.lines().enumerate();
        for (i, line) in lines_it {
            let lineno = i + 1;
            if is_comment(line) {
                continue;
            }
            if is_blank(line) {
                continue;
            }
            check_section(lineno, line)?;
            if let Some(section) = is_section(line) {
                self.section = section;
                // Reset current key as well
                self.key = "".to_string();
                self.state = ParseState::Section;
            } else if let Some((key, value)) = is_key_value(line) {
                self.key = key;
                self.value = value;
                self.state = ParseState::KeyValue;
            } else if let Some(key) = is_key(line) {
                self.key = key;
                // Reset current value as well
                self.value = "".to_string();
                self.state = ParseState::Key;
            } else {
                check_value(lineno, line)?;
                self.value = line.trim().to_string();
                self.state = ParseState::Value;
            }

            self.advance(lineno)?;
        }
        Ok(())
    }

    fn advance(&mut self, lineno: usize) -> Result<(), ParseError> {
        match self.state {
            ParseState::Section => {
                let new_section = Section::new(&self.section);
                self.res.sections.insert(self.section.clone(), new_section);
            }

            ParseState::KeyValue => {
                if self.section == "" {
                    return Err(ParseError::new(lineno, "key outside section"));
                }
                let section = self.res.sections.get_mut(&self.section);
                debug_assert!(
                    section.is_some(),
                    "{}: should have found section named '{}' when parsing key_value: '{}'='{}'",
                    lineno,
                    self.section,
                    self.key,
                    self.value
                );
                let contents = &mut section.unwrap().contents;
                contents.insert(self.key.clone(), vec![self.value.clone()]);
            }

            ParseState::Key => {
                if self.section == "" {
                    return Err(ParseError::new(lineno, "key outside section"));
                }
                let section = self.res.sections.get_mut(&self.section);
                debug_assert!(
                    section.is_some(),
                    "{}: should have found section named '{}' when parsing key: '{}'",
                    lineno,
                    self.section,
                    self.key
                );
                let contents = &mut section.unwrap().contents;
                contents.insert(self.key.clone(), vec![]);
            }

            ParseState::Value => {
                if self.key == "" {
                    return Err(ParseError::new(lineno, "value without key"));
                }
                let section = self.res.sections.get_mut(&self.section);
                debug_assert!(
                    section.is_some(),
                    "{}: should have found section named '{}' when parsing value: '{}'",
                    lineno,
                    self.section,
                    self.value
                );
                let contents = &mut section.unwrap().contents;
                let values = contents.get_mut(&self.key);
                debug_assert!(
                    values.is_some(),
                    "{}: should have found a key named '{}' when parsing value: '{}'",
                    lineno,
                    self.key,
                    self.value
                );
                values.unwrap().push(self.value.clone());
            }

            _ => {}
        }
        Ok(())
    }

    fn get(self) -> SetupCfg {
        self.res
    }
}

fn is_comment(line: &str) -> bool {
    let line = line.trim_start().to_string();
    for c in &COMMENT {
        if line.starts_with(&c.to_string()) {
            return true;
        }
    }
    false
}

fn is_blank(line: &str) -> bool {
    line.trim() == ""
}

fn check_section(lineno: usize, line: &str) -> Result<(), ParseError> {
    if line.starts_with('[') && !line.ends_with(']') {
        return Err(ParseError::new(lineno, "missing closing bracket"));
    }
    if line.ends_with(']') && !line.starts_with('[') {
        return Err(ParseError::new(lineno, "missing opening bracket"));
    }
    if line == "[]" {
        return Err(ParseError::new(lineno, "empty section"));
    }
    Ok(())
}

fn check_value(lineno: usize, line: &str) -> Result<(), ParseError> {
    if !is_indented(line) {
        return Err(ParseError::new(lineno, "expected indented value"));
    }
    Ok(())
}

fn is_section(line: &str) -> Option<String> {
    if line.starts_with('[') && line.ends_with(']') {
        return Some(line[1..line.len() - 1].to_string());
    }
    None
}

fn is_indented(line: &str) -> bool {
    line.starts_with(' ') || line.starts_with('\t')
}

fn is_key_value(line: &str) -> Option<(String, String)> {
    let n = line.len();
    if is_indented(line) {
        return None;
    }
    let sep_pos = line.find(is_separator)?;
    if sep_pos == n - 1 {
        return None;
    }

    let v_start_pos = sep_pos + 1;
    let mut chars_it = line.chars().skip(v_start_pos);

    // Advance until finding a comment or end of line
    let maybe_end = chars_it.position(|c: char| COMMENT.contains(&c));
    let v_end_pos = match maybe_end {
        None => n,
        Some(pos) => v_start_pos + pos,
    };

    let value = &line[v_start_pos..v_end_pos].trim();

    let key = &line[0..sep_pos].trim();
    Some((key.to_string(), value.to_string()))
}

fn is_separator(c: char) -> bool {
    SEPARATOR.contains(&c)
}

fn is_key(line: &str) -> Option<String> {
    if is_indented(line) {
        return None;
    }
    let maybe_key = line.trim();
    for c in &SEPARATOR {
        if maybe_key.ends_with(&c.to_string()) {
            let n = maybe_key.len();
            let key = &maybe_key[0..n - 1];
            return Some(key.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_comment() {
        assert!(is_comment("; foo"));
        assert!(is_comment("   # inddented comment "));
    }

    #[test]
    fn test_is_key() {
        assert_eq!(is_key("foo ="), Some("foo".to_string()));
        assert_eq!(is_key("foo:"), Some("foo".to_string()));
    }

    #[test]
    fn test_is_key_value_equal_sep() {
        assert_eq!(
            is_key_value("foo = bar"),
            Some(("foo".to_string(), "bar".to_string()))
        );
    }

    #[test]
    fn test_is_key_value_colon_sep() {
        assert_eq!(
            is_key_value("foo: bar"),
            Some(("foo".to_string(), "bar".to_string()))
        );
    }

    #[test]
    fn test_is_key_value_trailing_comment() {
        assert_eq!(
            is_key_value("foo=bar  # comment"),
            Some(("foo".to_string(), "bar".to_string()))
        );
    }

    fn assert_parse<F>(text: &str, matcher: F)
    where
        F: Fn(&SetupCfg) -> bool,
    {
        let parsed = parse(text);
        let parsed = parsed.unwrap_or_else(|e| panic!("\nCould not parse: \n{}\n: {}", text, e));
        let outcome = matcher(&parsed);
        if !outcome {
            panic!(format!("parse(\n{}\n)={:#?}", text, &parsed));
        }
    }

    #[test]
    fn test_one_section() {
        let text = "[foo]";
        assert_parse(text, |x| x.sections.contains_key("foo"));
    }

    #[test]
    fn test_two_sections() {
        let text = "[s1]\nbar=42\n[s2]\n";
        assert_parse(text, |x| x.sections.contains_key("s1"));
        assert_parse(text, |x| x.sections.contains_key("s2"));
    }

    #[test]
    fn test_one_key_value() {
        let text = "[foo]\nbar=42";
        assert_parse(text, |x| x.get_single("foo", "bar").unwrap() == "42");
    }

    #[test]
    fn test_blank_lines() {
        let text = "[s1]\nbar=42\n\n[s2]";
        assert_parse(text, |x| x.get_single("s1", "bar").unwrap() == "42");
    }

    #[test]
    fn test_two_key_values() {
        let text = "[foo]\nbar=42\nbaz=true";
        assert_parse(text, |x| x.get_single("foo", "bar").unwrap() == "42");
        assert_parse(text, |x| x.get_single("foo", "baz").unwrap() == "true");
    }

    #[test]
    fn test_one_key_one_value() {
        let text = "[foo]\nbar=\n  42";
        assert_parse(text, |x| x.get_single("foo", "bar").unwrap() == "42");
    }

    #[test]
    fn test_one_key_two_values() {
        let text = "[foo]\nbar=\n  one\n  two";
        assert_parse(text, |x| {
            x.get_multi("foo", "bar").unwrap() == ["one", "two"]
        });
    }

    #[test]
    fn test_one_key_empty_value() {
        let text = "[foo]\nbar=\n";
        assert_parse(text, |x| x.get_multi("foo", "bar").unwrap().is_empty())
    }

    fn assert_parse_error(text: &str, lineno: usize) {
        let parsed = parse(text);
        let err = parsed.unwrap_err();
        assert_eq!(err.lineno, lineno);
        print!("{}", err);
    }

    #[test]
    fn test_one_key_no_value() {
        let text = "[foo]\nbar";
        assert_parse_error(text, 2);
    }

    #[test]
    fn test_key_value_outside_section() {
        let text = "bar = 42";
        assert_parse_error(text, 1);
    }

    #[test]
    fn test_key_outside_section() {
        let text = "bar=";
        assert_parse_error(text, 1);
    }

    #[test]
    fn test_missing_closing_bracket() {
        let text = "[foo]\nspam=42\n[bad\n";
        assert_parse_error(text, 3);
    }

    #[test]
    fn test_missing_opening_bracket() {
        let text = "[foo]\nspam=42\nbad]\n";
        assert_parse_error(text, 3);
    }

    #[test]
    fn test_empty_section() {
        let text = "[foo]\nspam=42\n[]eggs=true\n";
        assert_parse_error(text, 3);
    }

    #[test]
    fn test_non_indented_value() {
        let text = "\
[options.extras_require]
dev =
  pytest
foo
";
        assert_parse_error(text, 4);
    }

    fn assert_get_error<F, R>(text: &str, get_func: F, expected_error: GetterError)
    where
        F: Fn(&SetupCfg) -> Result<R, GetterError>,
        R: std::fmt::Debug,
    {
        let parsed = parse(text);
        let parsed = parsed.unwrap_or_else(|e| panic!("\nCould not parse: \n{}\n: {}", text, e));
        let outcome = get_func(&parsed);
        let actual_error = outcome.unwrap_err();
        assert_eq!(actual_error, expected_error)
    }

    #[test]
    fn test_get_single_on_empty_value() {
        let text = "[foo]\nbar=\n";
        assert_get_error(
            text,
            |x| x.get_single("foo", "bar"),
            GetterError::EmptyValue {
                key: "bar".to_string(),
            },
        );
    }

    #[test]
    fn test_get_single_on_multi_value() {
        let text = "[foo]\nmy_list = \n  one \n two";
        assert_get_error(
            text,
            |x| x.get_single("foo", "my_list"),
            GetterError::MultipleValues {
                key: "my_list".to_string(),
            },
        );
    }

    fn does_not_crash_when_parsing(text: &str) {
        let _res = parse(text);
    }

    #[test]
    fn test_cpython_test1() {
        let cfg1 = include_str!("../tests/setup_cfg/cfgparser.1");
        does_not_crash_when_parsing(cfg1);
    }

    // Note: neither cfgparser.2 an cfgparser.3 generates errors when parsed by dmenv.
    // We are more strict than the upstream's configparser Python library
    // This is OK.
    //
    // We want to avoid these two cases:
    //   - dmenv crashes when parsing weird syntax
    //   - demnv accept configs that are *rejected* by setuptools
    //
    // Accepting everything setuptools accepts would be nice, but frankly the world
    // will be a better place if people get weird of weird syntaxes in their config
    // files :P
    #[test]
    fn test_cpython_test2() {
        let cfg2 = include_str!("../tests/setup_cfg/cfgparser.2");
        does_not_crash_when_parsing(cfg2);
    }

    #[test]
    fn test_cpython_test3() {
        let cfg3 = include_str!("../tests/setup_cfg/cfgparser.3");
        does_not_crash_when_parsing(cfg3);
    }
}
