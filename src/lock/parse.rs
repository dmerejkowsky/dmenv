use crate::dependencies::LockedDependency;
use crate::error::Error;
use crate::lock::Lock;

impl Lock {
    pub fn from_string(string: &str) -> Result<Self, Error> {
        let mut dependencies = vec![];
        for (i, line) in string.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let dep = LockedDependency::from_line(&line).map_err(|e| Error::MalformedLock {
                line: i + 1,
                details: e.details,
            })?;
            dependencies.push(dep);
        }
        Ok(Lock {
            dependencies,
            python_version: None,
            sys_platform: None,
        })
    }

    /// Serialize the lock to a string
    pub fn to_string(&self) -> String {
        // Dependencies are sorted according to their *lowercase* name.
        // This is consistent with how `pip freeze` is implemented.
        // See bottom of pip/_internal/operations/freeze.py:freeze()
        #![allow(clippy::redundant_closure)]
        let mut lines: Vec<_> = self.dependencies.iter().map(|x| x.line()).collect();
        lines.sort_by(|x, y| x.to_lowercase().cmp(&y.to_lowercase()));
        lines.join("\n") + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_lock() {
        let lock_contents = "bar==42\ngit://foo/bar.git@master#egggg=bar";
        let actual = Lock::from_string(&lock_contents);
        let actual = actual.unwrap_err();
        match actual {
            Error::MalformedLock { line, .. } => assert_eq!(line, 2),
            _ => panic!("Expecting MalformedLock, got: {}", actual),
        }
    }
}
