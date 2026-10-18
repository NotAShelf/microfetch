//! Syscall implementations for `mips`.

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "subu $2, $0, $2",
      "1:",
      inlateout("$2") 4000i32 + 5 => ret,  // SYS_open
      in("$4") path,
      in("$5") flags,
      in("$6") 0i32,  // mode
      lateout("$7") _,
      lateout("$8") _, lateout("$9") _, lateout("$10") _, lateout("$11") _,
      lateout("$12") _, lateout("$13") _, lateout("$14") _, lateout("$15") _,
      lateout("$24") _, lateout("$25") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    core::arch::asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "subu $2, $0, $2",
      "1:",
      inlateout("$2") 4000i32 + 3 => ret,  // SYS_read
      in("$4") fd,
      in("$5") buf,
      in("$6") count,
      lateout("$7") _,
      lateout("$8") _, lateout("$9") _, lateout("$10") _, lateout("$11") _,
      lateout("$12") _, lateout("$13") _, lateout("$14") _, lateout("$15") _,
      lateout("$24") _, lateout("$25") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    core::arch::asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "subu $2, $0, $2",
      "1:",
      inlateout("$2") 4000i32 + 4 => ret,  // SYS_write
      in("$4") fd,
      in("$5") buf,
      in("$6") count,
      lateout("$7") _,
      lateout("$8") _, lateout("$9") _, lateout("$10") _, lateout("$11") _,
      lateout("$12") _, lateout("$13") _, lateout("$14") _, lateout("$15") _,
      lateout("$24") _, lateout("$25") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "subu $2, $0, $2",
      "1:",
      inlateout("$2") 4000i32 + 6 => ret,  // SYS_close
      in("$4") fd,
      lateout("$7") _,
      lateout("$8") _, lateout("$9") _, lateout("$10") _, lateout("$11") _,
      lateout("$12") _, lateout("$13") _, lateout("$14") _, lateout("$15") _,
      lateout("$24") _, lateout("$25") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "subu $2, $0, $2",
      "1:",
      inlateout("$2") 4000i32 + 122 => ret,  // SYS_newuname
      in("$4") buf,
      lateout("$7") _,
      lateout("$8") _, lateout("$9") _, lateout("$10") _, lateout("$11") _,
      lateout("$12") _, lateout("$13") _, lateout("$14") _, lateout("$15") _,
      lateout("$24") _, lateout("$25") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "subu $2, $0, $2",
      "1:",
      inlateout("$2") 4000i32 + 255 => ret,  // SYS_statfs64
      in("$4") path,
      in("$5") core::mem::size_of::<StatfsBuf>(),
      in("$6") buf,
      lateout("$7") _,
      lateout("$8") _, lateout("$9") _, lateout("$10") _, lateout("$11") _,
      lateout("$12") _, lateout("$13") _, lateout("$14") _, lateout("$15") _,
      lateout("$24") _, lateout("$25") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_sysinfo(info: *mut SysInfo) -> i64 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "subu $2, $0, $2",
      "1:",
      inlateout("$2") 4000_i32 + 116 => ret,  // __NR_sysinfo
      in("$4") info,
      lateout("$7") _,
      lateout("$8") _, lateout("$9") _, lateout("$10") _, lateout("$11") _,
      lateout("$12") _, lateout("$13") _, lateout("$14") _, lateout("$15") _,
      lateout("$24") _, lateout("$25") _,
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
      "syscall",
      "beqz $7, 1f",
      "nop",
      "subu $2, $0, $2",
      "1:",
      inlateout("$2") 4000i32 + 240 => ret,  // __NR_sched_getaffinity
      in("$4") pid,
      in("$5") mask_size,
      in("$6") mask,
      lateout("$7") _,
      lateout("$8") _, lateout("$9") _, lateout("$10") _, lateout("$11") _,
      lateout("$12") _, lateout("$13") _, lateout("$14") _, lateout("$15") _,
      lateout("$24") _, lateout("$25") _,
      options(nostack)
    );
    ret
  }
}

pub unsafe fn sys_exit(code: i32) -> ! {
  unsafe {
    core::arch::asm!(
      "syscall",
      in("$2") 4000i32 + 1,  // SYS_exit
      in("$4") code,
      options(noreturn, nostack)
    );
  }
}
