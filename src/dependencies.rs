use crate::error::Error;

/// Home for types that represent dependencies.
///
/// * Frozen dependencies come from `pip freeze` output.
/// * Locked dependencies are read from the lock file and
///   are either the Simple variant (foo==42), or the Git variant
///   (git+https://git.local/foo@master#egg=foo)
///
/// Locked dependencies can either be *bumped* (when using `dmenv bump-in-lock`,
/// or *frozen*, when using `dmenv lock` and "merging" output from `pip freeze`
/// with the contents of the lock file.

#[derive(Debug)]
/// Represent a parsing error
// Note: may be returned any from_*() method,,
// for instance, when parsing `pip freeze` output or parsing
// a lock file
pub struct ParseError {
    pub details: String,
}

impl ParseError {
    pub fn new(details: &str) -> Self {
        ParseError {
            details: details.to_string(),
        }
    }
}

pub struct FrozenDependency {
    pub name: String,
    pub version: String,
}

impl FrozenDependency {
    /// Construct a new FrozenDependency from a line coming from
    /// `pip freeze` output
    pub fn from_string(string: &str) -> Result<Self, Error> {
        // Custom error in case we can't parse `pip freeze` output
        // This really should never happen (tm)
        let err = Error::BrokenPipFreezeLine {
            line: string.to_string(),
        };

        if !string.contains("==") {
            return Err(err);
        }

        let words: Vec<_> = string.split("==").collect();
        if words.len() != 2 {
            return Err(err);
        }

        let name = words[0];
        let version = words[1];
        if version.is_empty() {
            return Err(err);
        }

        Ok(FrozenDependency {
            name: name.to_string(),
            version: version.to_string(),
        })
    }
}

#[derive(Debug)]
pub enum LockedDependency {
    Git(GitDependency),
    Simple(SimpleDependency),
}

impl LockedDependency {
    /// Serialize a locked dependency to a string
    //
    // Used by Lock::to_string()
    pub fn line(&self) -> String {
        match self {
            LockedDependency::Git(x) => x.line.to_string(),
            LockedDependency::Simple(x) => x.line.to_string(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            LockedDependency::Git(x) => x.name.to_string(),
            LockedDependency::Simple(x) => x.name.to_string(),
        }
    }

    // Parse a line from the lock. Return either a GitDependency or a SimpleDependency
    // Note that each of them contain a VersionSpec field (either `version` or `git_ref`)
    pub fn from_line(line: &str) -> Result<LockedDependency, ParseError> {
        if line.contains("#egg=") {
            let name = Self::parse_git_name(line);
            let git_ref = Self::parse_git_ref(line)?;
            let dep = GitDependency {
                line: line.to_string(),
                name,
                git_ref,
            };
            return Ok(LockedDependency::Git(dep));
        }
        if line.contains("==") {
            let version = Self::parse_simple_version(&line);
            let name = Self::parse_simple_name(&line);
            let dep = SimpleDependency {
                line: line.to_string(),
                name,
                version,
            };
            return Ok(LockedDependency::Simple(dep));
        }
        Err(ParseError::new("neither a simple dep nor a git dep"))
    }

    fn parse_simple_name(line: &str) -> String {
        let words: Vec<_> = line.split("==").collect();
        let dep_name = words[0];
        dep_name.trim().to_string()
    }

    fn parse_simple_version(line: &str) -> VersionSpec {
        let equal_index = line
            .find("==")
            .unwrap_or_else(|| panic!("'{}' should contain '=='", line));
        let mut start = equal_index + 2;
        let mut end = line.len();
        let colon_index = line.find(';');
        if let Some(colon_index) = colon_index {
            end = colon_index;
        }
        let version = &line[start..end];
        let num_blank_start = version.len() - version.trim_start().len();
        let num_blank_end = version.len() - version.trim_end().len();
        start += num_blank_start;
        end -= num_blank_end;
        VersionSpec {
            start,
            end,
            value: version.trim().to_string(),
        }
    }

    fn parse_git_name(line: &str) -> String {
        let parts: Vec<_> = line.rsplit("egg=").collect();
        let dep_name = parts[0];
        dep_name.trim().to_string()
    }

    fn parse_git_ref(line: &str) -> Result<VersionSpec, ParseError> {
        let chunks: Vec<_> = line.rsplit('@').collect();
        // chunks is [git, foo:com:bar/baz, abce64#egg=bar]
        let after_at = chunks
            .first()
            .unwrap_or_else(|| panic!("'{}' should contain '@'", line));

        let chunks: Vec<_> = after_at.split('#').collect();
        // chunks is [abce64, egg=bar]
        if chunks.len() != 2 {
            return Err(ParseError::new(&format!(
                "expecting `<ref>#egg=<name>` after `@`, got '{}'",
                after_at
            )));
        }
        let value = chunks[0].to_string();
        let start = line.len() - after_at.len();
        let end = start + value.len();
        Ok(VersionSpec { value, start, end })
    }
}

#[derive(Debug)]
// Container for a git ref or a version number.
// We keep a record of the coordinates of the spec inside
// the line of the lock.
// This allows us to have meaningful diffs when calling `dmenv bump-in-lock`
pub struct VersionSpec {
    start: usize,
    end: usize,
    pub value: String,
}

#[derive(Debug)]
pub struct GitDependency {
    pub name: String,
    pub line: String,
    pub git_ref: VersionSpec,
}

impl GitDependency {
    pub fn bump(&mut self, new_ref: &str) -> bool {
        let VersionSpec { start, end, value } = &self.git_ref;
        if new_ref == value {
            return false;
        }
        self.line = format!("{}{}{}", &self.line[0..*start], new_ref, &self.line[*end..],);
        self.git_ref.value = new_ref.to_string();
        true
    }
}

#[derive(Debug)]
pub struct SimpleDependency {
    pub name: String,
    pub line: String,
    pub version: VersionSpec,
}

impl SimpleDependency {
    /// Convert a FrozenDependency to a SimpleDependency
    /// This allows adding a dependency coming from `pip freeze` to the lock.
    pub fn from_frozen(frozen: &FrozenDependency) -> Self {
        let name = &frozen.name;
        let line = format!("{}=={}", name, frozen.version);
        let version = LockedDependency::parse_simple_version(&line);
        SimpleDependency {
            name: name.to_string(),
            version,
            line,
        }
    }

