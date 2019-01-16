use crate::error::Error;

pub struct Lock {
    contents: String,
}

#[derive(Debug)]
struct ParseError {
    details: String,
}

impl ParseError {
    pub fn new(details: &str) -> Self {
        ParseError {
            details: details.to_string(),
        }
    }
}

/// Takes `(line, name, version)` and returns either:
///   - `None' if no match was found
///   - Or `Some(new line) if a match was found
type BumpFunc = Fn(&str, &str, &str) -> Result<Option<String>, ParseError>;

/// Bump the reference number for a git dependency specification
///
/// line is:
///   git@foo.com:bar/baz@<old>#egg=bar
/// we want:
///   git@foo.com:bar/baz@<new>@egg=bar
///
/// Return None if the name was not found, or Some(new_line)
fn git_bump(line: &str, name: &str, git_ref: &str) -> Result<Option<String>, ParseError> {
    if !line.contains('@') {
        return Ok(None);
    }
    let chunks: Vec<_> = line.rsplit('@').collect();
    // chunks is [git, foo:com:bar/baz, abce64#egg=bar]
    let after_at = chunks.first().unwrap();
    let chunks: Vec<_> = after_at.split('#').collect();
    // chunks is [abce64, egg=bar]
    if chunks.len() != 2 {
        return Err(ParseError::new(&format!(
            "expecting `<ref>#egg=<name>` after `@`, got '{}'",
            after_at
        )));
    }
    let dep_ref = chunks[0];

    let start = line.len() - after_at.len();
    let end = start + dep_ref.len();

    let with_egg = chunks[1];
    if !with_egg.starts_with("egg=") {
        return Err(ParseError::new(&format!(
            "expecting '{}' to start with `egg=`",
            with_egg
        )));
    }
    let dep_name = &with_egg[4..];
    if dep_name != name {
        return Ok(None);
    }

    let mut res = String::new();
    res.push_str(&line[0..start]);
    res.push_str(git_ref);
    res.push_str(&line[end..]);
    Ok(Some(res))
}

/// Bump the version number for a simple dependency specification
///
/// line is:
///    foo==<old>
///
///  we want:
///    foo==<new>
///
/// Return None if the name was not found, or Some(new_line)
fn simple_bump(line: &str, name: &str, version: &str) -> Result<Option<String>, ParseError> {
    if !line.contains("==") {
        return Ok(None);
    }
    let words: Vec<_> = line.split("==").collect();
    if words.len() != 2 {
        return Err(ParseError::new(&format!(
            "expecting `<name>==<version>`, got '{}'",
            line
        )));
    }

    let dep_name = words[0];
    if dep_name != name {
        return Ok(None);
    }

    Ok(Some(format!("{}=={}", dep_name, version)))
}

impl Lock {
    pub fn new(contents: &str) -> Lock {
        Lock {
            contents: contents.to_owned(),
        }
    }

    /// Bump the dependency `name` to new `version`.
    /// Returns a tuple (locked_changed: bool, new_contents: String)
    pub fn bump(&self, name: &str, version: &str) -> Result<(bool, String), Error> {
        self.bump_with_func(name, version, Box::new(simple_bump))
    }

    /// Bump the git dependency `name` to new `git_ref`.
    /// Returns a tuple (locked_changed: bool, new_contents: String)
    pub fn git_bump(&self, name: &str, git_ref: &str) -> Result<(bool, String), Error> {
        self.bump_with_func(name, git_ref, Box::new(git_bump))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn bump_with_func(
        &self,
        name: &str,
        version: &str,
        bump_func: Box<BumpFunc>,
    ) -> Result<(bool, String), Error> {
        let mut res = String::new();
        let mut num_matches = 0;
        let mut changed = false;
        for (i, line) in self.contents.lines().enumerate() {
            let bumped_line = (bump_func)(line, name, version);
            let bumped_line = bumped_line.map_err(|e| Error::MalformedLock {
                line: i + 1,
                details: e.details,
            })?;
            if let Some(bumped_line) = bumped_line {
                num_matches += 1;
                if bumped_line != line {
                    changed = true;
                }
                res.push_str(&format!("{}\n", bumped_line));
            } else {
                res.push_str(&format!("{}\n", line));
            };
        }
        if num_matches == 0 {
            return Err(Error::NothingToBump {
                name: name.to_string(),
            });
        }
        if num_matches > 1 {
            return Err(Error::MultipleBumps {
                name: name.to_string(),
            });
        }
        Ok((changed, res))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_lock() {
        let lock_contents = "\
# some comments
git@foo@dm/foo#egggg=bar
";
        let lock = Lock::new(lock_contents);
        let actual = lock.git_bump("bar", "0.43");
        match actual {
            Err(Error::MalformedLock { line, .. }) => assert_eq!(line, 2),
            _ => panic!("Expecting MalformedLock, got: {:?}", actual),
        }
    }

    #[test]
    fn simple_bump() {
        let lock_contents = r#"
# some comments
bar==0.3
foo==0.42
"#;
        let lock = Lock::new(lock_contents);
        let actual = lock.bump("foo", "0.43").unwrap();
        let expected = (true, lock_contents.replace("0.42", "0.43"));
        assert_eq!(actual, expected);
    }

    #[test]
    fn dep_not_found() {
        let lock_contents = r#"
# some comments
bar==0.3
foo==0.42
"#;
        let lock = Lock::new(lock_contents);
        let actual = lock.bump("no-such", "0.43");
        match actual {
            Err(Error::NothingToBump { name }) => assert_eq!(name, "no-such"),
            _ => panic!("Expecting NothingToBump, got: {:?}", actual),
        }
    }

    #[test]
    fn idem_potent_change() {
        let lock_contents = r#"
# some comments
bar==0.3
foo==0.42
"#;
        let lock = Lock::new(lock_contents);
        let actual = lock.bump("bar", "0.3").unwrap();
        assert_eq!(actual, (false, lock_contents.to_string()));
    }

    #[test]
    fn git_bump() {
        let old_sha1 = "dae42f";
        let lock_contents = format!(
            r#"
# some comments
git@example.com/bar.git@{}#egg=bar
"#,
            old_sha1
        );
        let lock = Lock::new(&lock_contents);
        let new_sha1 = "cda431";
        let actual = lock.git_bump("bar", new_sha1).unwrap();
        let expected = (true, lock_contents.replace(old_sha1, new_sha1));
        assert_eq!(actual, expected);
    }

}
