use std::collections::btree_map::{BTreeMap};
use std::ops::Bound::{Included, Unbounded};

pub const MAX_TEXTURE_SIZE: usize = 4096;

#[derive(Debug)]
pub struct Packer {
    pub texture_size: [usize; 2],
    pub spacing: usize,
    pub enable_rotate: bool,
}

#[derive(Debug)]
struct Packed {
    pub layouts: Vec<Layout>,
    pub spaces: Spaces,
}

#[derive(Debug)]
pub struct Layout {
    pub index: usize,
    pub position: [usize; 2],
    pub rotated: bool,
}

#[derive(Debug)]
pub struct Rect {
    pub size: [usize; 2],
    pub position: [usize; 2],
}

#[derive(Debug)]
pub(crate) struct Spaces {
    spaces: BTreeMap<usize, BTreeMap<usize, Vec<Rect>>>
}

impl Rect {
    pub fn has_intersection(&self, other: &Rect) -> bool {
        let [w, h] = other.size;
        let [x, y] = other.position;
        let [rw, rh] = self.size;
        let [rx, ry] = self.position;
        let (cx, cy) = (x * 2 + w, y * 2 + h);
        let (rcx, rcy) = (rx * 2 + rw, ry * 2 + rh);
        let (dx, dy) = ((if cx >= rcx {cx - rcx} else {rcx - cx}), if cy >= rcy {cy - rcy} else {rcy - cy});
        return dx < w + rw && dy < h + rh;
    }

    pub fn include(&self, other: &Rect) -> bool {
        let [w, h] = other.size;
        let [x, y] = other.position;
        let [rx, ry] = self.position;
        let [rw, rh] = self.size;
        return rx <= x && x + w <= rx + rw && ry <= y && y + h <= ry + rh;
    }

    pub fn divide(&self, other: &Rect) -> Vec<Rect> {
        let mut rects: Vec<Rect> = Vec::with_capacity(2);
        let [w, h] = other.size;
        let [x, y] = other.position;
        let [rx, ry] = self.position;
        let [rw, rh] = self.size;

        // left
        if rx < x && x < rx + rw {
            let size = [x - rx, rh];
            let position = [rx, ry];
            rects.push(Rect{ size, position });
        }

        // right
        if rx < x + w && x + w < rx + rw {
            let size = [rx + rw - (x + w), rh];
            let position = [x + w, ry];
            rects.push(Rect{ size, position });
        }

        // top
        if ry < y && y < ry + rh {
            let size = [rw, y - ry];
            let position = [rx, ry];
            rects.push(Rect{ size, position });
        }

        // bottom
        if ry < y + h && y + h < ry + rh {
            let size = [rw, ry + rh - (y + h)];
            let position = [rx, y + h];
            rects.push(Rect{ size, position });
        }

        if rects.is_empty() && !self.has_intersection(other) {
            rects.push(Rect {size: self.size, position: self.position});
        }

        return rects;
    }
}

impl Spaces {
    pub fn new(size: [usize; 2]) -> Spaces {
        let area = size[0] * size[1];
        let rect = Rect {
            size,
            position: [0, 0],
        };
        return Spaces { spaces: BTreeMap::from([(area, BTreeMap::from([(size[0], Vec::from([rect]))]))]) };
    }

    pub fn find_space(&self, size: [usize; 2]) -> Option<Rect> {
        let area = size[0] * size[1];
        for (space_area, spaces_equal_area) in self.spaces.range((Included(area), Unbounded)) {
            if let Some((_, found_spaces)) = spaces_equal_area
                    .range((Included(size[0]), Unbounded))
                    .find(|(space_width, spaces_equal_width)| !spaces_equal_width.is_empty() && (**space_width >= size[0]) && (*space_area >= (size[1] * (**space_width)))) {
                return Some(Rect{ size:found_spaces[0].size, position: found_spaces[0].position});
            }
        }
        return None;
    }

