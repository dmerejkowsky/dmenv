use crate::dependencies::LockedDependency;

pub fn dump(locked_dependencies: &[LockedDependency]) -> String {
    // Dependencies are sorted according to their *lowercase* name.
    // This is consistent with how `pip freeze` is implemented.
    // See bottom of pip/_internal/operations/freeze.py:freeze()
    #![allow(clippy::redundant_closure)]
    let mut lines: Vec<_> = locked_dependencies.iter().map(|x| x.line()).collect();
    lines.sort_by(|x, y| x.to_lowercase().cmp(&y.to_lowercase()));
    let mut res = lines.join("");
    if !res.ends_with('\n') {
        res.push('\n');
    }
    res
}