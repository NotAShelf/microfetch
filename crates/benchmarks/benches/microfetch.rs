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

#[expect(
  clippy::expect_used,
  reason = "the benchmarks cannot run without uname"
)]
fn main_benchmark(criterion: &mut Criterion) {
  let utsname = UtsName::uname().expect("Failed to get uname");
  let colors = Colors::new(false);

  criterion.bench_function("user_info", |bencher| {
    bencher.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut writer = StackWriter::new(&mut buf);
      system::write_username_and_hostname(&mut writer, &colors, &utsname);
      black_box(writer.written());
    });
  });
  criterion.bench_function("os_name", |bencher| {
    bencher.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut writer = StackWriter::new(&mut buf);
      release::write_os_pretty_name(&mut writer);
      black_box(writer.written());
    });
  });
  criterion.bench_function("kernel_version", |bencher| {
    bencher.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut writer = StackWriter::new(&mut buf);
      release::write_system_info(&mut writer, &utsname);
      black_box(writer.written());
    });
  });
  criterion.bench_function("cpu_name", |bencher| {
    bencher.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut writer = StackWriter::new(&mut buf);
      cpu::write_cpu_name(&mut writer);
      black_box(writer.written());
    });
  });
  criterion.bench_function("cpu_cores", |bencher| {
    bencher.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut writer = StackWriter::new(&mut buf);
      cpu::write_cpu_cores(&mut writer);
      black_box(writer.written());
    });
  });
  criterion.bench_function("shell", |bencher| {
    bencher.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut writer = StackWriter::new(&mut buf);
      system::write_shell(&mut writer);
      black_box(writer.written());
    });
  });
  criterion.bench_function("desktop", |bencher| {
    bencher.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut writer = StackWriter::new(&mut buf);
      desktop::write_desktop_info(&mut writer);
      black_box(writer.written());
    });
  });
  criterion.bench_function("uptime", |bencher| {
    bencher.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut writer = StackWriter::new(&mut buf);
      uptime::write(&mut writer);
      black_box(writer.written());
    });
  });
  criterion.bench_function("memory_usage", |bencher| {
    bencher.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut writer = StackWriter::new(&mut buf);
      system::write_memory_usage(&mut writer, &colors);
      black_box(writer.written());
    });
  });
  criterion.bench_function("storage", |bencher| {
    bencher.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut writer = StackWriter::new(&mut buf);
      system::write_root_disk_usage(&mut writer, &colors);
      black_box(writer.written());
    });
  });
  criterion.bench_function("colors", |bencher| {
    bencher.iter(|| {
      let mut buf = [MaybeUninit::uninit(); 256];
      let mut writer = StackWriter::new(&mut buf);
      colors::write_dots(&mut writer, &colors);
      black_box(writer.written());
    });
  });
}

criterion_group!(benches, main_benchmark);
criterion_main!(benches);
