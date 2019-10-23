use crate::dependencies::{GitDependency, LockedDependency, SimpleDependency, VersionSpec};
use crate::error::Error;

pub fn parse(text: &str) -> Result<Vec<LockedDependency>, Error> {
    let mut res = vec![];
    let lines = split_logical_lines(&text);
    for line in lines.iter() {
        if !line.starts_with('#') {
            // skip comments
            let locked_dependency = parse_line(line)?;
            res.push(locked_dependency);
        }
    }
    Ok(res)
}

fn split_logical_lines(text: &str) -> Vec<String> {
    let not_joined = text.split_terminator(|x| x == '\n');
    let mut res = vec![];
    let mut current_line = String::new();
    for line in not_joined {
        current_line.push_str(line);
        current_line.push('\n');
        if !line.ends_with('\\') {
            res.push(current_line.clone());
            current_line.clear();
        }
    }
    res
}

pub fn parse_line(line: &str) -> Result<LockedDependency, Error> {
    if line.contains("==") {
        let simple_dep = parse_simple_line(line)?;
        return Ok(LockedDependency::Simple(simple_dep));
    }
    if line.contains("#egg=") {
        let git_dep = parse_git_line(line)?;
        return Ok(LockedDependency::Git(git_dep));
    }
    Err(Error::MalformedLock {
        details: format!(
            "Could not parse line `{}` as either a simple or a git dependency",
            line.trim_end()
        ),
    })
}

// Note: technically this function cannot fail, but we want to keep symetry with
// parse_git_line()
pub fn parse_simple_line(line: &str) -> Result<SimpleDependency, Error> {
    let version = parse_simple_version(&line);
    let name = parse_simple_name(&line);
    Ok(SimpleDependency {
        line: line.to_string(),
        name,
        version,
    })
}

fn parse_simple_name(line: &str) -> String {
    let dep_name = line.split("==").next().unwrap();
    dep_name.trim().to_string()
}

fn parse_simple_version(line: &str) -> VersionSpec {
    let equal_index = line
        .find("==")
        .unwrap_or_else(|| panic!("'{}' should contain '=='", line));

    let search_pos = equal_index + 2;
    let mut chars_it = line.chars().skip(search_pos);

    let start = search_pos + chars_it.position(|c| !c.is_ascii_whitespace()).unwrap();

    // Advance `end` until:
    // * the next blank char,
    // * the end of the line,
    // * the backslash character (before a line continuation)
    // * the hash character (before a comment)
    let is_end = |c: char| c.is_ascii_whitespace() || (c == '\\') || (c == '#');
    let end = match chars_it.position(is_end) {
        Some(pos) => pos + start + 1,
        None => line.len(),
    };

    let version = &line[start..end];

    VersionSpec {
        start,
        end,
        value: version.to_string(),
    }
}

pub fn parse_git_line(line: &str) -> Result<GitDependency, Error> {
    let name = parse_git_name(line);
    let git_ref = parse_git_ref(line)?;
    Ok(GitDependency {
        line: line.to_string(),
        name,
        git_ref,
    })
}

fn parse_git_name(line: &str) -> String {
    let dep_name = line.rsplit("egg=").next().unwrap();
    dep_name.trim().to_string()
}

fn parse_git_ref(line: &str) -> Result<VersionSpec, Error> {
    let chunks: Vec<_> = line.rsplit('@').collect();
    // chunks is [git, foo:com:bar/baz, abce64#egg=bar]
    let after_at = chunks
        .first()
        .unwrap_or_else(|| panic!("'{}' should contain '@'", line));

    let chunks: Vec<_> = after_at.split('#').collect();
    // chunks is [abce64, egg=bar]
    if chunks.len() != 2 {
        return Err(Error::MalformedLock {
            details: format!(
                "expecting `<ref>#egg=<name>` after `@`, got '{}'",
                after_at.trim_end()
            ),
        });
    }
    let value = chunks[0].to_string();
    let start = line.len() - after_at.len();
    let end = start + value.len();
    Ok(VersionSpec { value, start, end })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_lock() {
        let lock_contents = "bar==42\ngit://foo/bar.git@master#egggg=bar";
        let actual = parse(&lock_contents);
        let actual = actual.unwrap_err();
        match actual {
            Error::MalformedLock { .. } => (),
            _ => panic!("Expecting MalformedLock, got: {}", actual),
        }
    }

    #[test]
    fn test_split_into_logical_lines() {
        let text = "\
foo
bar \\
  baz
";
        let lines = split_logical_lines(text);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "foo\n");
        assert_eq!(lines[1], "bar \\\n  baz\n");
    }

    #[test]
    fn test_no_newline_at_the_end() {
        let lines = split_logical_lines("foo\nbar");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "foo\n");
        assert_eq!(lines[1], "bar\n");
    }

    #[test]
    fn test_parse_lock() {
        let text = "\
foo==0.42
git+ssh://git@host.tld:team/name.git@v0.32#egg=bar
";
        let actual_deps = parse(text).unwrap();

        assert_eq!(actual_deps.len(), 2);
    }

    #[test]
    fn test_parse_simple_version() {
        assert_eq!(parse_simple_version("foo == 0.42").value, "0.42");
        assert_eq!(parse_simple_version("foo == 0.42").start, 7);
        assert_eq!(parse_simple_version("foo == 0.42").end, 11);
        assert_eq!(
            parse_simple_version("foo == 0.42 ; python_version > '3.3'").value,
            "0.42"
        );
        assert_eq!(
            parse_simple_version("foo == 0.42\\ --hash=sha256:32fde42").value,
            "0.42"
        );
        assert_eq!(
            parse_simple_version("foo == 0.42#this is a comment").value,
            "0.42"
        );
    }

    #[test]
    fn test_parse_git_ref() {
        assert_eq!(
            parse_git_ref("foo@host.tld@v0.42#egg=foo").unwrap().value,
            "v0.42"
        );
    }

    #[test]
    fn test_parse_git_name() {
        assert_eq!(parse_git_name("foo@host.tld@v0.42#egg=foo"), "foo");
    }
}
