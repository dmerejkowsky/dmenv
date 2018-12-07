use error::Error;

pub struct Lock {
    contents: String,
}

// Takes (line, name, version) and returns the bumped line
type BumpFunc = Fn(&str, &str, &str) -> Result<String, Error>;

// line is:
//   git@foo.com:bar/baz@<old>#egg=bar
// we want:
//   git@foo.com:bar/baz@<new>@egg=bar
fn git_bump(line: &str, name: &str, git_ref: &str) -> Result<String, Error> {
    if !line.contains("@") {
        return Ok(line.to_string());
    }
    let chunks: Vec<_> = line.rsplit("@").collect();
    // chunks is [git, foo:com:bar/baz, abce64#egg=bar]
    let after_at = chunks.first().unwrap();
    let chunks: Vec<_> = after_at.split("#").collect();
    // chunks is [abce64, egg=bar]
    if chunks.len() != 2 {
        return Err(Error::new(&format!(
            "Expecting `<ref>#egg=<name>` after `@`, got '{}'",
            after_at
        )));
    }
    let dep_ref = chunks[0];

    let start = line.len() - after_at.len();
    let end = start + dep_ref.len();

    let with_egg = chunks[1];
    if !with_egg.starts_with("egg=") {
        return Err(Error::new(&format!(
            "Expecting '{}' to start with `egg=`",
            with_egg,
        )));
    }
    let dep_name = &with_egg[4..];
    if dep_name != name {
        return Ok(line.to_string());
    }

    let mut res = String::new();
    res.push_str(&line[0..start]);
    res.push_str(git_ref);
    res.push_str(&line[end..]);
    Ok(res)
}

// line is:
//   foo==<old>
// we want:
//   foo==<new>
fn simple_bump(line: &str, name: &str, version: &str) -> Result<String, Error> {
    if !line.contains("==") {
        return Ok(line.to_string());
    }
    let words: Vec<_> = line.split("==").collect();
    if words.len() != 2 {
        return Err(Error::new(&format!(
            "Expecting `<name>==<version>`, got '{}'",
            line
        )));
    }

    let dep_name = words[0];
    if dep_name != name {
        return Ok(line.to_string());
    }

    Ok(format!("{}=={}", dep_name, version).to_string())
}

impl Lock {
    pub fn new(contents: &str) -> Lock {
        Lock {
            contents: contents.to_owned(),
        }
    }

    pub fn bump(&self, name: &str, version: &str) -> Result<String, Error> {
        self.bump_with_func(name, version, Box::new(simple_bump))
    }

    pub fn git_bump(&self, name: &str, git_ref: &str) -> Result<String, Error> {
        self.bump_with_func(name, git_ref, Box::new(git_bump))
    }

    fn bump_with_func(
        &self,
        name: &str,
        version: &str,
        bump_func: Box<BumpFunc>,
    ) -> Result<String, Error> {
        let mut res = String::new();
        let mut num_changes = 0;
        for (i, line) in self.contents.lines().enumerate() {
            let bumped_line = (bump_func)(line, name, version);
            if let Err(error) = bumped_line {
                return Err(Error::new(&format!(
                    "Malformed lock on line {}:\n{}",
                    (i + 1),
                    error
                )));
            }
            let bumped_line = bumped_line.unwrap();
            if bumped_line != line {
                num_changes += 1;
            }
            res.push_str(&bumped_line);
            res.push_str("\n");
        }
        if num_changes == 0 {
            return Err(Error::new("No changes made"));
        }
        if num_changes > 1 {
            return Err(Error::new("Too many changes"));
        }
        Ok(res)
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
        let err = actual.unwrap_err();
        assert!(err.to_string().contains("Malformed lock"));
        assert!(err.to_string().contains("line 2"));
    }

    #[test]
    fn simple_bump() {
        let lock_contents = r#"
# some comments
bar==0.3
foo==0.42
"#;
        let lock = Lock::new(lock_contents);
        let actual = lock.bump("foo", "0.43").expect("");
        let expected = lock_contents.replace("0.42", "0.43");
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
        let actual = lock.bump("no such", "0.43");
        let error = actual.unwrap_err();
        assert_eq!(error.to_string(), "No changes made");
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
        let actual = lock.git_bump("bar", new_sha1).expect("");
        let expected = lock_contents.replace(old_sha1, new_sha1);
        assert_eq!(actual, expected);
    }

}
