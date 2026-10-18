//! Syscall implementations for `mips64`.

use core::arch::asm;

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let ret: i32;
    asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "dsubu $2, $0, $2",
      "1:",
      in("$2") 5000_i64 + 2,  // SYS_open
      lateout("$2") ret,
      in("$4") path,
      in("$5") flags,
      in("$6") 0_i32,  // mode
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
    asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "dsubu $2, $0, $2",
      "1:",
      in("$2") 5000_i64 + 0,  // SYS_read
      lateout("$2") ret,
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
    asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "dsubu $2, $0, $2",
      "1:",
      in("$2") 5000_i64 + 1,  // SYS_write
      lateout("$2") ret,
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
    asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "dsubu $2, $0, $2",
      "1:",
      in("$2") 5000_i64 + 3,  // SYS_close
      lateout("$2") ret,
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
    asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "dsubu $2, $0, $2",
      "1:",
      in("$2") 5000_i64 + 61,  // SYS_newuname
      lateout("$2") ret,
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
    asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "dsubu $2, $0, $2",
      "1:",
      in("$2") 5000_i64 + 134,  // SYS_statfs
      lateout("$2") ret,
      in("$4") path,
      in("$5") buf,
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
    let ret: i64;
    asm!(
      "syscall",
      "beqz $7, 1f",
      "nop",
      "dsubu $2, $0, $2",
      "1:",
      in("$2") 5000_i64 + 97,  // SYS_sysinfo
      lateout("$2") ret,
      in("$4") info,
      lateout("$7") _,
      lateout("$8") _, lateout("$9") _, lateout("$10") _, lateout("$11") _,
      lateout("$12") _, lateout("$13") _, lateout("$14") _, lateout("$15") _,
      lateout("$24") _, lateout("$25") _,
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
      "syscall",
      in("$2") 5000_i64 + 196,  // __NR_sched_getaffinity
      lateout("$2") ret,
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
    asm!(
      "syscall",
      in("$2") 5000_i64 + 58,  // SYS_exit
      in("$4") code,
      options(noreturn, nostack)
    );
  }
}
