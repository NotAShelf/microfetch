use criterion::{criterion_group, criterion_main, Criterion};
use microfetch_lib::colors::print_dots;
use microfetch_lib::desktop::get_desktop_info;
use microfetch_lib::release::{get_os_pretty_name, get_system_info};
use microfetch_lib::system::{
    get_memory_usage, get_root_disk_usage, get_shell, get_username_and_hostname,
};
use microfetch_lib::uptime::get_current;

fn main_benchmark(c: &mut Criterion) {
    let utsname = nix::sys::utsname::uname().expect("lol");
    c.bench_function("user_info", |b| {
        b.iter(|| get_username_and_hostname(&utsname))
    });
    c.bench_function("os_name", |b| b.iter(get_os_pretty_name));
    c.bench_function("kernel_version", |b| b.iter(|| get_system_info(&utsname)));
    c.bench_function("shell", |b| b.iter(get_shell));

    c.bench_function("desktop", |b| b.iter(get_desktop_info));
    c.bench_function("uptime", |b| b.iter(get_current));
    c.bench_function("memory_usage", |b| b.iter(get_memory_usage));
    c.bench_function("storage", |b| b.iter(get_root_disk_usage));
    c.bench_function("colors", |b| b.iter(print_dots));
}

criterion_group!(benches, main_benchmark);
criterion_main!(benches);
