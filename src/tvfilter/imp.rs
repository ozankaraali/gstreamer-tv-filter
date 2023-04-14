// Copyright (C) 2017,2018 Sebastian Dröge <sebastian@centricular.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use gst::glib;
use gst::prelude::*;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::*;
use gst_video::subclass::prelude::*;

use rand::Rng;
use rand::distributions::{Distribution};
use image::{GenericImageView, ImageBuffer, Rgba};
use rand::rngs::ThreadRng;

use std::sync::Mutex;

use once_cell::sync::Lazy;

// This module contains the private implementation details of our element
//
static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "rstvfilter",
        gst::DebugColorFlags::empty(),
        Some("Rust TV Filter"),
    )
});

// Default values of properties
const DEFAULT_INVERT: bool = false;
const DEFAULT_SHIFT: u32 = 0;

// Property value storage
#[derive(Debug, Clone, Copy)]
struct Settings {
    invert: bool,
    shift: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            invert: DEFAULT_INVERT,
            shift: DEFAULT_SHIFT,
        }
    }
}

// Struct containing all the element data
#[derive(Default)]
pub struct TvFilter {
    settings: Mutex<Settings>,
}

impl TvFilter {
    // Converts one pixel of BGRx to a grayscale value, shifting and/or
    // inverting it as configured
    #[inline]
    fn bgrx_to_gray(in_p: &[u8], shift: u8, invert: bool) -> &[u8] {
        // See https://en.wikipedia.org/wiki/YUV#SDTV_with_BT.601
        // const R_Y: u32 = 19595; // 0.299 * 65536
        // const G_Y: u32 = 38470; // 0.587 * 65536
        // const B_Y: u32 = 7471; // 0.114 * 65536

        // assert_eq!(in_p.len(), 4);

        // let b = u32::from(in_p[0]);
        // let g = u32::from(in_p[1]);
        // let r = u32::from(in_p[2]);

        // let gray = ((r * R_Y) + (g * G_Y) + (b * B_Y)) / 65536;
        // let gray = (gray as u8).wrapping_add(shift);

        // if invert {
        //     255 - gray
        // } else {
        //     gray
        // }
        in_p
    }

    #[inline]
    fn video_tv_filter(rand: &[u8], in_p: &[u8]) -> Vec<u8> {
        // It should append the random noise to the input image
        let mut out_p = in_p.to_vec();
        // println!("rand: {:?}", rand); // r,g,b values
        
        
        out_p
    }
    
    // fn add_noise_to_image(image: &dyn GenericImageView, mean: f32, stddev: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    //     let (width, height) = image.dimensions();
    //     let mut noisy_image = ImageBuffer::new(width, height);

    //     // Create a normal distribution with the specified mean and standard deviation
    //     let distribution = Normal::new(mean, stddev);

    //     // Process the image in batches of 100 pixels at a time
    //     let batch_size = 100;
    //     for y in (0..height).step_by(batch_size as usize) {
    //         for x in (0..width).step_by(batch_size as usize) {
    //             // Process each pixel in the current batch
    //             for j in y..y+batch_size {
    //                 for i in x..x+batch_size {
    //                     if i < width && j < height {
    //                         let pixel = image.get_pixel(i, j);
    //                         let mut noisy_pixel = pixel.to_rgba();

    //                         // Add noise to each channel of the pixel using the distribution
    //                         for channel in &mut noisy_pixel.channels_mut()[0..3] {
    //                             let noise = distribution.sample(&mut rand::thread_rng()) as i16;
    //                             *channel = (*channel as i16 + noise).max(0).min(255) as u8;
    //                         }

    //                         noisy_image.put_pixel(i, j, noisy_pixel);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     noisy_image
    // }
}


// This trait registers our type with the GObject object system and
// provides the entry points for creating a new instance and setting
// up the class data
#[glib::object_subclass]
impl ObjectSubclass for TvFilter {
    const NAME: &'static str = "GstRsTvFilter";
    type Type = super::TvFilter;
    type ParentType = gst_video::VideoFilter;
}

// Implementation of glib::Object virtual methods
impl ObjectImpl for TvFilter {
    fn properties() -> &'static [glib::ParamSpec] {
        // Metadata for the properties
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecBoolean::builder("invert")
                    .nick("Invert")
                    .blurb("Invert grayscale output")
                    .default_value(DEFAULT_INVERT)
                    .mutable_playing()
                    .build(),
                glib::ParamSpecUInt::builder("shift")
                    .nick("Shift")
                    .blurb("Shift grayscale output (wrapping around)")
                    .maximum(255)
                    .default_value(DEFAULT_SHIFT)
                    .mutable_playing()
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    // Called whenever a value of a property is changed. It can be called
    // at any time from any thread.
    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "invert" => {
                let mut settings = self.settings.lock().unwrap();
                let invert = value.get().expect("type checked upstream");
                gst::info!(
                    CAT,
                    imp: self,
                    "Changing invert from {} to {}",
                    settings.invert,
                    invert
                );
                settings.invert = invert;
            }
            "shift" => {
                let mut settings = self.settings.lock().unwrap();
                let shift = value.get().expect("type checked upstream");
                gst::info!(
                    CAT,
                    imp: self,
                    "Changing shift from {} to {}",
                    settings.shift,
                    shift
                );
                settings.shift = shift;
            }
            _ => unimplemented!(),
        }
    }

    // Called whenever a value of a property is read. It can be called
    // at any time from any thread.
    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "invert" => {
                let settings = self.settings.lock().unwrap();
                settings.invert.to_value()
            }
            "shift" => {
                let settings = self.settings.lock().unwrap();
                settings.shift.to_value()
            }
            _ => unimplemented!(),
        }
    }
}

