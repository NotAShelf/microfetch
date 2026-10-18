//! Syscall implementations for `powerpc`.

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 5i32 => _,  // SYS_open
      inlateout("r3") path => ret,
      inlateout("r4") flags => _,
      inlateout("r5") 0i32 => _,
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
    let ret: i32;
    core::arch::asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 3i32 => _,  // SYS_read
      inlateout("r3") fd => ret,
      inlateout("r4") buf => _,
      inlateout("r5") count => _,
      out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
      options(nostack)
    );
    ret as isize
  }
}

pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 4i32 => _,  // SYS_write
      inlateout("r3") fd => ret,
      inlateout("r4") buf => _,
      inlateout("r5") count => _,
      out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
      options(nostack)
    );
    ret as isize
  }
}

pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 6i32 => _,  // SYS_close
      inlateout("r3") fd => ret,
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
    core::arch::asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 122i32 => _,  // SYS_newuname
      inlateout("r3") buf => ret,
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
    core::arch::asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 252i32 => _,  // SYS_statfs64
      inlateout("r3") path => ret,
      inlateout("r4") core::mem::size_of::<StatfsBuf>() => _,
      inlateout("r5") buf => _,
      out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_sysinfo(info: *mut SysInfo) -> i64 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 116_i32 => _, // __NR_sysinfo
      inlateout("r3") info => ret,
      out("r4") _, out("r5") _, out("r6") _, out("r7") _, out("r8") _,
      out("r9") _, out("r10") _, out("r11") _, out("r12") _,
      out("cr0") _,
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
      "sc",
      "bns+ 2f",
      "neg 3, 3",
      "2:",
      inlateout("r0") 223i32 => _,  // __NR_sched_getaffinity
      inlateout("r3") pid => ret,
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
    core::arch::asm!(
      "li 0, 1",  // SYS_exit
      "sc",
      in("r3") code,
      options(noreturn, nostack)
    );
  }
}
