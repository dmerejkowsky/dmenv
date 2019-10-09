use crate::error::Error;
use crate::lock::parse_simple_line;

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

    pub fn version(&self) -> String {
        match self {
            LockedDependency::Git(x) => x.git_ref.value.to_string(),
            LockedDependency::Simple(x) => x.version.value.to_string(),
        }
    }

    pub fn git_bump(&mut self, new_ref: &str) -> Result<(), Error> {
        match self {
            LockedDependency::Git(x) => {
                x.git_bump(new_ref);
                Ok(())
            }
            _ => Err(Error::IncorrectLockedType {
                name: self.name(),
                expected_type: "git".to_string(),
            }),
        }
    }

    pub fn simple_bump(&mut self, new_version: &str) -> Result<(), Error> {
        match self {
            LockedDependency::Simple(x) => {
                x.simple_bump(new_version);
                Ok(())
            }
            _ => Err(Error::IncorrectLockedType {
                name: self.name(),
                expected_type: "simple".to_string(),
            }),
        }
    }
}

#[derive(Debug)]
// Container for a git ref or a version number.
// We keep a record of the coordinates of the spec inside
// the line of the lock.
// This allows us to have meaningful diffs when calling `dmenv bump-in-lock`
pub struct VersionSpec {
    pub start: usize,
    pub end: usize,
    pub value: String,
}

#[derive(Debug)]
pub struct GitDependency {
    pub name: String,
    pub line: String,
    pub git_ref: VersionSpec,
}

impl GitDependency {
    pub fn git_bump(&mut self, new_ref: &str) {
        let VersionSpec { start, end, .. } = &self.git_ref;
        self.line = format!("{}{}{}", &self.line[0..*start], new_ref, &self.line[*end..],);
        self.git_ref.value = new_ref.to_string()
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
        let line = format!("{}=={}\n", name, frozen.version);
        parse_simple_line(&line).expect("failed to parse frozen line")
    }

    /// Make this dependency specific to a Python version
    pub fn python_version(&mut self, python_version: &str) {
        let trimmed_line = self.line.trim_end_matches('\n');
        self.line = format!("{} ; python_version {}\n", trimmed_line, python_version);
    }

    /// Make this dependency specific to a Python platform
    pub fn sys_platform(&mut self, sys_platform: &str) {
        let trimmed_line = self.line.trim_end_matches('\n');
        self.line = format!("{} ; sys_platform == '{}'\n", trimmed_line, sys_platform);
    }

    /// Bump a simple dependency to a new version
    pub fn simple_bump(&mut self, new_version: &str) {
        let VersionSpec { start, end, .. } = &self.version;
        self.line = format!(
            "{}{}{}",
            &self.line[0..*start],
            new_version,
            &self.line[*end..],
        );
        self.version.value = new_version.to_string();
    }

    /// Freeze a simple dependency to a new version
    pub fn freeze(&mut self, new_version: &str) {
        // Note: conceptually this is very different from
        // self.bump(). Here we are "merging" a version
        // from the lock with a version from `pip freeze`.
        // In self.bump() we are *setting* the new version
        // and want to know if the dependency has changed.
        // Both implementations just happen to be similar ...
        self.simple_bump(new_version);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::parse_git_line;

    #[test]
    fn git_bump() {
        let mut dep = parse_git_line("git@master.com:foo@master#egg=foo").unwrap();
        dep.git_bump("deadbeef");
        assert_eq!(dep.line, "git@master.com:foo@deadbeef#egg=foo");

    #[test]
    fn test_simple_freeze() {
        let dep = LockedDependency::from_line("foo==0.42").unwrap();
        let mut dep = unwrap_simple(dep);
        dep.freeze("0.43");
        assert_eq!(dep.line, "foo==0.43");
    }

    #[test]
    fn simple_bump() {
        let mut dep = parse_simple_line("foo == 0.42").unwrap();
        dep.simple_bump("0.43");
        assert_eq!(dep.line, "foo == 0.43");
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

    }
}