    pub fn exclude(&mut self, other: &Rect) {
        let mut divided_spaces: Vec<Rect> = Vec::new();
        for (_, spaces_equal_area) in self.spaces.iter_mut() {
            for (_, spaces_equal_width) in spaces_equal_area.into_iter() {
                let mut remove_indices: Vec<usize> = Vec::new();
                for (i, space) in spaces_equal_width.into_iter().enumerate() {
                    if space.has_intersection(other) {
                        remove_indices.push(i);
                        divided_spaces.append(&mut space.divide(&Rect{size: other.size, position: other.position}));
                    }
                }

                // remove reversely not to change other indices
                for i in remove_indices.iter().rev() {
                    spaces_equal_width.remove(*i);
                }
            }
        }

        // remove empty
        for (_, spaces_equal_area) in self.spaces.iter_mut() {
            spaces_equal_area.retain(|_, a|!a.is_empty());
        }
        self.spaces.retain(|_, a|!a.is_empty());

        // sort new divided spaces by area
        divided_spaces.sort_by(|a, b|(b.size[0] * b.size[1]).cmp(&(a.size[0] * a.size[1])));

        // add new spaces
        for space in divided_spaces {
            self.add(space);
        }
    }

    pub fn add(&mut self, new_space: Rect) {
        let area = new_space.size[0] * new_space.size[1];
        let width = new_space.size[0];
        for (_, spaces_equal_area) in self.spaces.range((Included(area), Unbounded)) {
            for (_, spaces_equal_width) in spaces_equal_area.range((Included(width), Unbounded)) {
                for a in spaces_equal_width {
                    if a.include(&new_space) {
                        // other space cover new one
                        return;
                    }
                }
            }
        }

        if let Some(spaces_equal_area) = self.spaces.get_mut(&area) {
            if let Some(spaces_equal_width) = spaces_equal_area.get_mut(&width) {
                spaces_equal_width.push(new_space);
            } else {
                spaces_equal_area.insert(width, vec![new_space]);
            }
        } else {
            self.spaces.insert(area, BTreeMap::from([
                (width, vec![new_space])
            ]));
        }
    }
}

impl Packed {
    pub fn new(texture_size: [usize; 2]) -> Packed {
        return Packed { layouts: Vec::new(), spaces: Spaces::new(texture_size) };
    }
}

impl Packer {
    pub fn pack(
        &self,
        image_sizes: &Vec<[usize; 2]>
    ) -> Result<Vec<Vec<Layout>>, String> {
        let mut results = Vec::new();
        let mut current = Packed::new(self.texture_size);

        if self.texture_size[0] == 0 || self.texture_size[1] == 0 || self.texture_size[0] > MAX_TEXTURE_SIZE || self.texture_size[1] > MAX_TEXTURE_SIZE {
            return Err(format!("bad texture size. {:?}", self));
        }

        if self.spacing >= self.texture_size[0] || self.spacing >= self.texture_size[1] {
            return Err(format!("spacing too large. {:?}", self));
        }

        for (index, size) in image_sizes.iter().enumerate() {
            if size[0] > self.texture_size[0] || size[1] > self.texture_size[1] {
                return Err(format!("pack failed. image size larger than texture size. ({}, {}) > ({}, {})", size[0], size[1], self.texture_size[0], self.texture_size[1]));
            }
            if !self.try_pack_one(&mut current, (index, size)) {
                let mut next = Packed::new(self.texture_size);
                std::mem::swap(&mut next, &mut current);
                results.push(next);
                self.try_pack_one(&mut current, (index, size));
            }
        }
        if !current.layouts.is_empty() {
            results.push(current);
        }

        return Ok(results.into_iter().map(|a|a.layouts).collect());
    }

    fn try_pack_one(
        &self,
        packed: &mut Packed,
        (index, size): (usize, &[usize; 2]),
    ) -> bool {
        let size_with_spacing = [std::cmp::min(size[0] + self.spacing, self.texture_size[0]), std::cmp::min(size[1] + self.spacing, self.texture_size[1])];
        if let Some(space) = packed.spaces.find_space(size_with_spacing) {
            let layout = Layout{ index, position: space.position, rotated: false };
            packed.layouts.push(layout);
            packed.spaces.exclude(&Rect{ position: space.position, size: size_with_spacing });
            return true;
        }
        if self.enable_rotate && size[1] <= self.texture_size[0] && size[0] <= self.texture_size[1] {
            let rotated_size = [std::cmp::min(size[1] + self.spacing, self.texture_size[0]), std::cmp::min(size[0] + self.spacing, self.texture_size[1])];
            if let Some(space) = packed.spaces.find_space(rotated_size) {
                let layout = Layout{ index, position: space.position, rotated: true };
                packed.layouts.push(layout);
                packed.spaces.exclude(&Rect{ position: space.position, size: rotated_size });
                return true;
            }
        }
        return false;
    }
}
