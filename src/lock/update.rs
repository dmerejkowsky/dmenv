use crate::dependencies::{FrozenDependency, LockedDependency, SimpleDependency};
use crate::lock::Lock;

impl Lock {
    /// Set the python version
    // Note: This cause the behavior of `freeze()` to change.
    // See `add_missing_deps` for details
    pub fn python_version(&mut self, python_version: &str) {
        self.python_version = Some(python_version.to_string())
    }

    /// Set the python platform
    // Note: This cause the behavior of `freeze()` to change.
    // See `add_missing_deps` for details
    pub fn sys_platform(&mut self, sys_platform: &str) {
        self.sys_platform = Some(sys_platform.to_string())
    }

    /// Applies a set of new FrozenDependency to the lock
    // Basically, "merge" `self.dependencies` with some new frozen deps and
    // make sure no existing information in the lock is lost
    // This in not an actual merge because we only modify existing lines
    // or add new ones (no deletion ocurrs).
    pub fn update(&mut self, deps: &[FrozenDependency]) {
        self.patch_existing_deps(deps);
        self.add_missing_deps(deps);
    }

    /// Add dependencies from `frozen_deps` that were missing in the lock
    fn add_missing_deps(&mut self, frozen_deps: &[FrozenDependency]) {
        #![allow(clippy::redundant_closure)]
        let known_names: &Vec<_> = &mut self.dependencies.iter().map(|d| d.name()).collect();
        let new_deps: Vec<_> = frozen_deps
            .iter()
            .filter(|x| !known_names.contains(&&x.name))
            .collect();
        for dep in new_deps {
            // If self.python_version or self.sys_platform is not None,
            // make sure to append that data.
            // For instance, if we generated the lock on Linux and we see a
            // new dependency `foo==42` while running `lock --platform=win32`,
            // we know `foo` *must* be Windows-specify.
            // Thus we want to write `foo==42; sys_platform = "win32"` in the lock
            // so that `foo` is *not* installed when running `pip install` on Linux.
            let mut locked_dep = SimpleDependency::from_frozen(dep);
            if let Some(python_version) = &self.python_version {
                locked_dep.python_version(python_version);
            }
            if let Some(sys_platform) = &self.sys_platform {
                locked_dep.sys_platform(sys_platform);
            }
            println!("+ {}", locked_dep.line);
            self.dependencies.push(LockedDependency::Simple(locked_dep));
        }
    }

    /// Modify dependencies that were in the lock to match those passed in `frozen_deps`
    fn patch_existing_deps(&mut self, frozen_deps: &[FrozenDependency]) {
        for dep in &mut self.dependencies {
            match dep {
                // frozen deps *never* contain git information (because `pip freeze`
                // only returns names and versions), so always keep those in the lock.
                LockedDependency::Git(_) => (),
                LockedDependency::Simple(s) => {
                    Self::patch_existing_dep(s, frozen_deps);
                }
            }
        }
    }

    /// Modify an existing dependency to match the frozen version
    fn patch_existing_dep(dep: &mut SimpleDependency, frozen_deps: &[FrozenDependency]) {
        let frozen_match = frozen_deps.iter().find(|x| x.name == dep.name);
        let frozen_version = match frozen_match {
            None => return,
            Some(frozen) => &frozen.version,
        };
        if &dep.version.value == frozen_version {
            return;
        }

        println!("{}: {} -> {}", dep.name, dep.version.value, &frozen_version);
        dep.freeze(&frozen_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl FrozenDependency {
        pub fn new(name: &str, version: &str) -> Self {
            FrozenDependency {
                name: name.to_string(),
                version: version.to_string(),
            }
        }
    }

    fn assert_update(contents: &str, frozen: &[FrozenDependency], expected: &str) {
        let mut lock = Lock::from_string(contents).unwrap();
        lock.update(frozen);
        let actual = lock.to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_dependency_upgraded() {
        assert_update(
            "foo==0.42\n",
            &[FrozenDependency::new("foo", "0.43")],
            "foo==0.43\n",
        );
    }

    #[test]
    fn keep_old_deps() {
        assert_update(
            "bar==1.3\nfoo==0.42\n",
            &[FrozenDependency::new("foo", "0.43")],
            "bar==1.3\nfoo==0.43\n",
        );
    }

    #[test]
    fn keep_git_deps() {
        assert_update(
            "git@example.com:bar/foo.git@master#egg=foo\n",
            &[FrozenDependency::new("foo", "0.42")],
            "git@example.com:bar/foo.git@master#egg=foo\n",
        );
    }

    #[test]
    fn keep_specifications() {
        assert_update(
            "foo == 1.3 ; python_version >= '3.6'\n",
            &[FrozenDependency::new("foo", "1.4")],
            "foo == 1.4 ; python_version >= '3.6'\n",
        );
    }

    #[test]
    fn add_new_deps() {
        assert_update("", &[FrozenDependency::new("foo", "0.42")], "foo==0.42\n");
    }

    #[test]
    fn different_python_version() {
        let mut lock = Lock::from_string("foo==0.42\n").unwrap();
        lock.python_version("< '3.6'");
        lock.update(&[
            FrozenDependency::new("foo", "0.42"),
            FrozenDependency::new("bar", "1.3"),
        ]);
        let actual = lock.to_string();
        assert_eq!(actual, "bar==1.3 ; python_version < '3.6'\nfoo==0.42\n");
    }

    #[test]
    fn different_platform() {
        let mut lock = Lock::from_string("foo==0.42\n").unwrap();
        lock.sys_platform("win32");
        lock.update(&[
            FrozenDependency::new("foo", "0.42"),
            FrozenDependency::new("winapi", "1.3"),
        ]);
        let actual = lock.to_string();
        assert_eq!(actual, "foo==0.42\nwinapi==1.3 ; sys_platform == 'win32'\n");
    }
}
