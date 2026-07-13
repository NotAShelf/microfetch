//! Syscall implementations for `x86_64`.

use super::{StatfsBuf, SysInfo, UtsNameBuf};

pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe {
    let fd: i64;
    core::arch::asm!(
      "syscall",
      in("rax") 2i64,  // SYS_open
      in("rdi") path,
      in("rsi") flags,
      in("rdx") 0i32,  // mode (not used for reading)
      lateout("rax") fd,
      lateout("rcx") _,
      lateout("r11") _,
      options(nostack)
    );
    #[allow(clippy::cast_possible_truncation)]
    {
      fd as i32
    }
  }
}

pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "syscall",
      in("rax") 0i64,  // SYS_read
      in("rdi") fd,
      in("rsi") buf,
      in("rdx") count,
      lateout("rax") ret,
      lateout("rcx") _,
      lateout("r11") _,
      options(nostack)
    );

    #[allow(clippy::cast_possible_truncation)]
    {
      ret as isize
    }
  }
}

pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "syscall",
      in("rax") 1i64,  // SYS_write
      in("rdi") fd,
      in("rsi") buf,
      in("rdx") count,
      lateout("rax") ret,
      lateout("rcx") _,
      lateout("r11") _,
      options(nostack)
    );

    #[allow(clippy::cast_possible_truncation)]
    {
      ret as isize
    }
  }
}

pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe {
    let ret: i64;
    core::arch::asm!(
      "syscall",
      in("rax") 3i64,  // SYS_close
      in("rdi") fd,
      lateout("rax") ret,
      lateout("rcx") _,
      lateout("r11") _,
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
      "syscall",
      in("rax") 63i64,  // SYS_uname
      in("rdi") buf,
      lateout("rax") ret,
      lateout("rcx") _,
      lateout("r11") _,
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
      "syscall",
      in("rax") 137i64,  // SYS_statfs
      in("rdi") path,
      in("rsi") buf,
      lateout("rax") ret,
      lateout("rcx") _,
      lateout("r11") _,
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
      "syscall",
      in("rax") 99_i64, // __NR_sysinfo
      in("rdi") info,
      out("rcx") _,
      out("r11") _,
      lateout("rax") ret,
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
      "syscall",
      in("rax") 204i64,  // __NR_sched_getaffinity
      in("rdi") pid,
      in("rsi") mask_size,
      in("rdx") mask,
      lateout("rax") ret,
      lateout("rcx") _,
      lateout("r11") _,
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
      "syscall",
      in("rax") 60i64,  // SYS_exit
      in("rdi") code,
      options(noreturn, nostack)
    );
  }
}
