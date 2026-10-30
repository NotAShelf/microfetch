use std::{env, fs, path::Path};

fn main() {
  let mut constants: Vec<String> = Vec::new();

  constants.push(format!(
    "pub const COLOR_LOGO1: &'static str = \"{}\";",
    match env::var("COLOR_LOGO1") {
      Ok(color) => {
        let r =
          u32::from_str_radix(&color[1..3], 16).expect("Invalid COLOR_LOGO1");
        let g =
          u32::from_str_radix(&color[3..5], 16).expect("Invalid COLOR_LOGO1");
        let b =
          u32::from_str_radix(&color[5..7], 16).expect("Invalid COLOR_LOGO1");
        format!("\\x1b[38;2;{r};{g};{b};m")
      },
      Err(_) => "\x1b[34m".to_string(),
    }
  ));

  constants.push(format!(
    "pub const COLOR_LOGO2: &'static str = \"{}\";",
    match env::var("COLOR_LOGO2") {
      Ok(color) => {
        let r =
          u32::from_str_radix(&color[1..3], 16).expect("Invalid COLOR_LOGO2");
        let g =
          u32::from_str_radix(&color[3..5], 16).expect("Invalid COLOR_LOGO2");
        let b =
          u32::from_str_radix(&color[5..7], 16).expect("Invalid COLOR_LOGO2");
        format!("\\x1b[38;2;{r};{g};{b};m")
      },
      Err(_) => "\x1b[36m".to_string(),
    }
  ));

  constants.push(format!(
    "pub const COLOR_ICON: &'static str = \"{}\";",
    match env::var("COLOR_ICON") {
      Ok(color) => {
        let r =
          u32::from_str_radix(&color[1..3], 16).expect("Invalid COLOR_ICON");
        let g =
          u32::from_str_radix(&color[3..5], 16).expect("Invalid COLOR_ICON");
        let b =
          u32::from_str_radix(&color[5..7], 16).expect("Invalid COLOR_ICON");
        format!("\\x1b[38;2;{r};{g};{b};m")
      },
      Err(_) => "\x1b[36m".to_string(),
    }
  ));

  constants.push(format!(
    "pub const COLOR_KEY: &'static str = \"{}\";",
    match env::var("COLOR_KEY") {
      Ok(color) => {
        let r =
          u32::from_str_radix(&color[1..3], 16).expect("Invalid COLOR_KEY");
        let g =
          u32::from_str_radix(&color[3..5], 16).expect("Invalid COLOR_KEY");
        let b =
          u32::from_str_radix(&color[5..7], 16).expect("Invalid COLOR_KEY");
        format!("\\x1b[38;2;{r};{g};{b};m")
      },
      Err(_) => "\x1b[34m".to_string(),
    }
  ));

  let val = match env::var("COLOR_VALUE") {
    Ok(color) => {
      let r =
        u32::from_str_radix(&color[1..3], 16).expect("Invalid COLOR_VALUE");
      let g =
        u32::from_str_radix(&color[3..5], 16).expect("Invalid COLOR_VALUE");
      let b =
        u32::from_str_radix(&color[5..7], 16).expect("Invalid COLOR_VALUE");
      format!("\\x1b[38;2;{r};{g};{b};m")
    },
    Err(_) => "".to_string(),
  };
  constants.push(format!("pub const COLOR_VAL: &'static str = \"{}\";", val));

  let out_dir = env::var("OUT_DIR").unwrap();
  let path1 = Path::new(&out_dir).join("colors_config.rs");
  let code = constants.join("\n");
  fs::write(path1, code).unwrap();
  let path2 = Path::new(&out_dir).join("color_helpers.rs");
  fs::write(
    path2,
    format!("pub const RPAREN: &'static str = \"{})\";\n", val),
  )
  .unwrap();
}
