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
    pub regions: Regions,
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
pub(crate) struct Regions {
    regions: BTreeMap<usize, BTreeMap<usize, Vec<Rect>>>
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

impl Regions {
    pub fn new(size: [usize; 2]) -> Regions {
        let area = size[0] * size[1];
        let rect = Rect {
            size,
            position: [0, 0],
        };
        return Regions { regions: BTreeMap::from([(area, BTreeMap::from([(size[0], Vec::from([rect]))]))]) };
    }

    pub fn find_space(&self, size: [usize; 2]) -> Option<Rect> {
        let area = size[0] * size[1];
        for (region_area, regions_equal_area) in self.regions.range((Included(area), Unbounded)) {
            if let Some((_, found_regions)) = regions_equal_area
                    .range((Included(size[0]), Unbounded))
                    .find(|(region_width, regions_equal_width)| !regions_equal_width.is_empty() && (**region_width >= size[0]) && (*region_area >= (size[1] * (**region_width)))) {
                return Some(Rect{ size:found_regions[0].size, position: found_regions[0].position});
            }
        }
        return None;
    }

    pub fn exclude(&mut self, other: &Rect) {
        let mut divided_regions: Vec<Rect> = Vec::new();
        for (_, regions_equal_area) in self.regions.iter_mut() {
            for (_, regions_equal_width) in regions_equal_area.into_iter() {
                let mut remove_indices: Vec<usize> = Vec::new();
                for (i, region) in regions_equal_width.into_iter().enumerate() {
                    if region.has_intersection(other) {
                        remove_indices.push(i);
                        divided_regions.append(&mut region.divide(&Rect{size: other.size, position: other.position}));
                    }
                }

                // remove reversely not to change other indices
                for i in remove_indices.iter().rev() {
                    regions_equal_width.remove(*i);
                }
            }
        }

        // remove empty
        for (_, regions_equal_area) in self.regions.iter_mut() {
            regions_equal_area.retain(|_, a|!a.is_empty());
        }
        self.regions.retain(|_, a|!a.is_empty());

        // sort new divided regions by area
        divided_regions.sort_by(|a, b|(b.size[0] * b.size[1]).cmp(&(a.size[0] * a.size[1])));

        // add new regions
        for region in divided_regions {
            self.add(region);
        }
    }

    pub fn add(&mut self, new_region: Rect) {
        let area = new_region.size[0] * new_region.size[1];
        let width = new_region.size[0];
        for (_, regions_equal_area) in self.regions.range((Included(area), Unbounded)) {
            for (_, regions_equal_width) in regions_equal_area.range((Included(width), Unbounded)) {
                for a in regions_equal_width {
                    if a.include(&new_region) {
                        // other region cover new one
                        return;
                    }
                }
            }
        }

        if let Some(regions_equal_area) = self.regions.get_mut(&area) {
            if let Some(regions_equal_width) = regions_equal_area.get_mut(&width) {
                regions_equal_width.push(new_region);
            } else {
                regions_equal_area.insert(width, vec![new_region]);
            }
        } else {
            self.regions.insert(area, BTreeMap::from([
                (width, vec![new_region])
            ]));
        }
    }
}

impl Packed {
    pub fn new(texture_size: [usize; 2]) -> Packed {
        return Packed { layouts: Vec::new(), regions: Regions::new(texture_size) };
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
        if let Some(space) = packed.regions.find_space(size_with_spacing) {
            let layout = Layout{ index, position: space.position, rotated: false };
            packed.layouts.push(layout);
            packed.regions.exclude(&Rect{ position: space.position, size: size_with_spacing });
            return true;
        }
        if self.enable_rotate && size[1] <= self.texture_size[0] && size[0] <= self.texture_size[1] {
            let rotated_size = [std::cmp::min(size[1] + self.spacing, self.texture_size[0]), std::cmp::min(size[0] + self.spacing, self.texture_size[1])];
            if let Some(space) = packed.regions.find_space(rotated_size) {
                let layout = Layout{ index, position: space.position, rotated: true };
                packed.layouts.push(layout);
                packed.regions.exclude(&Rect{ position: space.position, size: rotated_size });
                return true;
            }
        }
        return false;
    }
}