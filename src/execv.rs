use crate::error::*;
use std::convert::AsRef;
use std::ffi::OsStr;

#[cfg(unix)]
use std::{ffi::CString, os::unix::ffi::OsStrExt};

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
extern "C" {
    fn _wexecv(cmd: *const u16, args: *const *const u16) -> isize;
}

#[cfg(windows)]
fn to_utf16<S: AsRef<OsStr>>(string: S) -> Vec<u16> {
    let mut vec: Vec<_> = string.as_ref().encode_wide().collect();
    vec.push(0);
    vec
}

#[cfg(unix)]
fn to_c_string<S: AsRef<OsStr>>(string: S) -> Result<CString, Error> {
    CString::new(string.as_ref().as_bytes()).map_err(|_| Error::NulByteFound {
        arg: string.as_ref().to_string_lossy().to_string(),
    })
}

pub fn execv<Cmd, Args>(cmd: Cmd, args: Args) -> Result<(), Error>
where
    Cmd: AsRef<OsStr>,
    Args: IntoIterator,
    Args::Item: AsRef<OsStr>,
{
    #[cfg(windows)]
    {
        let cmd_wstr: Vec<_> = to_utf16(cmd);
        let args_wide_vec: Vec<Vec<_>> = args.into_iter().map(|s| to_utf16(s)).collect();
        let mut args_wstr: Vec<_> = args_wide_vec.iter().map(|v| v.as_ptr()).collect();
        args_wstr.push(std::ptr::null());

        unsafe {
            _wexecv(cmd_wstr.as_ptr(), args_wstr.as_ptr());
        }
    }
    #[cfg(unix)]
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

    Err(Error::ProcessStartError {
        message: format!("execv() failed: {}", std::io::Error::last_os_error()),
    })
}
