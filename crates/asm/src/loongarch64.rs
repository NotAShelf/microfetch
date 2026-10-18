//! Syscall implementations for `loongarch64`.

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "syscall 0",
      in("$a7") 56i64,  // SYS_openat
      in("$a0") -100i32,  // AT_FDCWD
      in("$a1") path,
      in("$a2") flags,
      in("$a3") 0i32,  // mode
      lateout("$a0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    core::arch::asm!(
      "syscall 0",
      in("$a7") 63i64,  // SYS_read
      in("$a0") fd,
      in("$a1") buf,
      in("$a2") count,
      lateout("$a0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    core::arch::asm!(
      "syscall 0",
      in("$a7") 64i64,  // SYS_write
      in("$a0") fd,
      in("$a1") buf,
      in("$a2") count,
      lateout("$a0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "syscall 0",
      in("$a7") 57i64,  // SYS_close
      in("$a0") fd,
      lateout("$a0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "syscall 0",
      in("$a7") 160i64,  // SYS_uname
      in("$a0") buf,
      lateout("$a0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "syscall 0",
      in("$a7") 43i64,  // SYS_statfs
      in("$a0") path,
      in("$a1") buf,
      lateout("$a0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_sysinfo(info: *mut SysInfo) -> i64 {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "syscall 0",
      in("$a7") 179_i64, // __NR_sysinfo
      in("$a0") info,
      lateout("$a0") ret,
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
      "syscall 0",
      in("$a7") 123i64,  // __NR_sched_getaffinity
      in("$a0") pid,
      in("$a1") mask_size,
      in("$a2") mask,
      lateout("$a0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_exit(code: i32) -> ! {
  unsafe {
    core::arch::asm!(
      "syscall 0",
      in("$a7") 93i64,  // SYS_exit
      in("$a0") code,
      options(noreturn, nostack)
    );
  }
}
