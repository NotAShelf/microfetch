// Pack logo rows into a marker-delimited stream:
// 1 = blue, 2 = cyan, 0 = row separator.
use std::{env, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
  let rows: &[&[(u8, &str)]] = &[
    &[(0, "⠀⠀⠀⠀⠀⠀⢼⣿⣄⠀⠀⠀"), (1, "⠹⣿⣷⡀⠀⣠⣿⡧⠀⠀⠀⠀⠀⠀")],
    &[(0, "⠀⠀⠀⠀⠀⠀⠈⢿⣿⣆⠀⠀⠀"), (1, "⠘⣿⣿⣴⣿⡿⠁⠀⠀⠀⠀⠀⠀")],
    &[(0, "⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡜"), (1, "⢿⣿⣟⠀⠀⠀"), (0, "⢀⡄⠀⠀⠀")],
    &[
      (0, "⠀⠀⠀⠉⠉⠉⠉"),
      (1, "⣩⣭⡭"),
      (0, "⠉⠉⠉⠉⠉"),
      (1, "⠈⢿⣿⣆⠀"),
      (0, "⢠⣿⣿⠂⠀⠀"),
    ],
    &[(1, "⠀⠀⠀⠀⠀⠀⣼⣿⡟⠀⠀⠀⠀⠀⠀⠀⠀⢻⡟"), (0, "⣡⣿⣿⠃⠀⠀⠀")],
    &[(1, "⢸⣿⣿⣿⣿⣿⣿⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"), (0, "⣰⣿⣿⣿⣿⣿⣿⡇")],
    &[(1, "⠀⠀⠀⢠⣿⣿⢋"), (0, "⣼⣧⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⡟⠀⠀⠀⠀⠀⠀")],
    &[
      (1, "⠀⠀⠠⣿⣿⠃⠀"),
      (0, "⠹⣿⣷⡀"),
      (1, "⣀⣀⣀⣀⣀"),
      (0, "⣚⣛⣋"),
      (1, "⣀⣀⣀⣀⠀⠀⠀"),
    ],
    &[(1, "⠀⠀⠀⠘⠁⠀⠀⠀"), (0, "⣽⣿⣷⡜"), (1, "⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠀")],
    &[(0, "⠀⠀⠀⠀⠀⠀⢀⣾⣿⠟⣿⣿⡄⠀⠀⠀"), (1, "⠹⣿⣷⡀⠀⠀⠀⠀⠀⠀")],
    &[(0, "⠀⠀⠀⠀⠀⠀⢺⣿⠋⠀⠈⢿⣿⣆⠀⠀⠀"), (1, "⠙⣿⡗⠀⠀⠀⠀⠀⠀")],
  ];

  let mut out = Vec::new();
  for (row_index, row) in rows.iter().enumerate() {
    for &(color, text) in *row {
      out.push(color + 1);
      out.extend_from_slice(text.as_bytes());
    }
    if row_index < rows.len() - 1 {
      out.push(0);
    }
  }

  let out_dir = env::var("OUT_DIR")?;
  fs::write(format!("{out_dir}/logo.bin"), &out)?;
  Ok(())
}
