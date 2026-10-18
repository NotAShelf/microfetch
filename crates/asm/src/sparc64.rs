//! Syscall implementations for `sparc64`.

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 5i64,  // SYS_open
      inlateout("o0") path => ret,
      in("o1") flags,
      in("o2") 0i32,  // mode
      options(nostack)
    );
    #[allow(clippy::cast_possible_truncation)]
    {
      ret as i32
    }
  }
}

pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 3i64,  // SYS_read
      inlateout("o0") fd as i64 => ret,
      in("o1") buf,
      in("o2") count,
      options(nostack)
    );
    ret as isize
  }
}

pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 4i64,  // SYS_write
      inlateout("o0") fd as i64 => ret,
      in("o1") buf,
      in("o2") count,
      options(nostack)
    );
    ret as isize
  }
}

pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 6i64,  // SYS_close
      inlateout("o0") fd as i64 => ret,
      options(nostack)
    );
    #[allow(clippy::cast_possible_truncation)]
    {
      ret as i32
    }
  }
}

pub unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 189i64,  // SYS_newuname
      inlateout("o0") buf => ret,
      options(nostack)
    );
    #[allow(clippy::cast_possible_truncation)]
    {
      ret as i32
    }
  }
}

pub unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 157i64,  // SYS_statfs
      inlateout("o0") path => ret,
      in("o1") buf,
      options(nostack)
    );
    #[allow(clippy::cast_possible_truncation)]
    {
      ret as i32
    }
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
      inlateout("o0") info => ret,
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
    let ret: i64;
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      "bcs,a %xcc, 1f",
      "sub %g0, %o0, %o0",
      "1:",
      nr = in(reg) 260i64,  // __NR_sched_getaffinity
      inlateout("o0") pid as i64 => ret,
      in("o1") mask_size,
      in("o2") mask,
      options(nostack)
    );
    #[allow(clippy::cast_possible_truncation)]
    {
      ret as i32
    }
  }
}

pub unsafe fn sys_exit(code: i32) -> ! {
  unsafe {
    core::arch::asm!(
      "mov {nr}, %g1",
      "t 0x6d",
      nr = in(reg) 1i64,  // SYS_exit
      in("o0") code,
      options(noreturn, nostack)
    );
  }
}