    /// Make this dependency specific to a Python version
    pub fn python_version(&mut self, python_version: &str) {
        self.line = format!("{} ; python_version {}", self.line, python_version);
    }

    /// Make this dependency specific to a Python platform
    pub fn sys_platform(&mut self, sys_platform: &str) {
        self.line = format!("{} ; sys_platform == '{}'", self.line, sys_platform);
    }

    /// Bump a simple dependency to a new version
    pub fn bump(&mut self, new_version: &str) -> bool {
        let VersionSpec { start, end, value } = &self.version;
        if new_version == value {
            return false;
        }
        self.line = format!(
            "{}{}{}",
            &self.line[0..*start],
            new_version,
            &self.line[*end..],
        );
        self.version.value = new_version.to_string();
        true
    }

    /// Freeze a simple dependency to a new version
    pub fn freeze(&mut self, new_version: &str) {
        // Note: conceptually this is very different from
        // self.bump(). Here we are "merging" a version
        // from the lock with a version from `pip freeze`.
        // In self.bump() we are *setting* the new version
        // and want to know if the dependency has changed.
        // Both implementations just happen to be similar ...
        self.bump(new_version);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_version_trivial() {
        let version = LockedDependency::parse_simple_version("foo==0.42");
        assert_eq!(version.value, "0.42");
        assert_eq!(version.start, 5);
        assert_eq!(version.end, 9);
    }

    #[test]
    fn test_parse_simple_version_with_padding() {
        let version = LockedDependency::parse_simple_version("foo == 0.42");
        assert_eq!(version.value, "0.42");
        assert_eq!(version.start, 7);
        assert_eq!(version.end, 11);
    }

    #[test]
    fn test_parse_simple_version_with_spec_no_padding() {
        let version = LockedDependency::parse_simple_version("foo==0.42;python_version <= '3.6'");
        assert_eq!(version.start, 5);
        assert_eq!(version.end, 9);
        assert_eq!(version.value, "0.42");
    }

    #[test]
    fn test_parse_simple_version_with_spec_and_padding() {
        let version =
            LockedDependency::parse_simple_version("foo == 0.42 ; python_version <= '3.6'");
        assert_eq!(version.start, 7);
        assert_eq!(version.end, 11);
        assert_eq!(version.value, "0.42");
    }

    #[test]
    fn test_parse_git_ref() {
        let git_ref = LockedDependency::parse_git_ref("git@host.tld:foo@master#egg=foo").unwrap();
        assert_eq!(git_ref.value, "master");
        assert_eq!(git_ref.start, 17);
        assert_eq!(git_ref.end, 23);
    }

    fn unwrap_simple(dep: LockedDependency) -> SimpleDependency {
        match dep {
            LockedDependency::Simple(s) => s,
            _ => panic!("Expected SimpleDependency, got {:?}", dep),
        }
    }

    fn unwrap_git(dep: LockedDependency) -> GitDependency {
        match dep {
            LockedDependency::Git(g) => g,
            _ => panic!("Expected GitDependency, got {:?}", dep),
        }
    }

    #[test]
    fn test_simple_freeze() {
        let dep = LockedDependency::from_line("foo==0.42").unwrap();
        let mut dep = unwrap_simple(dep);
        dep.freeze("0.43");
        assert_eq!(dep.line, "foo==0.43");
    }

    #[test]
    fn test_simple_keep_spec() {
        let dep = LockedDependency::from_line("foo==0.42 ; python_version >= '3.6'").unwrap();
        let mut dep = unwrap_simple(dep);
        dep.freeze("0.43");
        assert_eq!(dep.line, "foo==0.43 ; python_version >= '3.6'");
    }

    #[test]
    fn test_simple_keep_spec_with_padding() {
        let dep = LockedDependency::from_line("foo == 0.42 ; python_version >= '3.6'").unwrap();
        let mut dep = unwrap_simple(dep);
        dep.freeze("0.43");
        assert_eq!(dep.line, "foo == 0.43 ; python_version >= '3.6'");
    }

    #[test]
    fn test_freeze_version() {
        let dep = LockedDependency::from_line("foo2==2").unwrap();
        let mut dep = unwrap_simple(dep);
        dep.freeze("3");
        assert_eq!(dep.line, "foo2==3");
    }

    #[test]
    fn test_bump_git() {
        let dep = LockedDependency::from_line("git@master.com:foo@master#egg=foo").unwrap();
        let mut dep = unwrap_git(dep);
        dep.bump("deadbeef");
        assert_eq!(dep.line, "git@master.com:foo@deadbeef#egg=foo");
    }
}
