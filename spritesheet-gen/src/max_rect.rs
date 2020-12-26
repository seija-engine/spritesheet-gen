#[derive(Debug,Clone)]
pub struct Rect {
   pub x:i32,
   pub y:i32,
   pub width:i32,
   pub height:i32
}

impl Default for Rect {
    fn default() -> Rect {
        Rect {x:0,y:0,width:0,height:0}
    }
}

pub enum FreeRectChoiceHeuristic {
    BestShortSideFit,
    BestLongSideFit,
    BestAreaFit,
    BottomLeftRule,
    ContactPointRule
}

#[derive(Debug)]
pub struct MaxRectsBinPack {
    width:u32,
    height:u32,
    allow_rotations:bool,
    used_rect:Vec<Rect>,
    free_rect:Vec<Rect>
}

impl Default for MaxRectsBinPack {
    fn default() -> Self {
        MaxRectsBinPack {
            width:0,
            height:0,
            allow_rotations:true,
            used_rect:Vec::new(),
            free_rect:Vec::new()
        }
    }
}

impl MaxRectsBinPack {
    pub fn new(width:u32,height:u32,rotations:bool) -> Self {
        let mut max_rect = MaxRectsBinPack::default();
        max_rect.init(width, height, rotations);
        max_rect   
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn free_rect(&self) -> &Vec<Rect> {
        &self.free_rect
    }

    pub fn used_rect(&self) -> &Vec<Rect> {
        &self.used_rect
    }

    pub fn init(&mut self,width:u32,height:u32,rotations:bool) {
        self.width = width;
        self.height = height;
        self.allow_rotations = rotations;
        self.free_rect.clear();
        self.used_rect.clear();

        self.free_rect.push(Rect {x:0,y:0,width:width as i32,height:height as i32});
    }

    pub fn insert(&mut self,width:i32,height:i32,method:FreeRectChoiceHeuristic) -> Rect {
        let mut new_node = Rect::default();
        let mut score1 = 0;
        let mut score2 = 0;
        match method {
            FreeRectChoiceHeuristic::BestShortSideFit => {
                new_node = self.find_best_short_side_fit(width, height,&mut score1,&mut score2);
            },
            FreeRectChoiceHeuristic::BottomLeftRule => {
                new_node = self.find_bottom_left(width, height,&mut score1,&mut score2);
            },
            FreeRectChoiceHeuristic::ContactPointRule => {
                new_node = self.find_contact_point(width, height,&mut score1);
            },
            FreeRectChoiceHeuristic::BestAreaFit => {
                new_node = self.find_best_area_fit(width,height,&mut score1,&mut score2);
            },
            FreeRectChoiceHeuristic::BestLongSideFit => {
                new_node = self.find_best_long_side_fit(width, height,&mut score1,&mut score2);
            }
        }
        if new_node.height == 0 {
            return new_node;
        }
        let mut num_rect_to_process = self.free_rect.len();
        let mut i = 0;
        while i < num_rect_to_process {
            let free_rect:Rect = unsafe { self.free_rect.get_unchecked(i).clone() };
            if self.split_free_node(free_rect,&new_node) {
                self.free_rect.remove(i);
                num_rect_to_process -= 1;
            } else {
                i += 1;
            }
        }
        self.prune_free_list();
        self.used_rect.push(new_node.clone());
        new_node
    }

    fn find_best_short_side_fit(&mut self,width:i32,height:i32,best_short_side_fit:&mut i32,best_long_side_fit:&mut i32) -> Rect{
        let mut best_node = Rect::default();
        *best_short_side_fit = i32::max_value();
        for rect in self.free_rect.iter() {
            if rect.width >= width && rect.height >= height {
                let left_over_horiz = i32::abs(rect.width - width);
                let left_over_vert = i32::abs(rect.height - height);
                let short_side_fit = i32::min(left_over_horiz,left_over_vert);
                let long_side_fit = i32::max(left_over_horiz,left_over_vert);
                if short_side_fit < *best_short_side_fit || (short_side_fit == *best_short_side_fit && long_side_fit < *best_long_side_fit) {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = width;
                    best_node.height = height;
                    *best_short_side_fit = short_side_fit;
                    *best_long_side_fit = long_side_fit; 
                }
            } else if self.allow_rotations && rect.width >= height && rect.height >= width {
                let flip_left_over_horiz = i32::abs(rect.width - height);
                let flip_left_over_vert = i32::abs(rect.height - width);
                let flip_short_side_fit = i32::min(flip_left_over_horiz,flip_left_over_vert);
                let flip_long_side_fit = i32::max(flip_left_over_horiz,flip_left_over_vert);
                if flip_short_side_fit < *best_short_side_fit || (flip_short_side_fit == *best_short_side_fit && flip_long_side_fit < *best_long_side_fit) {
                    best_node.x = rect.x;
                    best_node.y = rect.y;
                    best_node.width = height;
                    best_node.height = width;
                    *best_short_side_fit = flip_short_side_fit;
                    *best_long_side_fit = flip_long_side_fit;
                }
            }
        }
        best_node
    }

    fn find_best_long_side_fit(&mut self,width:i32,height:i32,best_short_side_fit:&mut i32,best_long_side_fit:&mut i32) -> Rect {
        let mut best_node = Rect::default();
        *best_long_side_fit = i32::max_value();
        for free_rect in self.free_rect().iter() {
            if free_rect.width >= width && free_rect.height >= height {
                let left_over_horiz = i32::abs(free_rect.width - width);
                let left_over_vert = i32::abs(free_rect.height - height);
                let short_side_fit = i32::min(left_over_horiz,left_over_vert);
                let long_side_fit = i32::max(left_over_horiz,left_over_vert);
               if long_side_fit < *best_long_side_fit || (long_side_fit == *best_long_side_fit && short_side_fit < *best_short_side_fit) {
                   best_node.x = free_rect.x;
                   best_node.y = free_rect.y;
                   best_node.width = width;
                   best_node.height = height;
                   *best_short_side_fit = short_side_fit;
                   *best_long_side_fit = long_side_fit;
               }
            }
            if self.allow_rotations && free_rect.width >= height && free_rect.height >= width {
                let left_over_horiz = i32::abs(free_rect.width - height);
                let left_over_vert = i32::abs(free_rect.height - width);
                let short_side_fit = i32::min(left_over_horiz,left_over_vert);
                let long_side_fit = i32::max(left_over_horiz,left_over_vert);
                if long_side_fit < *best_long_side_fit || (long_side_fit == *best_long_side_fit && short_side_fit < *best_short_side_fit) {
                    best_node.x = free_rect.x;
                    best_node.y = free_rect.y;
                    best_node.width = height;
                    best_node.height = width;
                    *best_short_side_fit = short_side_fit;
                    *best_long_side_fit = long_side_fit;
                }
            }
        }
        best_node
    }

    fn find_bottom_left(&mut self,width:i32,height:i32,best_y:&mut i32,best_x:&mut i32) -> Rect {
        let mut best_node = Rect::default();
        *best_y = i32::max_value();
        for free_rect in self.free_rect.iter() {
            if free_rect.width >= width && free_rect.height >= height {
                let top_side_y = free_rect.y + height;
                if top_side_y < *best_y || (top_side_y == *best_y && free_rect.x < *best_x) {
                    best_node.x = free_rect.x;
                    best_node.y = free_rect.y;
                    best_node.width = width;
                    best_node.height = height;
                    *best_y = top_side_y;
                    *best_x = free_rect.x;
                }

                if self.allow_rotations && free_rect.width >= height && free_rect.height >= width {
                    if top_side_y < *best_y || (top_side_y == *best_y && free_rect.x < *best_x) {
                        best_node.x = free_rect.x;
                        best_node.y = free_rect.y;
                        best_node.width = height;
                        best_node.height = width;
                        *best_y = top_side_y;
                        *best_x = free_rect.x;
                    }
                }
            }
        }
        return best_node;
    }
    
    fn common_interval_length(i1start:i32,i1end:i32,i2start:i32,i2end:i32) -> i32 {
        if i1end < i2start || i2end < i1start {
            return 0;
        }
        i32::min(i1end,i2end) - i32::max(i1start,i2start)
    }

    fn contact_point_score_node(&self,x:i32,y:i32,width:i32,height:i32) -> i32 {
        let mut score = 0;
        if x == 0 || x + width == (self.width as i32) {
            score += height;
        }
        if y == 0 || y + height == self.height() as i32 {
            score += width;
        }
        for use_rect in self.used_rect().iter() {
            if use_rect.x == x + width  || use_rect.x + use_rect.width == x {
                score += Self::common_interval_length(use_rect.y,use_rect.y + use_rect.height,y,y+height);
            }
            if use_rect.y == y + height  || use_rect.y + use_rect.height == y {
                score += Self::common_interval_length(use_rect.x,use_rect.x + use_rect.width,x,x+width);
            }
        }
        score
    }

    fn find_contact_point(&mut self,width:i32,height:i32,best_contact_score:&mut i32) -> Rect {
        let mut best_node = Rect::default();
        *best_contact_score = -1;
        for free_rect in self.free_rect.iter() {
            if free_rect.width >= width && free_rect.height >= height {
                let score = self.contact_point_score_node(free_rect.x, free_rect.y,width,height);
                if score > *best_contact_score {
                    best_node.x = free_rect.x;
                    best_node.y = free_rect.y;
                    best_node.width = width;
                    best_node.height = height;
                    *best_contact_score = score;
                }
            }
            if self.allow_rotations && free_rect.width >= height && free_rect.height >= width {
                let score = self.contact_point_score_node(free_rect.x, free_rect.y,height,width);
                if score > *best_contact_score {
                    best_node.x = free_rect.x;
                    best_node.y = free_rect.y;
                    best_node.width = height;
                    best_node.height = width;
                    *best_contact_score = score;
                }
            }
        }
        best_node
    }

    fn find_best_area_fit(&mut self,width:i32,height:i32,best_area_fit:&mut i32,best_short_side_fit:&mut i32) -> Rect {
        let mut best_node = Rect::default();
        *best_area_fit = i32::max_value();
        for free_rect in self.free_rect().iter() {
            let area_fit = free_rect.width * free_rect.height - width * height;
            if free_rect.width >= width && free_rect.height >= height {
                let left_over_horiz = i32::abs(free_rect.width - width);
                let left_over_vert = i32::abs(free_rect.height - height);
                let short_side_fit = i32::min(left_over_horiz,left_over_vert);
                if area_fit < *best_area_fit || (area_fit == *best_area_fit && short_side_fit < *best_short_side_fit) {
                    best_node.x = free_rect.x;
                    best_node.y = free_rect.y;
                    best_node.width = width;
                    best_node.height = height;
                    *best_short_side_fit = short_side_fit;
                    *best_area_fit = area_fit;
                }
            }
            if self.allow_rotations && free_rect.height >= width && free_rect.width > height {
                let left_over_horiz = i32::abs(free_rect.width - height);
                let left_over_vert = i32::abs(free_rect.height - width);
                let short_side_fit = i32::min(left_over_horiz,left_over_vert);
                if area_fit < *best_area_fit || (area_fit == *best_area_fit && short_side_fit < *best_short_side_fit) {
                    best_node.x = free_rect.x;
                    best_node.y = free_rect.y;
                    best_node.width = height;
                    best_node.height = width;
                    *best_short_side_fit = short_side_fit;
                    *best_area_fit = area_fit;
                }
            }
        }
        best_node
    }

    fn split_free_node(&mut self,free_node:Rect,used_node:&Rect) -> bool {
        if used_node.x >= free_node.x + free_node.width  ||
           used_node.x + used_node.width <= free_node.x  || 
           used_node.y >= free_node.y + free_node.height || 
           used_node.y + used_node.height <= free_node.y {
            return false
        }
        if used_node.x < free_node.x + free_node.width && used_node.x + used_node.width > free_node.x {
            if used_node.y > free_node.y && used_node.y < free_node.y + free_node.height {
                let mut new_rect = free_node.clone();
                new_rect.height = used_node.y - new_rect.y;
                self.free_rect.push(new_rect);
            }

            if used_node.y + used_node.height < free_node.y + free_node.height {
                let mut new_node = free_node.clone();
                new_node.y = used_node.y + used_node.height;
                new_node.height = free_node.y + free_node.height - (used_node.y + used_node.height);
                self.free_rect.push(new_node);
            }
        }

        if used_node.y < free_node.y + free_node.height && used_node.y + used_node.height > free_node.y {
            if used_node.x > free_node.x && used_node.x < free_node.x + free_node.width {
                let mut new_node = free_node.clone();
                new_node.width = used_node.x - new_node.x;
                self.free_rect.push(new_node);
            }
            if used_node.x + used_node.width < free_node.x + free_node.width {
                let mut new_node = free_node.clone();
                new_node.x = used_node.x + used_node.width;
                new_node.width = free_node.x + free_node.width - (used_node.x + used_node.width);
                self.free_rect.push(new_node);
            }
        }
        true
    }

    fn prune_free_list(&mut self) {
        let mut i = 0;
        while i < self.free_rect.len() {
            let mut j = i + 1;
            while j < self.free_rect.len() {
                let ref_a = unsafe { self.free_rect.get_unchecked(i) };
                let ref_b = unsafe { self.free_rect.get_unchecked(j) };
                if Self::is_contained_in(ref_a,ref_b) {
                    self.free_rect.remove(i);
                    i-=1;
                    break;
                }
                if Self::is_contained_in(ref_b,ref_a) {
                    self.free_rect.remove(j);
                    j-=1;
                }
                j += 1;
            }
            i += 1;
        }
    }

    fn is_contained_in(a:&Rect,b:&Rect) -> bool {
        return a.x >= b.x && a.y >= b.y && a.x + a.width <= b.x + b.width && a.y + a.height <= b.y + b.height;
    }
}

