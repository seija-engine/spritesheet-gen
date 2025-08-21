pub mod max_rect;
use std::fs::{self};
use image::{RgbaImage};
use serde_json::{Value,Map,Number};
use std::path::{Path};


pub struct SpriteSheetGenConfig {
    dir:String,
    width:u32,
    height:u32,
    padding:u32,
    is_rotation:bool,
    write_desc_fn:Box<dyn Fn(&String,&SpriteSheetGenConfig,&Vec<(String,max_rect::Rect)>)>,
    out_file:Option<String>,
    sprite_list:Vec<String>,
}

impl Default for SpriteSheetGenConfig {
    fn default() -> Self {
        SpriteSheetGenConfig {
            dir : String::from("./"),
            width: 1024,
            height: 1024,
            is_rotation: false,
            padding:2,
            write_desc_fn:Box::new(write_default_json),
            out_file: None,
            sprite_list:vec![]
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
    pub fn set_padding(&mut self,padding:u32) {
        self.padding = padding;
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

    pub fn set_sprite_list(&mut self, sprite_list: Vec<String>) {
        self.sprite_list = sprite_list;
    }
}

fn process_image(
    path: &Path,
    out_image: &mut RgbaImage,
    max_rect: &mut max_rect::MaxRectsBinPack,
    padding: u32,
    writed_list: &mut Vec<(String, max_rect::Rect)>
) {
    if let Ok(img) = image::open(path) {
        let mut rgba_image = img.to_rgba();
        let (w, h) = rgba_image.dimensions();
        let padding_w = w + padding * 2;
        let padding_h = h + padding * 2;
        let mut insert_rect = max_rect.insert(
            padding_w as i32,
            padding_h as i32,
            max_rect::FreeRectChoiceHeuristic::BestAreaFit
        );
        if insert_rect.height <= 0 {
            eprintln!("image to small, can't place {:?}", path);
            return;
        }
        if insert_rect.width != padding_w as i32 {
            rgba_image = image::imageops::rotate90(&rgba_image);
        }
        image::imageops::overlay(
            out_image,
            &rgba_image,
            insert_rect.x as u32 + padding,
            insert_rect.y as u32 + padding
        );
        let may_file_name = path
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .map(|s| String::from(s));
        if let Some(file_name) = may_file_name {
            insert_rect.x += padding as i32;
            insert_rect.y += padding as i32;
            insert_rect.width -= (padding as i32) * 2;
            insert_rect.height -= (padding as i32) * 2;
            writed_list.push((file_name, insert_rect));
        } else {
            eprintln!("can't get filename {:?}", path);
        }
    } else {
        eprintln!("can't open image: {:?}", path);
    }
}

pub fn sprite_sheet_gen(cfg:SpriteSheetGenConfig) -> Result<bool,String> {
    let mut out_image:RgbaImage = image::ImageBuffer::new(cfg.width,cfg.height);
    let mut max_rect = max_rect::MaxRectsBinPack::new(cfg.width, cfg.height,cfg.is_rotation);
    let mut writed_list:Vec<(String,max_rect::Rect)> = Vec::new();
    
    if cfg.sprite_list.is_empty() {
        // 如果 sprite_list 为空，使用原来的逻辑遍历目录
        let read_dir:fs::ReadDir = fs::read_dir(&cfg.dir).map_err(|_| String::from("dir not found"))?;
        for may_item in read_dir {
            if let Ok(item) = may_item {
                let path = item.path();
                if path.is_dir() {
                    continue;
                }
                process_image(&path, &mut out_image, &mut max_rect, cfg.padding, &mut writed_list);
            }
        }
    } else {
        // 如果 sprite_list 不为空，使用指定的文件列表
        for sprite_file in &cfg.sprite_list {
            let path = Path::new(sprite_file);
            process_image(&path, &mut out_image, &mut max_rect, cfg.padding, &mut writed_list);
        }
    }
    let def_name = Path::new(&cfg.dir).file_name().and_then(|os_str| os_str.to_str()).map(|s| String::from(s));
    let out_path = cfg.out_file.clone().unwrap_or(def_name.unwrap_or(String::from("default")));
    out_image.save(out_path.clone() + ".png").map_err(|_| String::from("save image error"))?;
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
   
    #[test]
    fn test_insert() {
        let mut max_rect = MaxRectsBinPack::new(1024,1024, false);
        let test_data =  [(100,100),(32,32),(32,12),(50,50),(27,15),(128,45),(1000,198),(44,89)];
        for tp in test_data.iter() {
            max_rect.insert(tp.0, tp.1, FreeRectChoiceHeuristic::BestAreaFit);
        }
        dbg!(&max_rect);
        draw_debug_rect(&max_rect)
    }

    fn draw_debug_rect(max_rect:&MaxRectsBinPack) {
        use image::DynamicImage;
        use image::{Rgba};
        use  image::{GenericImage};
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