impl GstObjectImpl for TvFilter {}

// Implementation of gst::Element virtual methods
impl ElementImpl for TvFilter {
    // Set the element specific metadata. This information is what
    // is visible from gst-inspect-1.0 and can also be programmatically
    // retrieved from the gst::Registry after initial registration
    // without having to load the plugin in memory.
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                "TV Filter",
                "Filter/Effect/Video/Audio",
                "Applies a TV-like effect to video",
                "Ozan Karaali <ozan@karaali.org>",
            )
        });

        Some(&*ELEMENT_METADATA)
    }

    // Create and add pad templates for our sink and source pad. These
    // are later used for actually creating the pads and beforehand
    // already provide information to GStreamer about all possible
    // pads that could exist for this type.
    //
    // Our element here can convert BGRx to BGRx or GRAY8, both being grayscale.
    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            // On the src pad, we can produce BGRx and GRAY8 of any
            // width/height and with any framerate
            let caps = gst_video::VideoCapsBuilder::new()
                .format_list([gst_video::VideoFormat::Bgrx, gst_video::VideoFormat::Gray8])
                .build();
            // The src pad template must be named "src" for basetransform
            // and specific a pad that is always there
            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            // On the sink pad, we can accept BGRx of any
            // width/height and with any framerate
            let caps = gst_video::VideoCapsBuilder::new()
                .format(gst_video::VideoFormat::Bgrx)
                .build();
            // The sink pad template must be named "sink" for basetransform
            // and specific a pad that is always there
            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            vec![src_pad_template, sink_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }
}

// Implementation of gst_base::BaseTransform virtual methods
impl BaseTransformImpl for TvFilter {
    // Configure basetransform so that we are never running in-place,
    // don't passthrough on same caps and also never call transform_ip
    // in passthrough mode (which does not matter for us here).
    //
    // We could work in-place for BGRx->BGRx but don't do here for simplicity
    // for now.
    const MODE: gst_base::subclass::BaseTransformMode =
        gst_base::subclass::BaseTransformMode::NeverInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = false;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = false;

    // Called for converting caps from one pad to another to account for any
    // changes in the media format this element is performing.
    //
    // In our case that means that:
    fn transform_caps(
        &self,
        direction: gst::PadDirection,
        caps: &gst::Caps,
        filter: Option<&gst::Caps>,
    ) -> Option<gst::Caps> {
        let other_caps = if direction == gst::PadDirection::Src {
            // For src to sink, no matter if we get asked for BGRx or GRAY8 caps, we can only
            // accept corresponding BGRx caps on the sinkpad. We will only ever get BGRx and GRAY8
            // caps here as input.
            let mut caps = caps.clone();

            for s in caps.make_mut().iter_mut() {
                s.set("format", gst_video::VideoFormat::Bgrx.to_str());
            }

            caps
        } else {
            // For the sink to src case, we will only get BGRx caps and for each of them we could
            // output the same caps or the same caps as GRAY8. We prefer GRAY8 (put it first), and
            // at a later point the caps negotiation mechanism of GStreamer will decide on which
            // one to actually produce.
            let mut caps = caps.clone();

            for s in caps.make_mut().iter_mut() {
                s.set("format", gst_video::VideoFormat::Bgrx.to_str());
            }

            caps
            // let mut gray_caps = gst::Caps::new_empty();

            // {
            //     let gray_caps = gray_caps.get_mut().unwrap();

            //     for s in caps.iter() {
            //         let mut s_gray = s.to_owned();
            //         s_gray.set("format", gst_video::VideoFormat::Gray8.to_str());
            //         gray_caps.append_structure(s_gray);
            //     }
            //     gray_caps.append(caps.clone());
            // }

            // gray_caps
        };

        gst::debug!(
            CAT,
            imp: self,
            "Transformed caps from {} to {} in direction {:?}",
            caps,
            other_caps,
            direction
        );

        // In the end we need to filter the caps through an optional filter caps to get rid of any
        // unwanted caps.
        if let Some(filter) = filter {
            Some(filter.intersect_with_mode(&other_caps, gst::CapsIntersectMode::First))
        } else {
            Some(other_caps)
        }
    }
}

