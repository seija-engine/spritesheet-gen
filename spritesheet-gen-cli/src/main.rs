use clap::{App,Arg};
use spritesheet_gen::{SpriteSheetGenConfig,sprite_sheet_gen};
fn main() {
    let matchs = App::new("spritesheet-gen")
                    .version("0.1.0")
                    .arg(Arg::with_name("dir").short("d").long("dir").value_name("Folder").help("source image folder").required(false))
                    .arg(Arg::with_name("width").short("w").long("width").value_name("Width").help("image width").required(false))
                    .arg(Arg::with_name("height").short("h").long("height").value_name("Height").help("image height").required(false))
                    .arg(Arg::with_name("outfile").short("o").long("outfile").value_name("OutFile").help("output file name").required(false))
                    .arg(Arg::with_name("rotation").short("r").long("rotation").value_name("Rotation").help("is rotation").required(false))
                    .get_matches();
    let dir = matchs.value_of("dir").unwrap_or("./");
    let mut cfg = SpriteSheetGenConfig::default();
    cfg.set_dir(dir);
    if let Some(w) = matchs.value_of("width") {
        cfg.set_width(w.parse().unwrap_or(1024));
    }
    if let Some(h) = matchs.value_of("height") {
        cfg.set_height(h.parse().unwrap_or(1024));
    }
    if let Some(out_name) = matchs.value_of("outname") {
        cfg.set_out_file(out_name);   
    }
    if let Some(r) = matchs.value_of("rotation") {
        cfg.set_is_rotation(r.parse().unwrap_or(false));   
    }
    sprite_sheet_gen(cfg).unwrap();
}
