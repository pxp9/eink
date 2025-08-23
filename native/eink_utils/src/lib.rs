use std::{path::Path, sync::Mutex};

use dither_lib::{
    clamp_f64_to_u8,
    color::RGB,
    create_quantize_n_bits_func,
    ditherer::{
        atkinson_ditherer, burkes_ditherer, floyd_steinberg_ditherer, jarvis_ditherer,
        sierra3_ditherer, stucki_ditherer, Dither,
    },
    img::Img,
};
use image::{imageops::FilterType, DynamicImage, GrayImage, ImageError, RgbImage};
use rustler::{Atom, Env, NewBinary, Resource, ResourceArc, Term};

mod atoms {
    rustler::atoms! {
        file_not_found,
        io_error,
        image_decode_failed,
        unknown,
        success,

        // ditherer names
        floyd_steinberg,
        atkinson,
        stucki,
        burkes,
        jarvis,
        sierra,
    }
}

pub struct ImageResource {
    pub inner: Mutex<DynamicImage>,
}

#[rustler::resource_impl]
impl Resource for ImageResource {}

type ImageArc = ResourceArc<ImageResource>;

#[derive(Debug)]
enum DitherAlgorithm {
    FloydSteinberg,
    Atkinson,
    Stucki,
    Burkes,
    Jarvis,
    Sierra,
}

impl<'a> rustler::Decoder<'a> for DitherAlgorithm {
    fn decode(term: rustler::Term<'a>) -> rustler::NifResult<Self> {
        let atom: rustler::Atom = term.decode()?;

        if atom == atoms::floyd_steinberg() {
            Ok(DitherAlgorithm::FloydSteinberg)
        } else if atom == atoms::atkinson() {
            Ok(DitherAlgorithm::Atkinson)
        } else if atom == atoms::stucki() {
            Ok(DitherAlgorithm::Stucki)
        } else if atom == atoms::burkes() {
            Ok(DitherAlgorithm::Burkes)
        } else if atom == atoms::jarvis() {
            Ok(DitherAlgorithm::Jarvis)
        } else if atom == atoms::sierra() {
            Ok(DitherAlgorithm::Sierra)
        } else {
            Err(rustler::Error::BadArg)
        }
    }
}

#[rustler::nif]
fn nif_load(path: String) -> Result<ImageArc, Atom> {
    if Path::new(&path).exists() {
        // Load and decode the image
        let img = image::open(&path)
            .map_err(|err| match err {
                ImageError::Decoding(_) => atoms::image_decode_failed(),
                ImageError::IoError(_) => atoms::io_error(),
                _ => atoms::unknown(),
            })?
            .to_rgb8();

        let img = DynamicImage::ImageRgb8(img);

        let img_resource = ImageResource {
            inner: Mutex::new(img),
        };

        Ok(ImageArc::new(img_resource))
    } else {
        Err(atoms::file_not_found())
    }
}

#[rustler::nif]
fn nif_resize(img: ImageArc, width: u32, height: u32) -> Result<ImageArc, Atom> {
    let mut locked = img.inner.lock().map_err(|_| atoms::unknown())?;
    *locked = locked.resize_to_fill(width, height, FilterType::Triangle);
    Ok(img.clone())
}

#[rustler::nif]
fn nif_dither_rgb(
    img: ImageArc,
    dither_algorithm: DitherAlgorithm,
    palette: Term,
) -> Result<ImageArc, Atom> {
    let ditherer = match dither_algorithm {
        DitherAlgorithm::FloydSteinberg => floyd_steinberg_ditherer(),
        DitherAlgorithm::Atkinson => atkinson_ditherer(),
        DitherAlgorithm::Stucki => stucki_ditherer(),
        DitherAlgorithm::Burkes => burkes_ditherer(),
        DitherAlgorithm::Jarvis => jarvis_ditherer(),
        DitherAlgorithm::Sierra => sierra3_ditherer(),
    };

    let quantize = create_quantize_n_bits_func(1).map_err(|_| atoms::unknown())?;

    let mut locked = img.inner.lock().map_err(|_| atoms::unknown())?;

    let rgb_pixels: Vec<RGB<u8>> = locked.to_rgb8().pixels().map(|p| RGB::from(p.0)).collect();

    let dither_img = Img::<RGB<u8>>::new(rgb_pixels, locked.width())
        .ok_or(atoms::unknown())?
        .convert_with(|rgb| rgb.convert_with(f64::from));

    let dither_img = ditherer.dither(dither_img, RGB::map_across(quantize));
    let img_w = dither_img.width();
    let img_h = dither_img.height();
    let dithered_u8 = dither_img
        .convert_with(|rgb| rgb.convert_with(clamp_f64_to_u8))
        .raw_buf();

    let result = RgbImage::from_raw(img_w, img_h, dithered_u8).ok_or(atoms::unknown())?;

    *locked = DynamicImage::ImageRgb8(result);

    Ok(img.clone())
}

#[rustler::nif]
fn nif_dither_grayscale(
    img: ImageArc,
    dither_algorithm: DitherAlgorithm,
    depth: u8,
) -> Result<ImageArc, Atom> {
    let ditherer = match dither_algorithm {
        DitherAlgorithm::FloydSteinberg => floyd_steinberg_ditherer(),
        DitherAlgorithm::Atkinson => atkinson_ditherer(),
        DitherAlgorithm::Stucki => stucki_ditherer(),
        DitherAlgorithm::Burkes => burkes_ditherer(),
        DitherAlgorithm::Jarvis => jarvis_ditherer(),
        DitherAlgorithm::Sierra => sierra3_ditherer(),
    };

    let quantize = create_quantize_n_bits_func(depth).map_err(|_| atoms::unknown())?;

    let mut locked = img.inner.lock().map_err(|_| atoms::unknown())?;

    let dither_img = Img::<u8>::new(locked.to_luma8().into_raw(), locked.width())
        .ok_or(atoms::unknown())?
        .convert_with(f64::from);

    let dither_img = ditherer.dither(dither_img, quantize);
    let img_w = dither_img.width();
    let img_h = dither_img.height();
    let dithered_u8 = dither_img.convert_with(clamp_f64_to_u8).into_vec();

    let result = GrayImage::from_raw(img_w, img_h, dithered_u8).ok_or(atoms::unknown())?;

    *locked = DynamicImage::ImageLuma8(result);

    Ok(img.clone())
}

#[rustler::nif]
fn nif_save(img: ImageArc, path: String) -> Result<Atom, Atom> {
    let locked = img.inner.lock().map_err(|_| atoms::unknown())?;
    locked.save(path).map_err(|_| atoms::unknown())?;
    Ok(atoms::success())
}

#[rustler::nif]
fn nif_to_binary<'a>(env: Env<'a>, img: ImageArc) -> Result<Term<'a>, Atom> {
    let locked = img.inner.lock().map_err(|_| atoms::unknown())?;
    let bytes = locked.clone().into_bytes();
    let mut binary = NewBinary::new(env, bytes.len());
    binary.as_mut_slice().copy_from_slice(bytes.as_slice());

    Ok(binary.into())
}

rustler::init!("Elixir.EInk.Utils");
