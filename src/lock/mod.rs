use crate::dependencies::LockedDependency;

/// Implements various operations on the lock file
/// Usage:
/// ```text
/// let lock_contents = read_from("foo.lock");
/// let mut lock = Lock::from_string(lock_contents)
/// // Mutate the lock, for instance with `bump()` or `freeze()
/// let lock_contents = lock.to_string();
/// write_to("foo.lock", lock_contents);
/// ```
#[derive(Debug)]
pub struct Lock {
    dependencies: Vec<LockedDependency>,
    python_version: Option<String>,
    sys_platform: Option<String>,
}

mod bump;
mod freeze;
mod parse;
