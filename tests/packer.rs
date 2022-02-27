
#[cfg(test)]
use core::default::Default;
use image_packer::*;
use proptest::prelude::*;
use proptest::array::uniform2;

proptest! {
    #[test]
    fn test_pack(spacing in 0usize..2, enable_rotate in any::<bool>(), ref sizes in proptest::collection::vec(uniform2(1usize..1024), 1..100)) {
        let texture_size = [1024, 1024];
        let packer = Packer {
            texture_size,
            spacing,
            enable_rotate,
        };
        let results = packer.pack(sizes).unwrap();

        // assert layouted image count
        let size_sum: usize = results.iter().map(|a|a.len()).sum();
        prop_assert_eq!(sizes.len(), size_sum, "{:?} {:?}", sizes, results);

        // assert all indices exist
        let mut indices = results.iter().map(|a|a.iter().map(|b|b.index).collect::<Vec<usize>>()).flatten().collect::<Vec<usize>>();
        indices.sort();
        for (index, actual) in indices.iter().enumerate() {
            prop_assert_eq!(index, *actual);
        }

        // assert all images are layouted inside of texture
        let texture = Rect { position: [0, 0], size: texture_size };
        for layout in results.iter().map(|a|a.iter()).flatten().collect::<Vec<&Layout>>() {
            let size = if layout.rotated
                    { let s = sizes[layout.index]; [s[1], s[0]] } else
                    { sizes[layout.index] };
            let rect = Rect { position: layout.position, size };
            prop_assert!(texture.include(&rect));
        }

        // assert all images do not have intersection
        for layouts in results.iter() {
            for layout1 in layouts.iter() {
                for layout2 in layouts.iter() {
                    if layout1.index < layout2.index {
                        let size1 = if layout1.rotated
                                { let s = sizes[layout1.index]; [s[1], s[0]] } else
                                { sizes[layout1.index] };
                        let rect1 = Rect { position: layout1.position, size: [size1[0] + spacing, size1[1] + spacing] };
                        let size2 = if layout2.rotated
                                { let s = sizes[layout2.index]; [s[1], s[0]] } else
                                { sizes[layout2.index] };
                        let rect2 = Rect { position: layout2.position, size: [size2[0] + spacing, size2[1] + spacing] };
                        prop_assert!(!rect1.has_intersection(&rect2), "{:?} {:?} {:?}", rect1, rect2, results);
                    }
                }
            }
        }
    }
}
