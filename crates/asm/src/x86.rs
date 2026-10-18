//! Syscall implementations for `x86`.

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "push ebx",
      "mov ebx, {arg1:e}",
      "int 0x80",
      "pop ebx",
      arg1 = in(reg) path,
      inlateout("eax") 5i32 => ret,  // SYS_open
      in("ecx") flags,
      in("edx") 0i32,  // mode
    );
    ret
  }
}

pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    core::arch::asm!(
      "push ebx",
      "mov ebx, {arg1:e}",
      "int 0x80",
      "pop ebx",
      arg1 = in(reg) fd,
      inlateout("eax") 3i32 => ret,  // SYS_read
      in("ecx") buf,
      in("edx") count,
    );
    ret
  }
}

pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe {
    let ret: isize;
    core::arch::asm!(
      "push ebx",
      "mov ebx, {arg1:e}",
      "int 0x80",
      "pop ebx",
      arg1 = in(reg) fd,
      inlateout("eax") 4i32 => ret,  // SYS_write
      in("ecx") buf,
      in("edx") count,
    );
    ret
  }
}

pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "push ebx",
      "mov ebx, {arg1:e}",
      "int 0x80",
      "pop ebx",
      arg1 = in(reg) fd,
      inlateout("eax") 6i32 => ret,  // SYS_close
    );
    ret
  }
}

pub unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "push ebx",
      "mov ebx, {arg1:e}",
      "int 0x80",
      "pop ebx",
      arg1 = in(reg) buf,
      inlateout("eax") 122i32 => ret,  // SYS_newuname
    );
    ret
  }
}

pub unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "push ebx",
      "mov ebx, {arg1:e}",
      "int 0x80",
      "pop ebx",
      arg1 = in(reg) path,
      inlateout("eax") 268i32 => ret,  // SYS_statfs64
      in("ecx") core::mem::size_of::<StatfsBuf>(),
      in("edx") buf,
    );
    ret
  }
}

pub unsafe fn sys_sysinfo(info: *mut SysInfo) -> i64 {
  unsafe {
    let ret: i32;
    core::arch::asm!(
      "push ebx",
      "mov ebx, {arg1:e}",
      "int 0x80",
      "pop ebx",
      arg1 = in(reg) info,
      inlateout("eax") 116_i32 => ret, // __NR_sysinfo
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
      "push ebx",
      "mov ebx, {arg1:e}",
      "int 0x80",
      "pop ebx",
      arg1 = in(reg) pid,
      inlateout("eax") 242i32 => ret,  // __NR_sched_getaffinity
      in("ecx") mask_size,
      in("edx") mask,
    );
    ret
  }
}

pub unsafe fn sys_exit(code: i32) -> ! {
  unsafe {
    core::arch::asm!(
      "mov ebx, {code:e}",
      "int 0x80",
      code = in(reg) code,
      in("eax") 1i32,  // SYS_exit
      options(noreturn)
    );
  }
}
