//! Image processing.
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use image::{
    imageops::overlay, io::Reader, DynamicImage, GenericImageView, ImageResult, Rgba, RgbaImage,
};
use num::rational::Ratio;
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// Common aspect ratios for images.
pub static COMMON_RATIOS: Lazy<Vec<Ratio<u32>>> = Lazy::new(|| {
    vec![
        Ratio::new(1, 1),
        Ratio::new(3, 2),
        Ratio::new(2, 3),
        Ratio::new(4, 3),
        Ratio::new(3, 4),
        Ratio::new(4, 5),
        Ratio::new(5, 4),
        Ratio::new(16, 9),
        Ratio::new(9, 16),
    ]
});

/// Process an image, adding a white border to it, and save it locally.
///
/// The final image will be adjusted to match the closest common image ratio,
/// so the border size may not be respected along the smaller dimension.
/// The larger dimension, however, will always have the border size respected.
///
/// The image is saved to a new file with "_bordered" appended to the file stem.
/// This does not prevent overwrites to any existing files that would have the same file name.
///
/// # Args
/// * `file` - The image's file path.
/// * `border` - The border size in pixels.
/// * `force_ratio` - The ratio to force, or none if the ratio should be guessed.
/// * `force_orientation` - Whether to force the orientation when the ratio is forced.
pub fn process_and_save_local(
    file: &Path,
    border: u32,
    force_ratio: Option<Ratio<u32>>,
    force_orientation: bool,
) -> ImageResult<()> {
    let image = Reader::open(file)?.decode()?;
    let final_ratio = match force_ratio {
        Some(ratio) => {
            if force_orientation {
                ratio
            } else {
                approximation(image.dimensions(), &[ratio, ratio.recip()])
            }
        }
        None => approximation(image.dimensions(), &COMMON_RATIOS),
    };
    add_border(&image, adjust(image.dimensions(), border, final_ratio)).save(file_name(file))
}

/// Generate a new file name for an image that is to be bordered.
///
/// Currently just take the file stem and append "_bordered" to the end.
///
/// # Args
/// * `file` - The image's file path.
///
/// # Returns
/// A new file path for the bordered image.
fn file_name(file: &Path) -> PathBuf {
    let mut new_path = PathBuf::new();
    new_path.push(file);
    if let Some(stem) = file.file_stem() {
        let mut new_stem = OsString::new();
        new_stem.push(stem);
        new_stem.push("_bordered");
        new_path.set_file_name(new_stem);
    }
    if let Some(extension) = file.extension() {
        new_path.set_extension(extension);
    }
    new_path
}

/// Add a white border to an image, matching the final dimensions given.
///
/// # Args
/// * `image` - The original image.
/// * `final_dims` - The final dimensions (width and height) of the bordered image.
fn add_border(image: &DynamicImage, final_dims: (u32, u32)) -> DynamicImage {
    let (width, height) = final_dims;
    let mut background = RgbaImage::from_pixel(width, height, Rgba([255, 255, 255, 255]));
    let x_offset = (width - image.width()) / 2;
    let y_offset = (height - image.height()) / 2;
    overlay(&mut background, image, x_offset as i64, y_offset as i64);
    DynamicImage::ImageRgba8(background)
}

/// Adjust a pair of dimensions to account for a border while satisfying a given ratio.
///
/// The adjustment works by adding the border size to the larger dimension
/// and then filling in the smaller dimension to match the ratio.
///
/// # Args
/// * `dims` - The pair of dimensions; typically represents width and height of a rectangle.
/// * `border` - Amount of border to add to the dimensions; always respected on the larger dimension.
/// * `ratio` - The expected ratio of the final dimensions; the smaller dimension will get extra border to satisfy this.
///
/// # Returns
/// The adjusted dimensions.
fn adjust(dims: (u32, u32), border: u32, ratio: Ratio<u32>) -> (u32, u32) {
    let (width, height) = dims;
    match width > height {
        true => {
            let new_w = width + border;
            let new_h = new_w * ratio.denom() / ratio.numer();
            (new_w, new_h)
        }
        false => {
            let new_h = height + border;
            let new_w = new_h * ratio.numer() / ratio.denom();
            (new_w, new_h)
        }
    }
}

/// Return the best ratio approximation of a pair of given dimensions.
///
/// # Args
/// * `dims` - The pair of dimensions; typically represents width and height of a rectangle.
/// * `options` - Ratio options for approximating.
///
/// # Returns
/// The best approximation from the given options.
/// If there are multiple best options then the first one in the list is used.
/// In the rare case that no given options work at all, the raw ratio of the dimensions is returned.
fn approximation(dims: (u32, u32), options: &[Ratio<u32>]) -> Ratio<u32> {
    let ratio = Ratio::new(dims.0, dims.1);
    *options
        .par_iter()
        .map(|option| (option, proximity(option, &ratio)))
        .reduce(
            || (&ratio, Ratio::from_integer(u32::MAX)),
            |a, b| {
                if b.1 < a.1 {
                    b
                } else {
                    a
                }
            },
        )
        .0
}

