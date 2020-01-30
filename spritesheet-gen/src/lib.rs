pub mod max_rect;
use std::fs::{self};
use image::{RgbaImage};
use serde_json::{Value,Map,Number};
use std::path::{Path};


pub struct SpriteSheetGenConfig {
    dir:String,
    width:u32,
    height:u32,
    is_rotation:bool,
    write_desc_fn:Box<dyn Fn(&String,&SpriteSheetGenConfig,&Vec<(String,max_rect::Rect)>)>,
    out_file:Option<String>
}

impl Default for SpriteSheetGenConfig {
    fn default() -> Self {
        SpriteSheetGenConfig {
            dir : String::from("./"),
            width: 1024,
            height: 1024,
            is_rotation: false,
            write_desc_fn:Box::new(write_default_json),
            out_file: None
        }
    }
}

impl SpriteSheetGenConfig {
    pub fn set_dir(&mut self,path:&str)  {
        self.dir = String::from(path);
    }
    pub fn set_size(&mut self,w:u32,h:u32) {
        self.width = w;
        self.height = h;
    }
    pub fn set_is_rotation(&mut self,b:bool) {
        self.is_rotation = b;
    }
    pub fn set_width(&mut self,w:u32) {
        self.width = w;
    }
    pub fn set_height(&mut self,h:u32) {
        self.height = h;
    }

    pub fn set_out_file(&mut self,out_file:&str) {
        self.out_file = Some(String::from(out_file));
    }
}

pub fn sprite_sheet_gen(cfg:SpriteSheetGenConfig) -> Result<bool,String> {
    let mut gen_image:RgbaImage = image::ImageBuffer::new(cfg.width,cfg.height);
    let mut max_rect = max_rect::MaxRectsBinPack::new(cfg.width, cfg.height,cfg.is_rotation);
    let read_dir:fs::ReadDir = fs::read_dir(&cfg.dir).map_err(|_| String::from("dir not found"))?;
    let mut writed_list:Vec<(String,max_rect::Rect)> = Vec::new();
    for may_item in read_dir {
        if let Ok(item) = may_item {
          let path = item.path();
          if path.is_dir() {
              continue;
          }
          if let Ok(img) = image::open(&path) {
              let mut rgba_image = img.to_rgba();
              let (w,h) = rgba_image.dimensions();
              let insert_rect = max_rect.insert(w as i32,h as i32,max_rect::FreeRectChoiceHeuristic::BestAreaFit);
              if insert_rect.height <= 0 {
                  eprintln!("image to small, can't place {:?}",path);
                  continue;
              }
              if insert_rect.width != w as i32 {
                rgba_image = image::imageops::rotate90(&rgba_image);
              }
              image::imageops::overlay(&mut gen_image, &rgba_image, insert_rect.x as u32, insert_rect.y as u32);
              let may_file_name = path.as_path().file_stem().and_then(|os_str| os_str.to_str()).map(|s| String::from(s));
              if let Some(file_name) = may_file_name {
                writed_list.push((file_name,insert_rect));
              } else {
                eprintln!("can't get filename {:?}",path);
              }
          }
        }
    }
    let def_name = Path::new(&cfg.dir).file_name().and_then(|os_str| os_str.to_str()).map(|s| String::from(s));
    let out_path = cfg.out_file.clone().unwrap_or(def_name.unwrap_or(String::from("default")));
    gen_image.save(out_path.clone() + ".png").map_err(|_| String::from("save image error"))?;
    (cfg.write_desc_fn)(&out_path,&cfg,&writed_list);
    Ok(true)
}

fn write_default_json(out_path:&String,cfg:&SpriteSheetGenConfig,data_list:&Vec<(String,max_rect::Rect)>) {
    let mut meta_map:Map<String,Value> = Map::default();
    let tex_name = Path::new(out_path).file_stem().and_then(|os_str| os_str.to_str()).map(|s| String::from(s));
    meta_map.insert(String::from("texture"), Value::String(tex_name.unwrap() + ".png"));
    meta_map.insert(String::from("width"), Value::Number(serde_json::Number::from(cfg.width)));
    meta_map.insert(String::from("height"), Value::Number(serde_json::Number::from(cfg.height)));
    let mut sprite_list:Vec<Value>  = Vec::new();
    for item in data_list {
        let mut sprite_map = Map::default();
        sprite_map.insert(String::from("name"),Value::String(String::from(item.0.clone())));
        sprite_map.insert(String::from("x"),Value::Number(Number::from(item.1.x)));
        sprite_map.insert(String::from("y"),Value::Number(Number::from(item.1.y)));
        sprite_map.insert(String::from("width"),Value::Number(Number::from(item.1.width)));
        sprite_map.insert(String::from("height"),Value::Number(Number::from(item.1.height)));
        sprite_list.push(Value::Object(sprite_map));
    }
    let mut out_json_map:Map<String,Value> = Map::default();
    out_json_map.insert(String::from("meta"), Value::Object(meta_map));
    out_json_map.insert(String::from("sprites"),Value::Array(sprite_list));
    let json_str = serde_json::to_string_pretty(&Value::Object(out_json_map)).unwrap();
    std::fs::write(out_path.clone() + ".json", json_str).unwrap();
}

#[cfg(test)]
mod tests {
    use crate::max_rect::{FreeRectChoiceHeuristic,MaxRectsBinPack};
    use crate::{sprite_sheet_gen,SpriteSheetGenConfig};
   

    #[test]
    fn test_insert() {
        let mut max_rect = MaxRectsBinPack::new(1024,1024, false);
        let test_data =  [(100,100),(32,32),(50,50),(27,15),(128,45),(1000,198),(44,89)];
        for tp in test_data.iter() {
            max_rect.insert(tp.0, tp.1, FreeRectChoiceHeuristic::BestAreaFit);
        }
        dbg!(&max_rect);
        draw_debug_rect(&max_rect)
    }

    fn draw_debug_rect(max_rect:&MaxRectsBinPack) {
        use image::DynamicImage;
        use image::{Rgba};
        use  image::{GenericImage,GenericImageView};
        use imageproc::drawing;
        use imageproc::rect::{Rect};
        let mut new_image = DynamicImage::new_rgba8(max_rect.width(),max_rect.height());
        let used_list = max_rect.used_rect();
        for use_rect in used_list {
          let color = Rgba([255,0,0,255]);
           drawing::draw_hollow_rect_mut(&mut new_image,Rect::at(use_rect.x, use_rect.y)
                                                     .of_size(use_rect.width as u32, use_rect.height as u32), color);
        }

        for free_rect in max_rect.free_rect() {
            let color = Rgba([105,0,255,255]);
             drawing::draw_filled_rect_mut(&mut new_image,Rect::at(free_rect.x, free_rect.y)
                                                       .of_size(free_rect.width as u32, free_rect.height as u32), color);
          }
        new_image.put_pixel(0, 0, image::Rgba([1,1,1,1]));
        new_image.save("test2.png").unwrap();
    }
}
