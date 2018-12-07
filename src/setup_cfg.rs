use crate::error::Error;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SetupCfg {
    sections: HashMap<String, Section>,
}

#[derive(Debug)]
struct Section {
    name: String,
    contents: HashMap<String, Vec<String>>,
}

impl Section {
    pub fn new(name: &str) -> Self {
        Section {
            name: name.to_owned(),
            contents: HashMap::new(),
        }
    }
}

impl SetupCfg {
    fn new() -> Self {
        SetupCfg {
            sections: HashMap::new(),
        }
    }

    fn from_str(contents: &str) -> Result<Self, Error> {
        parse(contents)
    }

    pub fn from_path(path: &std::path::Path) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(|e| Error::ReadError {
            path: path.to_path_buf(),
            io_error: e,
        })?;
        let res = Self::from_str(&contents)?;
        Ok(res)
    }

    pub fn get_single(&self, section: &str, key: &str) -> Result<String, Error> {
        let values = self.get_multi(section, key)?;
        if values.len() > 1 {
            return Err(Error::MultipleValues {
                key: key.to_string(),
            });
        }
        Ok(values[0].to_string())
    }

    pub fn get_multi(&self, section: &str, key: &str) -> Result<Vec<String>, Error> {
        if !&self.sections.contains_key(section) {
            return Err(Error::SectionNotFound {
                name: section.to_string(),
            });
        }
        let section = &self.sections[section];
        let section_contents = &section.contents;
        if !section_contents.contains_key(key) {
            return Err(Error::KeyNotFound {
                name: key.to_string(),
            });
        }
        let values = &section_contents[key];
        Ok(values.to_vec())
    }
}

fn is_comment(line: &str) -> bool {
    for c in &["#", ";"] {
        if line.trim_start().starts_with(c) {
            return true;
        }
    }
    false
}

fn is_section(line: &str) -> Option<String> {
    if line.starts_with('[') && line.ends_with(']') {
        return Some(line[1..line.len() - 1].to_string());
    }
    None
}

fn is_just_key(line: &str) -> Option<String> {
    if line.ends_with('=') {
        return Some(line[..line.len() - 1].trim().to_string());
    }
    None
}

fn find_delimiter(line: &str) -> Option<&str> {
    if line.contains('=') {
        return Some("=");
    }
    if line.contains(':') {
        return Some(":");
    }

    None
}

fn is_key_value(line: &str) -> Option<(String, String)> {
    if line.contains(">=") || line.contains("<=") || line.contains("==") {
        return None;
    }

    let delimiter = find_delimiter(line)?;
    let chunks: Vec<_> = line.splitn(2, delimiter).collect();
    let key = chunks[0].trim().to_string();
    let value = chunks[1].trim().to_string();
    Some((key, value))
}

#[derive(Debug, Eq, PartialEq)]
enum Token {
    Section(String),
    Key(String),
    Value(String),
}

fn tokenize(contents: &str) -> Vec<Token> {
    let mut tokens = vec![];
    for line in contents.lines() {
        if is_comment(line) {
            continue;
        }
        if let Some(section_name) = is_section(line) {
            let token = Token::Section(section_name.to_string());
            tokens.push(token);
            continue;
        }
        if let Some(key_name) = is_just_key(line) {
            let token = Token::Key(key_name);
            tokens.push(token);
            continue;
        }
        if let Some((key_name, value)) = is_key_value(line) {
            let token = Token::Key(key_name);
            tokens.push(token);
            let token = Token::Value(value);
            tokens.push(token);
            continue;
        }
        if !line.trim().is_empty() {
            tokens.push(Token::Value(line.trim().to_string()))
        }
    }
    tokens
}

fn peek_value(index: usize, tokens: &[Token]) -> Option<String> {
    if index >= tokens.len() - 1 {
        return None;
    }
    let token = &tokens[index + 1];
    match token {
        Token::Value(val) => Some(val.to_string()),
        _ => None,
    }
}

