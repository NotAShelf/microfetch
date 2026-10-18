//! Syscall implementations for `powerpc64`.

use core::arch::asm;

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let ret: i32;
    asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 5_i64 => _,  // SYS_open
      in("r3") path,
      lateout("r3") ret,
      inlateout("r4") flags => _,
      inlateout("r5") 0_i32 => _,  // mode
      out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 3_i64 => _,  // SYS_read
      in("r3") i64::from(fd),
      lateout("r3") ret,
      inlateout("r4") buf => _,
      inlateout("r5") count => _,
      out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 4_i64 => _,  // SYS_write
      in("r3") i64::from(fd),
      lateout("r3") ret,
      inlateout("r4") buf => _,
      inlateout("r5") count => _,
      out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe {
    let ret: i32;
    asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 6_i64 => _,  // SYS_close
      in("r3") i64::from(fd),
      lateout("r3") ret,
      out("r4") _, out("r5") _, out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  unsafe {
    let ret: i32;
    asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 122_i64 => _,  // SYS_uname
      in("r3") buf,
      lateout("r3") ret,
      out("r4") _, out("r5") _, out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  unsafe {
    let ret: i32;
    asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 99_i64 => _,  // SYS_statfs
      in("r3") path,
      lateout("r3") ret,
      inlateout("r4") buf => _,
      out("r5") _, out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_sysinfo(info: *mut SysInfo) -> i64 {
  unsafe {
    let ret: i64;
    asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 116_i64 => _, // __NR_sysinfo
      in("r3") info,
      lateout("r3") ret,
      out("r4") _, out("r5") _, out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
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
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 223_i64 => _,  // __NR_sched_getaffinity
      in("r3") i64::from(pid),
      lateout("r3") ret,
      inlateout("r4") mask_size => _,
      inlateout("r5") mask => _,
      out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_exit(code: i32) -> ! {
  unsafe {
    asm!(
      "li 0, 1",  // SYS_exit
      "sc",
      in("r3") code,
      options(noreturn, nostack)
    );
  }
}
