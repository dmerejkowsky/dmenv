//! Create a process job so that new processes we create get killed
//! when we die.
//!
//! Adapted from
//! https://github.com/rust-lang/rust/blob/1.31.1/src/bootstrap/job.rs

#![allow(nonstandard_style, dead_code)]

use std::io;
use std::mem;

type HANDLE = *mut u8;
type BOOL = i32;
type DWORD = u32;
type LPHANDLE = *mut HANDLE;
type LPVOID = *mut u8;
type JOBOBJECTINFOCLASS = i32;
type SIZE_T = usize;
type LARGE_INTEGER = i64;
type UINT = u32;
type ULONG_PTR = usize;
type ULONGLONG = u64;

const FALSE: BOOL = 0;
const DUPLICATE_SAME_ACCESS: DWORD = 0x2;
const PROCESS_DUP_HANDLE: DWORD = 0x40;
const JobObjectExtendedLimitInformation: JOBOBJECTINFOCLASS = 9;
const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: DWORD = 0x2000;
const JOB_OBJECT_LIMIT_PRIORITY_CLASS: DWORD = 0x00000020;
const SEM_FAILCRITICALERRORS: UINT = 0x0001;
const SEM_NOGPFAULTERRORBOX: UINT = 0x0002;
const BELOW_NORMAL_PRIORITY_CLASS: DWORD = 0x00004000;

extern "system" {
    fn CreateJobObjectW(lpJobAttributes: *mut u8, lpName: *const u8) -> HANDLE;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn GetCurrentProcess() -> HANDLE;
    fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
    fn DuplicateHandle(
        hSourceProcessHandle: HANDLE,
        hSourceHandle: HANDLE,
        hTargetProcessHandle: HANDLE,
        lpTargetHandle: LPHANDLE,
        dwDesiredAccess: DWORD,
        bInheritHandle: BOOL,
        dwOptions: DWORD,
    ) -> BOOL;
    fn AssignProcessToJobObject(hJob: HANDLE, hProcess: HANDLE) -> BOOL;
    fn SetInformationJobObject(
        hJob: HANDLE,
        JobObjectInformationClass: JOBOBJECTINFOCLASS,
        lpJobObjectInformation: LPVOID,
        cbJobObjectInformationLength: DWORD,
    ) -> BOOL;
    fn SetErrorMode(mode: UINT) -> UINT;
}

#[repr(C)]
struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
    BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION,
    IoInfo: IO_COUNTERS,
    ProcessMemoryLimit: SIZE_T,
    JobMemoryLimit: SIZE_T,
    PeakProcessMemoryUsed: SIZE_T,
    PeakJobMemoryUsed: SIZE_T,
}

#[repr(C)]
struct IO_COUNTERS {
    ReadOperationCount: ULONGLONG,
    WriteOperationCount: ULONGLONG,
    OtherOperationCount: ULONGLONG,
    ReadTransferCount: ULONGLONG,
    WriteTransferCount: ULONGLONG,
    OtherTransferCount: ULONGLONG,
}

#[repr(C)]
struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
    PerProcessUserTimeLimit: LARGE_INTEGER,
    PerJobUserTimeLimit: LARGE_INTEGER,
    LimitFlags: DWORD,
    MinimumWorkingsetSize: SIZE_T,
    MaximumWorkingsetSize: SIZE_T,
    ActiveProcessLimit: DWORD,
    Affinity: ULONG_PTR,
    PriorityClass: DWORD,
    SchedulingClass: DWORD,
}

pub unsafe fn setup() {
    // Create a new job object for us to use
    let job = CreateJobObjectW(0 as *mut _, 0 as *const _);
    assert!(job != 0 as *mut _, "{}", io::Error::last_os_error());

    // Indicate that when all handles to the job object are gone that all
    // process in the object should be killed. Note that this includes our
    // entire process tree by default because we've added ourselves and our
    // children will reside in the job by default.
    let mut info = mem::zeroed::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>();
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    let r = SetInformationJobObject(
        job,
        JobObjectExtendedLimitInformation,
        &mut info as *mut _ as LPVOID,
        mem::size_of_val(&info) as DWORD,
    );
    assert!(r != 0, "{}", io::Error::last_os_error());

    // Assign our process to this job object. Note that if this fails, one very
    // likely reason is that we are ourselves already in a job object! This can
    // happen on the build bots that we've got for Windows, or if just anyone
    // else is instrumenting the build. In this case we just bail out
    // immediately and assume that they take care of it.
    //
    // Also note that nested jobs (why this might fail) are supported in recent
    // versions of Windows, but the version of Windows that our bots are running
    // at least don't support nested job objects.
    let r = AssignProcessToJobObject(job, GetCurrentProcess());
    if r == 0 {
        CloseHandle(job);
        return;
    }

    // Note: intentionally leak the job object handle. When our process exits
    // (normally or abnormally) it will close the handle implicitly, causing all
    // processes in the job to be cleaned up.
}