fn parse(contents: &str) -> Result<SetupCfg, Error> {
    let tokens = tokenize(contents);
    let mut res = SetupCfg::new();
    let mut i = 0;
    let mut section_name = "";
    while i < tokens.len() {
        let token = &tokens[i];
        match token {
            Token::Section(name) => {
                section_name = name;
                let section = Section::new(&name);
                res.sections.insert(name.to_string(), section);
                i += 1;
            }
            Token::Key(name) => {
                if !res.sections.contains_key(section_name) {
                    return Err(Error::MalformedSetupCfg {
                        details: "Missing section header".to_string(),
                    });
                }
                let current_section = res.sections.get_mut(section_name).unwrap();
                let mut values = vec![];
                while let Some(value) = peek_value(i, &tokens) {
                    i += 1;
                    values.push(value)
                }
                current_section.contents.insert(name.to_string(), values);
                i += 1;
            }
            Token::Value(value) => {
                return Err(Error::MalformedSetupCfg {
                    details: format!("Unexpected value: {}", value),
                });
            }
        }
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tokens(contents: &str, tokens: &[Token]) {
        let actual = tokenize(contents);
        assert_eq!(actual, tokens);
    }

    #[test]
    fn test_is_comment() {
        assert!(is_comment("; foo"));
        assert!(is_comment("   # inddented comment "));
    }

    #[test]
    fn test_tokenize_basics() {
        let contents = r#"
[options]
install_requires =
  pathlib2 ; python <= "3.5"
"#;
        assert_tokens(
            contents,
            &[
                Token::Section("options".to_string()),
                Token::Key("install_requires".to_string()),
                Token::Value("pathlib2 ; python <= \"3.5\"".to_string()),
            ],
        );
    }

    #[test]
    fn test_tokenize_colon_as_key_value_sep() {
        let contents = r#"
[options]
foo: bar
"#;
        assert_tokens(
            contents,
            &[
                Token::Section("options".to_string()),
                Token::Key("foo".to_string()),
                Token::Value("bar".to_string()),
            ],
        );
    }

    fn assert_invalid_syntax(contents: &str) {
        let outcome = SetupCfg::from_str(contents);
        assert!(outcome.is_err());
        println!("error: {}", &outcome.unwrap_err());
    }

    #[test]
    fn test_invalid_syntax() {
        assert_invalid_syntax(
            "
[section]
did_not_expect_me_there
",
        );

        assert_invalid_syntax("key_without_section=value");
    }

    #[test]
    fn test_corner_cases() {
        let setup_cfg = SetupCfg::from_str(
            "
[section]
foo =
  bar
  ",
        )
        .expect(""); // note the trailing space in the last line
        assert_multi(&setup_cfg, "section", "foo", &["bar".to_string()]);
    }

    const CONTENTS: &str = "\
[metadata]
name = foo
version = 0.42

[options]
install_requires =
  colorama
  # this is a comment
  path-py
";

    fn assert_single(setup_cfg: &SetupCfg, section: &str, key: &str, value: &str) {
        let actual = setup_cfg.get_single(section, key).expect("");
        assert_eq!(actual, value);
    }

    fn assert_multi(setup_cfg: &SetupCfg, section: &str, key: &str, values: &[String]) {
        let actual = setup_cfg.get_multi(section, key).expect("");
        assert_eq!(actual, values);
    }

    fn assert_single_error(setup_cfg: &SetupCfg, section: &str, key: &str) {
        let actual = setup_cfg.get_single(section, key);
        assert!(actual.is_err());
        println!("error: {}", actual.unwrap_err());
    }

    fn assert_multi_error(setup_cfg: &SetupCfg, section: &str, key: &str) {
        let actual = setup_cfg.get_multi(section, key);
        assert!(actual.is_err());
        println!("error: {}", actual.unwrap_err());
    }

    #[test]
    fn test_parse_happy() {
        let setup_cfg = SetupCfg::from_str(CONTENTS).expect("");

        assert_single(&setup_cfg, "metadata", "name", "foo");
        assert_single(&setup_cfg, "metadata", "version", "0.42");
        assert_multi(
            &setup_cfg,
            "options",
            "install_requires",
            &["colorama".to_string(), "path-py".to_string()],
        );
    }

    #[test]
    fn test_parse_errors() {
        let setup_cfg = SetupCfg::from_str(CONTENTS).expect("");
        assert_single_error(&setup_cfg, "metadata", "no-such-key");
        assert_single_error(&setup_cfg, "no-such-section", "version");
        assert_multi_error(&setup_cfg, "options", "no-such-option");
        assert_single_error(&setup_cfg, "options", "install_requires");
    }
}