/// Calculate the proximity of two ratios.
///
/// # Args
/// * `a` - The first ratio.
/// * `b` - The second ratio.
///
/// # Returns
/// The proximity ratio; the closer to 0 the better.
fn proximity(a: &Ratio<u32>, b: &Ratio<u32>) -> Ratio<u32> {
    let check = a * b.recip();
    let one = Ratio::from_integer(1);
    if check > one {
        check - one
    } else {
        one - check
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::*;

    #[fixture]
    fn base_pixel() -> Rgba<u8> {
        Rgba([0, 0, 0, 255])
    }

    #[fixture]
    fn base_image(base_pixel: Rgba<u8>) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 2, base_pixel))
    }

    #[rstest]
    #[case(Path::new("test.jpg"), Path::new("test_bordered.jpg"))]
    #[case(Path::new(".png"), Path::new(".png_bordered"))]
    #[case(Path::new("test"), Path::new("test_bordered"))]
    fn test_file_name(#[case] input: &Path, #[case] expected: &Path) {
        assert_eq!(file_name(input), expected);
    }

    #[rstest]
    #[case(
        (2, 2),
        DynamicImage::ImageRgba8(RgbaImage::from_vec(2, 2, vec![0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]).unwrap()),
    )]
    #[case(
        (2, 3),
        DynamicImage::ImageRgba8(RgbaImage::from_vec(2, 3, vec![0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255]).unwrap()),
    )]
    #[case(
        (2, 4),
        DynamicImage::ImageRgba8(RgbaImage::from_vec(2, 4, vec![255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255]).unwrap()),
    )]
    #[case(
        (3, 2),
        DynamicImage::ImageRgba8(RgbaImage::from_vec(3, 2, vec![0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255]).unwrap()),
    )]
    #[case(
        (4, 2),
        DynamicImage::ImageRgba8(RgbaImage::from_vec(4, 2, vec![255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255]).unwrap()),
    )]
    fn test_add_border(
        base_image: DynamicImage,
        #[case] final_dims: (u32, u32),
        #[case] expected: DynamicImage,
    ) {
        assert_eq!(add_border(&base_image, final_dims), expected);
    }

    #[rstest]
    fn test_add_border_transparent_base(#[with(Rgba([0, 0, 0, 0]))] base_image: DynamicImage) {
        assert_eq!(
            add_border(&base_image, (12, 24)),
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(12, 24, Rgba([255, 255, 255, 255])))
        );
    }

    #[rstest]
    #[case((0, 0), 10, Ratio::new(1, 10), (1, 10))]
    #[case((0, 0), 10, Ratio::new(10, 1), (100, 10))]
    #[case((20, 30), 10, Ratio::new(3, 4), (30, 40))]
    #[case((30, 20), 10, Ratio::new(1, 1), (40, 40))]
    fn test_adjust(
        #[case] dims: (u32, u32),
        #[case] border: u32,
        #[case] ratio: Ratio<u32>,
        #[case] expected: (u32, u32),
    ) {
        assert_eq!(adjust(dims, border, ratio), expected);
    }

    #[rstest]
    #[case((14, 18), &[], Ratio::new(7, 9))]
    #[case((14, 18), &[Ratio::new(5, 9)], Ratio::new(5, 9))]
    #[case((14, 18), &[Ratio::new(12, 18), Ratio::new(13, 18)], Ratio::new(13, 18))]
    #[case((14, 18), &[Ratio::new(13, 18), Ratio::new(15, 18)], Ratio::new(13, 18))]
    #[case((14, 18), &[Ratio::new(15, 18), Ratio::new(13, 18)], Ratio::new(5, 6))]
    fn test_approximation(
        #[case] dims: (u32, u32),
        #[case] options: &[Ratio<u32>],
        #[case] expected: Ratio<u32>,
    ) {
        assert_eq!(approximation(dims, options), expected);
    }

    #[rstest]
    #[case(Ratio::new(2, 3), Ratio::new(4, 6), Ratio::from_integer(0))]
    #[case(Ratio::new(4, 6), Ratio::new(2, 3), Ratio::from_integer(0))]
    #[case(Ratio::new(2, 1), Ratio::new(4, 1), Ratio::new(1, 2))]
    #[case(Ratio::new(4, 1), Ratio::new(2, 1), Ratio::from_integer(1))]
    #[case(Ratio::new(3, 4), Ratio::new(3, 5), Ratio::new(1, 4))]
    #[case(Ratio::new(3, 5), Ratio::new(3, 4), Ratio::new(1, 5))]
    #[case(Ratio::new(4, 5), Ratio::new(5, 4), Ratio::new(9, 25))]
    #[case(Ratio::new(99, 100), Ratio::new(99, 101), Ratio::new(1, 100))]
    #[case(Ratio::new(99, 101), Ratio::new(99, 100), Ratio::new(1, 101))]
    fn test_proximity(#[case] a: Ratio<u32>, #[case] b: Ratio<u32>, #[case] expected: Ratio<u32>) {
        assert_eq!(proximity(&a, &b), expected);
    }
}
