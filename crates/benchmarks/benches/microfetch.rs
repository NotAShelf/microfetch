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
      let mut buf = [0u8; 256];
      let mut w = StackWriter::new(&mut buf);
      system::write_username_and_hostname(&mut w, &colors, &utsname);
    });
  });
  c.bench_function("os_name", |b| {
    b.iter(|| {
      let mut buf = [0u8; 256];
      let mut w = StackWriter::new(&mut buf);
      let _ = release::write_os_pretty_name(&mut w);
    });
  });
  c.bench_function("kernel_version", |b| {
    b.iter(|| {
      let mut buf = [0u8; 256];
      let mut w = StackWriter::new(&mut buf);
      release::write_system_info(&mut w, &utsname);
    });
  });
  c.bench_function("shell", |b| {
    b.iter(|| {
      let mut buf = [0u8; 256];
      let mut w = StackWriter::new(&mut buf);
      system::write_shell(&mut w);
    });
  });
  c.bench_function("desktop", |b| {
    b.iter(|| {
      let mut buf = [0u8; 256];
      let mut w = StackWriter::new(&mut buf);
      desktop::write_desktop_info(&mut w);
    });
  });
  c.bench_function("uptime", |b| {
    b.iter(|| {
      let mut buf = [0u8; 256];
      let mut w = StackWriter::new(&mut buf);
      let _ = uptime::write_uptime(&mut w);
    });
  });
  c.bench_function("memory_usage", |b| {
    b.iter(|| {
      let mut buf = [0u8; 256];
      let mut w = StackWriter::new(&mut buf);
      let _ = system::write_memory_usage(&mut w, &colors);
    });
  });
  c.bench_function("storage", |b| {
    b.iter(|| {
      let mut buf = [0u8; 256];
      let mut w = StackWriter::new(&mut buf);
      let _ = system::write_root_disk_usage(&mut w, &colors);
    });
  });
  c.bench_function("colors", |b| {
    b.iter(|| {
      let mut buf = [0u8; 256];
      let mut w = StackWriter::new(&mut buf);
      colors::write_dots(&mut w, &colors);
    });
  });
}

criterion_group!(benches, main_benchmark);
criterion_main!(benches);
