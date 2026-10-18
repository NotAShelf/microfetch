//! Syscall implementations for `s390x`.

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "svc 0",
      in("r1") 5_i64,  // SYS_open
      in("r2") path,
      in("r3") flags,
      in("r4") 0_i32,  // mode
      lateout("r2") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    core::arch::asm!(
      "svc 0",
      in("r1") 3_i64,  // SYS_read
      in("r2") fd,
      in("r3") buf,
      in("r4") count,
      lateout("r2") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    core::arch::asm!(
      "svc 0",
      in("r1") 4_i64,  // SYS_write
      in("r2") fd,
      in("r3") buf,
      in("r4") count,
      lateout("r2") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "svc 0",
      in("r1") 6_i64,  // SYS_close
      in("r2") fd,
      lateout("r2") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "svc 0",
      in("r1") 122_i64,  // SYS_uname
      in("r2") buf,
      lateout("r2") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "svc 0",
      in("r1") 99_i64,  // SYS_statfs
      in("r2") path,
      in("r3") buf,
      lateout("r2") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_sysinfo(info: *mut SysInfo) -> i64 {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "svc 0",
      in("r1") 116_i64, // __NR_sysinfo
      in("r2") info,
      lateout("r2") ret,
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
    core::arch::asm!(
      "svc 0",
      in("r1") 240_i64,  // __NR_sched_getaffinity
      in("r2") pid,
      in("r3") mask_size,
      in("r4") mask,
      lateout("r2") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_exit(code: i32) -> ! {
  unsafe {
    core::arch::asm!(
      "svc 0",
      in("r1") 1_i64,  // SYS_exit
      in("r2") code,
      options(noreturn, nostack)
    );
  }
}
