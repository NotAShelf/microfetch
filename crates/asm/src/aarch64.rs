//! Syscall implementations for `aarch64`.

use core::arch::asm;

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let ret: i32;
    asm!(
      "svc #0",
      in("x8") 56_i64,  // SYS_openat
      in("x0") -100_i32,  // AT_FDCWD
      in("x1") path,
      in("x2") flags,
      in("x3") 0_i32,  // mode
      lateout("x0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    asm!(
      "svc #0",
      in("x8") 63_i64,  // SYS_read
      in("x0") fd,
      in("x1") buf,
      in("x2") count,
      lateout("x0") ret,
      options(nostack)
    );

    ret
  }
}

pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    asm!(
      "svc #0",
      in("x8") 64_i64,  // SYS_write
      in("x0") fd,
      in("x1") buf,
      in("x2") count,
      lateout("x0") ret,
      options(nostack)
    );

    ret
  }
}

pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe {
    let ret: i32;
    asm!(
      "svc #0",
      in("x8") 57_i64,  // SYS_close
      in("x0") fd,
      lateout("x0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  unsafe {
    let ret: i32;
    asm!(
      "svc #0",
      in("x8") 160_i64,  // SYS_uname
      in("x0") buf,
      lateout("x0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  unsafe {
    let ret: i32;
    asm!(
      "svc #0",
      in("x8") 43_i64,  // SYS_statfs
      in("x0") path,
      in("x1") buf,
      lateout("x0") ret,
      options(nostack)
    );

    ret
  }
}

pub unsafe fn sys_sysinfo(info: *mut SysInfo) -> i64 {
  unsafe {
    let ret: i64;
    asm!(
      "svc #0",
      in("x8") 179_i64, // __NR_sysinfo
      in("x0") info,
      lateout("x0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_sched_getaffinity(
  pid: i32,
  mask_size: usize,
  mask: *mut u8,
) -> i32 {
  unsafe {
    let ret: i32;
    asm!(
      "svc #0",
      in("x8") 123_i64,  // __NR_sched_getaffinity
      in("x0") pid,
      in("x1") mask_size,
      in("x2") mask,
      lateout("x0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_exit(code: i32) -> ! {
  unsafe {
    asm!(
      "svc #0",
      in("x8") 93_i64,  // SYS_exit
      in("x0") code,
      options(noreturn, nostack)
    );
  }
}
