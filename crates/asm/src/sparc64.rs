//! Syscall implementations for `sparc64`.

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 5_i64,  // SYS_open
      in("o0") path,
      lateout("o0") ret,
      in("o1") flags,
      in("o2") 0_i32,  // mode
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 3_i64,  // SYS_read
      in("o0") i64::from(fd),
      lateout("o0") ret,
      in("o1") buf,
      in("o2") count,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 4_i64,  // SYS_write
      in("o0") i64::from(fd),
      lateout("o0") ret,
      in("o1") buf,
      in("o2") count,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 6_i64,  // SYS_close
      in("o0") i64::from(fd),
      lateout("o0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 189_i64,  // SYS_newuname
      in("o0") buf,
      lateout("o0") ret,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 157_i64,  // SYS_statfs
      in("o0") path,
      lateout("o0") ret,
      in("o1") buf,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_sysinfo(info: *mut SysInfo) -> i64 {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 214_i64, // __NR_sysinfo
      in("o0") info,
      lateout("o0") ret,
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
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 260_i64,  // __NR_sched_getaffinity
      in("o0") i64::from(pid),
      lateout("o0") ret,
      in("o1") mask_size,
      in("o2") mask,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_exit(code: i32) -> ! {
  unsafe {
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      nr = in(reg) 1_i64,  // SYS_exit
      in("o0") code,
      options(noreturn, nostack)
    );
  }
}
