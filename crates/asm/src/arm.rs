//! Syscall implementations for `arm`.

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "svc #0",
      in("r7") 5i32,  // SYS_open
      in("r0") path,
      in("r1") flags,
      in("r2") 0i32,  // mode
      lateout("r0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "svc #0",
      in("r7") 3i32,  // SYS_read
      in("r0") fd,
      in("r1") buf,
      in("r2") count,
      lateout("r0") ret,
      options(nostack)
    );
    ret as isize
  }
}

pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "svc #0",
      in("r7") 4i32,  // SYS_write
      in("r0") fd,
      in("r1") buf,
      in("r2") count,
      lateout("r0") ret,
      options(nostack)
    );
    ret as isize
  }
}

pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "svc #0",
      in("r7") 6i32,  // SYS_close
      in("r0") fd,
      lateout("r0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "svc #0",
      in("r7") 122i32,  // SYS_newuname
      in("r0") buf,
      lateout("r0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "svc #0",
      in("r7") 266i32,  // SYS_statfs64
      in("r0") path,
      in("r1") core::mem::size_of::<StatfsBuf>(),
      in("r2") buf,
      lateout("r0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_sysinfo(info: *mut SysInfo) -> i64 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "svc #0",
      in("r7") 116_i32, // __NR_sysinfo
      in("r0") info,
      lateout("r0") ret,
      options(nostack)
    );
    i64::from(ret)
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
      "svc #0",
      in("r7") 242i32,  // __NR_sched_getaffinity
      in("r0") pid,
      in("r1") mask_size,
      in("r2") mask,
      lateout("r0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_exit(code: i32) -> ! {
  unsafe {
    core::arch::asm!(
      "svc #0",
      in("r7") 1i32,  // SYS_exit
      in("r0") code,
      options(noreturn, nostack)
    );
  }
}
