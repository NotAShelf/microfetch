use std::{hint::black_box, mem::MaybeUninit};

use criterion::{Criterion, criterion_group, criterion_main};
use microfetch_lib::{
  StackWriter,
  UtsName,
  colors::{self, Colors},
  cpu,
  desktop,
  release,
  system,
  uptime,
};

fn main_benchmark(c: &mut Criterion) {
  let utsname = UtsName::uname().expect("Failed to get uname");
  let colors = Colors::new(false);

  c.bench_function("user_info", |b| {
    b.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut w = StackWriter::new(&mut buf);
      system::write_username_and_hostname(&mut w, &colors, &utsname);
      black_box(w.written());
    });
  });
  c.bench_function("os_name", |b| {
    b.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut w = StackWriter::new(&mut buf);
      let _ = release::write_os_pretty_name(&mut w);
      black_box(w.written());
    });
  });
  c.bench_function("kernel_version", |b| {
    b.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut w = StackWriter::new(&mut buf);
      release::write_system_info(&mut w, &utsname);
      black_box(w.written());
    });
  });
  c.bench_function("cpu_name", |b| {
    b.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut w = StackWriter::new(&mut buf);
      cpu::write_cpu_name(&mut w);
      black_box(w.written());
    });
  });
  c.bench_function("cpu_cores", |b| {
    b.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut w = StackWriter::new(&mut buf);
      let _ = cpu::write_cpu_cores(&mut w);
      black_box(w.written());
    });
  });
  c.bench_function("shell", |b| {
    b.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut w = StackWriter::new(&mut buf);
      system::write_shell(&mut w);
      black_box(w.written());
    });
  });
  c.bench_function("desktop", |b| {
    b.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut w = StackWriter::new(&mut buf);
      desktop::write_desktop_info(&mut w);
      black_box(w.written());
    });
  });
  c.bench_function("uptime", |b| {
    b.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut w = StackWriter::new(&mut buf);
      let _ = uptime::write_uptime(&mut w);
      black_box(w.written());
    });
  });
  c.bench_function("memory_usage", |b| {
    b.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut w = StackWriter::new(&mut buf);
      let _ = system::write_memory_usage(&mut w, &colors);
      black_box(w.written());
    });
  });
  c.bench_function("storage", |b| {
    b.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut w = StackWriter::new(&mut buf);
      let _ = system::write_root_disk_usage(&mut w, &colors);
      black_box(w.written());
    });
  });
  c.bench_function("colors", |b| {
    b.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut w = StackWriter::new(&mut buf);
      colors::write_dots(&mut w, &colors);
      black_box(w.written());
    });
  });
}

criterion_group!(benches, main_benchmark);
criterion_main!(benches);
