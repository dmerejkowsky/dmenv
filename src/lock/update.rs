use crate::dependencies::{FrozenDependency, LockedDependency, SimpleDependency};
use crate::UpdateOptions;

#[derive(Debug)]
pub struct Updater {
    python_version: Option<String>,
    sys_platform: Option<String>,
}

impl Updater {
    pub fn new() -> Self {
        Updater {
            python_version: None,
            sys_platform: None,
        }
    }

    pub fn set_options(
        &mut self,
        UpdateOptions {
            python_version,
            sys_platform,
        }: UpdateOptions,
    ) {
        self.python_version = python_version;
        self.sys_platform = sys_platform;
    }

    /// Applies a set of new FrozenDependency to the lock
    // Basically, update `self.dependencies` using the new frozen deps,
    // making sure no existing information in the lock is lost.
    // Note that we only modify existing lines or add new ones
    // (no deletion occurs).
    pub fn update(
        &self,
        locked_dependencies: &mut Vec<LockedDependency>,
        frozen_dependencies: &[FrozenDependency],
    ) {
        self.patch_existing_deps(locked_dependencies, frozen_dependencies);
        self.add_missing_deps(locked_dependencies, frozen_dependencies);
    }

    /// Add dependencies from `frozen_deps` that were missing in the lock
    fn add_missing_deps(
        &self,
        locked_dependencies: &mut Vec<LockedDependency>,
        frozen_deps: &[FrozenDependency],
    ) {
        let known_names: Vec<_> = locked_dependencies.iter().map(|d| d.name()).collect();
        let new_deps: Vec<_> = frozen_deps
            .iter()
            .filter(|x| !known_names.contains(&x.name))
            .collect();
        for dep in new_deps {
            // If self.python_version or self.sys_platform is not None,
            // make sure to append that data.
            // For instance, if we generated the lock on Linux and we see a
            // new dependency `foo==42` while running `lock --platform=win32`,
            // we know `foo` *must* be Windows-specific.
            // Thus we want to write `foo==42; sys_platform = "win32"` in the lock
            // so that `foo` is *not* installed when running `pip install` on Linux.
            let mut locked_dep = SimpleDependency::from_frozen(dep);
            if let Some(python_version) = &self.python_version {
                locked_dep.python_version(python_version);
            }
            if let Some(sys_platform) = &self.sys_platform {
                locked_dep.sys_platform(sys_platform);
            }
            print!("+ {}", locked_dep.line);
            locked_dependencies.push(LockedDependency::Simple(locked_dep));
        }
    }

    /// Modify dependencies that were in the lock to match those passed in `frozen_deps`
    fn patch_existing_deps(
        &self,
        locked_dependencies: &mut Vec<LockedDependency>,
        frozen_deps: &[FrozenDependency],
    ) {
        for dep in locked_dependencies.iter_mut() {
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
        dep.update(&frozen_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::{dump, parse};

    impl FrozenDependency {
        pub fn new(name: &str, version: &str) -> Self {
            FrozenDependency {
                name: name.to_string(),
                version: version.to_string(),
            }
        }
    }

    fn assert_update(
        updater: Updater,
        initial_contents: &str,
        frozen: &[FrozenDependency],
        final_contents: &str,
    ) {
        let mut locked = parse(initial_contents).unwrap();
        updater.update(&mut locked, frozen);
        let actual = dump(locked);
        assert_eq!(actual, final_contents);
    }

    #[test]
    fn simple_dependency_upgraded() {
        let updater = Updater::new();
        assert_update(
            updater,
            "foo==0.42\n",
            &[FrozenDependency::new("foo", "0.43")],
            "foo==0.43\n",
        )
    }

    #[test]
    fn keep_old_deps() {
        let updater = Updater::new();
        assert_update(
            updater,
            "bar==1.3\nfoo==0.42\n",
            &[FrozenDependency::new("foo", "0.43")],
            "bar==1.3\nfoo==0.43\n",
        );
    }

    #[test]
    fn from_scratch() {
        let frozen_deps = vec![
            FrozenDependency::new("bar", "1.3"),
            FrozenDependency::new("foo", "0.42"),
        ];

        let updater = Updater::new();
        assert_update(updater, "", &frozen_deps, "bar==1.3\nfoo==0.42\n");
    }

    #[test]
    fn keep_git_deps() {
        let updater = Updater::new();
        assert_update(
            updater,
            "git@example.com:bar/foo.git@master#egg=foo\n",
            &[FrozenDependency::new("foo", "0.42")],
            "git@example.com:bar/foo.git@master#egg=foo\n",
        );
    }

    #[test]
    fn keep_specifications() {
        let updater = Updater::new();
        assert_update(
            updater,
            "foo == 1.3 ; python_version >= '3.6'\n",
            &[FrozenDependency::new("foo", "1.4")],
            "foo == 1.4 ; python_version >= '3.6'\n",
        );
    }

    #[test]
    fn add_new_deps() {
        let updater = Updater::new();
        assert_update(
            updater,
            "bar==6.2\n",
            &[FrozenDependency::new("foo", "0.42")],
            "bar==6.2\nfoo==0.42\n",
        );
    }

    #[test]
    fn different_python_version() {
        let mut updater = Updater::new();
        updater.set_options(UpdateOptions {
            python_version: Some("< '3.6'".to_string()),
            sys_platform: None,
        });
        assert_update(
            updater,
            "foo==0.42\n",
            &[
                FrozenDependency::new("bar", "1.3"),
                FrozenDependency::new("foo", "0.42"),
            ],
            "bar==1.3 ; python_version < '3.6'\nfoo==0.42\n",
        );
    }

    #[test]
    fn different_platform() {
        let mut updater = Updater::new();
        updater.set_options(UpdateOptions {
            python_version: None,
            sys_platform: Some("win32".to_string()),
        });
        assert_update(
            updater,
            "foo==0.42\n",
            &[
                FrozenDependency::new("foo", "0.42"),
                FrozenDependency::new("winapi", "1.3"),
            ],
            "foo==0.42\nwinapi==1.3 ; sys_platform == 'win32'\n",
        );
    }
}
