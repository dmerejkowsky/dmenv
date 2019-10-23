use crate::error::*;
use std::convert::AsRef;
use std::ffi::OsStr;

use std::{ffi::CString, os::unix::ffi::OsStrExt};

fn to_c_string<S: AsRef<OsStr>>(string: S) -> Result<CString, Error> {
    CString::new(string.as_ref().as_bytes()).map_err(|_| Error::NulByteError {
        arg: string.as_ref().to_string_lossy().to_string(),
    })
}

/// Wrap execv() C function from libc crate
// Note: Use by `dmenv run` so that killing the dmenv process
// does not create an orphan process
pub fn execv<Cmd, Args>(cmd: Cmd, args: Args) -> Result<(), Error>
where
    Cmd: AsRef<OsStr>,
    Args: IntoIterator,
    Args::Item: AsRef<OsStr>,
{
    {
        let cmd_cstring = to_c_string(cmd)?;
        let mut args_cstring = Vec::new();
        for arg in args {
            args_cstring.push(to_c_string(&arg)?);
        }
        let mut args_ptr: Vec<_> = args_cstring.iter().map(|x| x.as_ptr()).collect();
        args_ptr.push(std::ptr::null());

        unsafe {
            libc::execv(cmd_cstring.as_ptr(), args_ptr.as_ptr());
        }
    }

    Err(Error::StartProcessError {
        message: format!("execv() failed: {}", std::io::Error::last_os_error()),
    })
}
