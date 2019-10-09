use crate::dependencies::LockedDependency;
use crate::error::Error;

pub enum BumpType {
    Git,
    Simple,
}

pub fn simple_bump(
    dependencies: &mut [LockedDependency],
    name: &str,
    version: &str,
) -> Result<bool, Error> {
    bump_impl(dependencies, name, version, BumpType::Simple)
}

pub fn git_bump(
    dependencies: &mut [LockedDependency],
    name: &str,
    version: &str,
) -> Result<bool, Error> {
    bump_impl(dependencies, name, version, BumpType::Git)
}

fn bump_impl(
    dependencies: &mut [LockedDependency],
    name: &str,
    version: &str,
    bump_type: BumpType,
) -> Result<bool, Error> {
    let mut matching_names: Vec<_> = dependencies
        .iter_mut()
        .filter(|x| x.name() == name)
        .collect();
    if matching_names.is_empty() {
        return Err(Error::NothingToBump {
            name: name.to_string(),
        });
    }
    if matching_names.len() > 1 {
        return Err(Error::MultipleBumps {
            name: name.to_string(),
        });
    }
    let dep = &mut matching_names[0];
    if dep.version() == version {
        return Ok(false);
    }
    match bump_type {
        BumpType::Git => dep.git_bump(version)?,
        BumpType::Simple => dep.simple_bump(version)?,
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::{dump, parse};

    #[test]
    fn simple_change() {
        let lock_contents = "bar==0.3\nfoo==0.42\n";
        let mut deps = parse(&lock_contents).unwrap();
        let changed = simple_bump(&mut deps, "foo", "0.43").unwrap();
        assert!(changed);
        let actual = dump(&deps);
        assert_eq!(actual, "bar==0.3\nfoo==0.43\n");
    }

    #[test]
    fn simple_no_change() {
        let lock_contents = "bar==0.3\nfoo==0.42\n";
        let mut deps = parse(&lock_contents).unwrap();
        let changed = simple_bump(&mut deps, "foo", "0.42").unwrap();
        assert!(!changed);
        let actual = dump(&deps);
        assert_eq!(actual, "bar==0.3\nfoo==0.42\n");
    }

    #[test]
    fn dep_not_found() {
        let lock_contents = "bar==0.3\nfoo==0.42\n";
        let mut deps = parse(&lock_contents).unwrap();
        let actual_error = simple_bump(&mut deps, "no-such", "1.2");
        match actual_error {
            Err(Error::NothingToBump { name }) => assert_eq!(name, "no-such"),
            _ => panic!("Expecting NothingToBump, got: {:?}", actual_error),
        }
    }

    #[test]
    fn bump_git_ref() {
        let lock_contents = "git@example.com/bar.git@dae42f#egg=bar\n";
        let mut deps = parse(&lock_contents).unwrap();
        let changed = git_bump(&mut deps, "bar", "cda431").unwrap();
        assert!(changed);
        let actual = dump(&deps);
        let expected = "git@example.com/bar.git@cda431#egg=bar\n";
        assert_eq!(actual, expected);
    }
}
