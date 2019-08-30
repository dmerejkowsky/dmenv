use crate::dependencies::LockedDependency;
use crate::error::Error;
use crate::lock::Lock;

// Common trait used by any struct able to bump a dependency
trait Bump {
    /// Modify the dep passed as argument.
    /// Returns true if the dependency actually changed
    fn bump(&self, dep: &mut LockedDependency) -> bool;
}

struct SimpleBumper {
    version: String,
}

/// Changes the `version` field for the `Simple`
/// variant of the `LockedDependency` enum
impl SimpleBumper {
    fn new(version: &str) -> Self {
        SimpleBumper {
            version: version.to_string(),
        }
    }
}

impl Bump for SimpleBumper {
    fn bump(&self, dep: &mut LockedDependency) -> bool {
        if let LockedDependency::Simple(s) = dep {
            s.bump(&self.version)
        } else {
            false
        }
    }
}

/// Changes the `git_ref` field for the `Git`
/// variant of the `LockedDependency` enum
struct GitBumper {
    git_ref: String,
}

impl GitBumper {
    fn new(git_ref: &str) -> Self {
        GitBumper {
            git_ref: git_ref.to_string(),
        }
    }
}

impl Bump for GitBumper {
    fn bump(&self, dep: &mut LockedDependency) -> bool {
        if let LockedDependency::Git(g) = dep {
            g.bump(&self.git_ref)
        } else {
            false
        }
    }
}

impl Lock {
    /// Bump the dependency `name` to new `version`.
    /// Returns a tuple (locked_changed: bool, new_contents: String)
    // Note: the locked_changed boolean is used to improve precision of
    // messages printed by the VenvManager struct.
    pub fn bump(&mut self, name: &str, version: &str) -> Result<bool, Error> {
        let simple_bumper = SimpleBumper::new(version);
        self.bump_impl(&simple_bumper, name)
    }

    /// Bump the git dependency `name` to new `git_ref`.
    /// Returns a tuple (locked_changed: bool, new_contents: String)
    // Note: the locked_changed boolean is used to improve precision of
    // messages printed by the VenvManager struct.
    pub fn git_bump(&mut self, name: &str, git_ref: &str) -> Result<bool, Error> {
        let git_bumper = GitBumper::new(git_ref);
        self.bump_impl(&git_bumper, name)
    }

    // Implement common behavior for any Bumper (regular or git)
    fn bump_impl<T>(&mut self, bumper: &T, name: &str) -> Result<bool, Error>
    where
        T: Bump,
    {
        let mut changed = true;
        let mut num_matches = 0;
        for dep in &mut self.dependencies {
            if dep.name() == name {
                num_matches += 1;
                changed = bumper.bump(dep);
            }
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
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_bump() {
        let lock_contents = "bar==0.3\nfoo==0.42\n";
        let mut lock = Lock::from_string(lock_contents).unwrap();
        let changed = lock.bump("foo", "0.43").unwrap();
        assert!(changed);
        let expected = lock_contents.replace("0.42", "0.43");
        let actual = lock.to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn dep_not_found() {
        let lock_contents = "bar==0.3\nfoo==0.42\n";
        let mut lock = Lock::from_string(lock_contents).unwrap();
        let actual = lock.bump("no-such", "0.43");
        match actual {
            Err(Error::NothingToBump { name }) => assert_eq!(name, "no-such"),
            _ => panic!("Expecting NothingToBump, got: {:?}", actual),
        }
    }

    #[test]
    fn idem_potent_change() {
        let lock_contents = "bar==0.3\nfoo==0.42\n";
        let mut lock = Lock::from_string(lock_contents).unwrap();
        let changed = lock.bump("bar", "0.3").unwrap();
        let actual = lock.to_string();
        assert!(!changed);
        assert_eq!(actual, lock_contents.to_string());
    }

    #[test]
    fn git_bump() {
        let old_sha1 = "dae42f";
        let lock_contents = format!("git@example.com/bar.git@{}#egg=bar\n", old_sha1);
        let mut lock = Lock::from_string(&lock_contents).unwrap();
        let new_sha1 = "cda431";
        let changed = lock.git_bump("bar", new_sha1).unwrap();
        assert!(changed);
        let expected = lock_contents.replace(old_sha1, new_sha1);
        let actual = lock.to_string();
        assert_eq!(actual, expected);
    }

}
