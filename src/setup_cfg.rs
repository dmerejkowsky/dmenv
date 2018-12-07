use error::Error;
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
        let contents = std::fs::read_to_string(path)?;
        let res = Self::from_str(&contents)?;
        Ok(res)
    }

    pub fn get_single(&self, section: &str, key: &str) -> Result<String, Error> {
        let values = self.get_multi(section, key)?;
        if values.len() > 1 {
            return Err(Error::new(&format!("Multiple values found for '{}'", key)));
        }
        Ok(values[0].to_string())
    }

    pub fn get_multi(&self, section: &str, key: &str) -> Result<Vec<String>, Error> {
        if !&self.sections.contains_key(section) {
            return Err(Error::new(&format!("Section '{}' not found", section)));
        }
        let section = &self.sections[section];
        let section_contents = &section.contents;
        if !section_contents.contains_key(key) {
            return Err(Error::new(&format!("Key '{}' not found", key)));
        }
        let values = &section_contents[key];
        Ok(values.to_vec())
    }
}

fn is_section(line: &str) -> Option<String> {
    if line.starts_with("[") && line.ends_with("]") {
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

fn is_key_value(line: &str) -> Option<(String, String)> {
    if line.contains('=') {
        let chunks: Vec<_> = line.splitn(2, '=').collect();
        let key = chunks[0].trim().to_string();
        let value = chunks[1].trim().to_string();
        return Some((key, value));
    }
    None
}

#[derive(Debug)]
enum Token {
    Section(String),
    Key(String),
    Value(String),
}

fn tokenize(contents: &str) -> Vec<Token> {
    let mut tokens = vec![];
    for line in contents.lines() {
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

fn peek_value(index: usize, tokens: &Vec<Token>) -> Option<String> {
    if index >= tokens.len() - 1 {
        return None;
    }
    let token = &tokens[index + 1];
    match token {
        Token::Value(val) => return Some(val.to_string()),
        _ => return None,
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
                    return Err(Error::new("Missing section header"));
                }
                let mut current_section = res.sections.get_mut(section_name).unwrap();
                let mut values = vec![];
                while let Some(value) = peek_value(i, &tokens) {
                    i += 1;
                    values.push(value)
                }
                current_section.contents.insert(name.to_string(), values);
                i += 1;
            }
            Token::Value(value) => {
                return Err(Error::new(&format!("Unexpected value: {}", value)));
            }
        }
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        ).expect(""); // note the trailing space in the last line
        assert_multi(&setup_cfg, "section", "foo", vec!["bar".to_string()]);
    }

    const CONTENTS: &str = "\
[metadata]
name = foo
version = 0.42

[options]
install_requires =
  colorama
  path-py
";

    fn assert_single(setup_cfg: &SetupCfg, section: &str, key: &str, value: &str) {
        let actual = setup_cfg.get_single(section, key).expect("");
        assert_eq!(actual, value);
    }

    fn assert_multi(setup_cfg: &SetupCfg, section: &str, key: &str, values: Vec<String>) {
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
            vec!["colorama".to_string(), "path-py".to_string()],
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