impl VideoFilterImpl for TvFilter {
    // Does the actual transformation of the input buffer to the output buffer
    fn transform_frame(
        &self,
        in_frame: &gst_video::VideoFrameRef<&gst::BufferRef>,
        out_frame: &mut gst_video::VideoFrameRef<&mut gst::BufferRef>,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        // Keep a local copy of the values of all our properties at this very moment. This
        // ensures that the mutex is never locked for long and the application wouldn't
        // have to block until this function returns when getting/setting property values
        let settings = *self.settings.lock().unwrap();

        // Keep the various metadata we need for working with the video frames in
        // local variables. This saves some typing below.
        let width = in_frame.width() as usize;
        let in_stride = in_frame.plane_stride()[0] as usize;
        let in_data = in_frame.plane_data(0).unwrap();
        let out_stride = out_frame.plane_stride()[0] as usize;
        let out_format = out_frame.format();
        let out_data = out_frame.plane_data_mut(0).unwrap();

        // First check the output format. Our input format is always BGRx but the output might
        // be BGRx or GRAY8. Based on what it is we need to do processing slightly differently.
        if out_format == gst_video::VideoFormat::Bgrx {
            // Some assertions about our assumptions how the data looks like. This is only there
            // to give some further information to the compiler, in case these can be used for
            // better optimizations of the resulting code.
            //
            // If any of the assertions were not true, the code below would fail cleanly.
            assert_eq!(in_data.len() % 4, 0);
            assert_eq!(out_data.len() % 4, 0);
            assert_eq!(out_data.len() / out_stride, in_data.len() / in_stride);

            let in_line_bytes = width * 4;
            let out_line_bytes = width * 4;

            assert!(in_line_bytes <= in_stride);
            assert!(out_line_bytes <= out_stride);

            // Iterate over each line of the input and output frame, mutable for the output frame.
            // Each input line has in_stride bytes, each output line out_stride. We use the
            // chunks_exact/chunks_exact_mut iterators here for getting a chunks of that many bytes per
            // iteration and zip them together to have access to both at the same time.
            for (in_line, out_line) in in_data
                .chunks_exact(in_stride)
                .zip(out_data.chunks_exact_mut(out_stride))
            {
                // Next iterate the same way over each actual pixel in each line. Every pixel is 4
                // bytes in the input and output, so we again use the chunks_exact/chunks_exact_mut iterators
                // to give us each pixel individually and zip them together.
                //
                // Note that we take a sub-slice of the whole lines: each line can contain an
                // arbitrary amount of padding at the end (e.g. for alignment purposes) and we
                // don't want to process that padding.
                for (in_p, out_p) in in_line[..in_line_bytes]
                    .chunks_exact(4)
                    .zip(out_line[..out_line_bytes].chunks_exact_mut(4))
                {
                    assert_eq!(out_p.len(), 4);

                    // Use our above-defined function to convert a BGRx pixel with the settings to
                    // a grayscale value. Then store the same value in the red/green/blue component
                    // of the pixel.
                    // let gray = TvFilter::bgrx_to_gray(in_p, settings.shift as u8, settings.invert);
                    #[inline]
                    fn generate_random_array(size: usize) -> Vec<u8> {
                        let mut rng = rand::thread_rng();
                        (0..size).map(|_| rng.gen_range(0..=12)).collect()
                    }

                    let rand = generate_random_array(3);

                    let filtered = TvFilter::video_tv_filter(&rand, in_p);
                    out_p.copy_from_slice(&filtered)

                    // out_p[1] = gray;
                    // out_p[2] = gray;
                }
            }
        } else if out_format == gst_video::VideoFormat::Gray8 {
            assert_eq!(in_data.len() % 4, 0);
            assert_eq!(out_data.len() / out_stride, in_data.len() / in_stride);

            let in_line_bytes = width * 4;
            let out_line_bytes = width;

            assert!(in_line_bytes <= in_stride);
            assert!(out_line_bytes <= out_stride);

            // Iterate over each line of the input and output frame, mutable for the output frame.
            // Each input line has in_stride bytes, each output line out_stride. We use the
            // chunks_exact/chunks_exact_mut iterators here for getting a chunks of that many bytes per
            // iteration and zip them together to have access to both at the same time.
            for (in_line, out_line) in in_data
                .chunks_exact(in_stride)
                .zip(out_data.chunks_exact_mut(out_stride))
            {
                // Next iterate the same way over each actual pixel in each line. Every pixel is 4
                // bytes in the input and 1 byte in the output, so we again use the
                // chunks_exact/chunks_exact_mut iterators to give us each pixel individually and zip them
                // together.
                //
                // Note that we take a sub-slice of the whole lines: each line can contain an
                // arbitrary amount of padding at the end (e.g. for alignment purposes) and we
                // don't want to process that padding.
                for (in_p, out_p) in in_line[..in_line_bytes]
                    .chunks_exact(4)
                    .zip(out_line[..out_line_bytes].iter_mut())
                {
                    // Use our above-defined function to convert a BGRx pixel with the settings to
                    // a grayscale value. Then store the value in the grayscale output directly.
                    let gray = TvFilter::bgrx_to_gray(in_p, settings.shift as u8, settings.invert);
                    *out_p = in_p[0];//gray;
                }
            }
        } else {
            unimplemented!();
        }

        Ok(gst::FlowSuccess::Ok)
    }
}
