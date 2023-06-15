use gst::glib;
use gst::prelude::*;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::BaseTransformImpl;
use gst_video::subclass::prelude::VideoFilterImpl;
use std::sync::Mutex;

use once_cell::sync::Lazy;

use fastrand;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "gstaging",
        gst::DebugColorFlags::empty(),
        Some("gstaging Element"),
    )
});

const DEFAULT_SCRATCH_LINES: u32 = 7;
const DEFAULT_COLOR_AGING: bool = true;
const DEFAULT_PITS: bool = true;
const DEFAULT_DUSTS: bool = true;

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    scratch_lines: u32,
    color_aging: bool,
    pits: bool,
    dusts: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            scratch_lines: DEFAULT_SCRATCH_LINES,
            color_aging: DEFAULT_COLOR_AGING,
            pits: DEFAULT_PITS,
            dusts: DEFAULT_DUSTS,
        }
    }
}

#[derive(Default)]
pub struct GstAgingTv {
    settings: Mutex<Settings>,
    scratches: Mutex<Vec<Scratch>>,
}

impl GstAgingTv {}

#[glib::object_subclass]
impl ObjectSubclass for GstAgingTv {
    const NAME: &'static str = "GstAgingTv";
    type Type = super::GstAgingTv;
    type ParentType = gst_video::VideoFilter;

    fn new() -> Self {
        Self::default()
    }
}

impl ObjectImpl for GstAgingTv {
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecUInt::builder("scratch-lines")
                    .nick("Scratch Lines")
                    .blurb("Number of scratch lines")
                    // .minimum(0)
                    // .maximum(100)
                    .default_value(DEFAULT_SCRATCH_LINES)
                    .build(),
                glib::ParamSpecBoolean::builder("color-aging")
                    .nick("Color Aging")
                    .blurb("Color Aging")
                    .default_value(DEFAULT_COLOR_AGING)
                    .build(),
                glib::ParamSpecBoolean::builder("pits")
                    .nick("Pits")
                    .blurb("Pits")
                    .default_value(DEFAULT_PITS)
                    .build(),
                glib::ParamSpecBoolean::builder("dusts")
                    .nick("Dusts")
                    .blurb("Dusts")
                    .default_value(DEFAULT_DUSTS)
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    // Called whenever a value of a property is changed. It can be called
    // at any time from any thread.
    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "scratch_lines" => {
                let mut settings = self.settings.lock().unwrap();
                let scratch_lines = value.get().expect("type checked upstream");
                gst::info!(
                    CAT,
                    imp: self,
                    "Changing scratch_lines from {} to {}",
                    settings.scratch_lines,
                    scratch_lines
                );
                settings.scratch_lines = scratch_lines;
            }
            "color_aging" => {
                let mut settings = self.settings.lock().unwrap();
                let color_aging = value.get().expect("type checked upstream");
                gst::info!(
                    CAT,
                    imp: self,
                    "Changing color_aging from {} to {}",
                    settings.color_aging,
                    color_aging
                );
                settings.color_aging = color_aging;
            }
            "pits" => {
                let mut settings = self.settings.lock().unwrap();
                let pits = value.get().expect("type checked upstream");
                gst::info!(
                    CAT,
                    imp: self,
                    "Changing pits from {} to {}",
                    settings.pits,
                    pits
                );
                settings.pits = pits;
            }
            "dusts" => {
                let mut settings = self.settings.lock().unwrap();
                let dusts = value.get().expect("type checked upstream");
                gst::info!(
                    CAT,
                    imp: self,
                    "Changing dusts from {} to {}",
                    settings.dusts,
                    dusts
                );
                settings.dusts = dusts;
            }
            _ => unimplemented!(),
        }
    }

    // Called whenever a value of a property is read. It can be called
    // at any time from any thread.
    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "scratch_lines" => {
                let settings = self.settings.lock().unwrap();
                settings.scratch_lines.to_value()
            }
            "color_aging" => {
                let settings = self.settings.lock().unwrap();
                settings.color_aging.to_value()
            }
            "pits" => {
                let settings = self.settings.lock().unwrap();
                settings.pits.to_value()
            }
            "dusts" => {
                let settings = self.settings.lock().unwrap();
                settings.dusts.to_value()
            }
            _ => unimplemented!(),
        }
    }
}

impl GstObjectImpl for GstAgingTv {}

impl ElementImpl for GstAgingTv {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                "Aging Filter",
                "Filter/Effect/Converter/Video",
                "Aging Filter for VCR emulation",
                "Ozan Karaali <ozan.karaali@gmail.com>",
            )
        });

        Some(&*ELEMENT_METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let caps = gst_video::VideoCapsBuilder::new()
                .format_list([gst_video::VideoFormat::Bgrx, gst_video::VideoFormat::Gray8])
                .build();

            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            let caps = gst_video::VideoCapsBuilder::new()
                .format(gst_video::VideoFormat::Bgrx)
                .build();

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

impl BaseTransformImpl for GstAgingTv {
    const MODE: gst_base::subclass::BaseTransformMode =
        gst_base::subclass::BaseTransformMode::NeverInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = false;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = false;

    fn transform_caps(
        &self,
        direction: gst::PadDirection,
        caps: &gst::Caps,
        filter: Option<&gst::Caps>,
    ) -> Option<gst::Caps> {
        let other_caps = if direction == gst::PadDirection::Src {
            let mut caps = caps.clone();

            caps
        } else {
            let mut caps = caps.clone();

            caps
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

    fn start(&self) -> Result<(), gst::ErrorMessage> {
        gst::info!(CAT, imp: self, "Starting");

        let settings = self.settings.lock().unwrap();
        let scratch_lines = settings.scratch_lines;

        let mut scratches_ex = vec![];
        for _i in 0..scratch_lines {
            scratches_ex.push(Scratch::default());
        }

        let mut scratches = self.scratches.lock().unwrap();
        *scratches = scratches_ex;

        Ok(())
    }
}

struct Scratch {
    pub life: i32,
    pub x: i32,
    pub dx: i32,
    pub init: i32,
}

impl Default for Scratch {
    fn default() -> Self {
        Scratch {
            life: fastrand::i32(..),
            x: fastrand::i32(..),
            dx: fastrand::i32(..),
            init: fastrand::i32(..),
        }
    }
}

fn coloraging(src: &[u8], dest: &mut [u8], video_area: usize, c: &mut u8) {
    let mut c_tmp = *c;

    c_tmp = c_tmp.wrapping_sub((fastrand::u8(..) >> 7) as u8);

    if c_tmp > 0x18 {
        c_tmp = 0x18;
    }

    let mut i = 0;

    while i < video_area {
        let offset = i * 4;

        let a_r = src[offset];
        let a_g = src[offset + 1];
        let a_b = src[offset + 2];
        let a_x = src[offset + 3];

        let b_r = (a_r & 0xfc) >> 2;
        let b_g = (a_g & 0xfc) >> 2;
        let b_b = (a_b & 0xfc) >> 2;

        let rnd = fastrand::u8(..) >> 3;
        let dest_r = a_r
            .wrapping_sub(b_r)
            .wrapping_add(c_tmp)
            .wrapping_add(rnd & 0x10);
        let dest_g = a_g
            .wrapping_sub(b_g)
            .wrapping_add(c_tmp)
            .wrapping_add(rnd & 0x10);
        let dest_b = a_b
            .wrapping_sub(b_b)
            .wrapping_add(c_tmp)
            .wrapping_add(rnd & 0x10);

        dest[offset] = dest_r;
        dest[offset + 1] = dest_g;
        dest[offset + 2] = dest_b;
        dest[offset + 3] = a_x;

        i += 1;
    }

    *c = c_tmp;
}

fn scratching(
    scratches: &mut [Scratch],
    dest: &mut [u8],
    width: usize,
    height: usize,
) 
{
    for scratch in scratches.iter_mut() {
        if scratch.life > 0 {
            scratch.x = scratch.x.wrapping_add(scratch.dx);

            if scratch.x < 0 || scratch.x > (width as i32 * 256) {
                scratch.life = 0;
                continue;
            }
            let mut p = &mut dest[(scratch.x as usize)..];

            let y1 = if scratch.init != 0 {
                let y1 = scratch.init;
                scratch.init = 0;
                y1
            } else {
                0
            };

            scratch.life -= 1;

            let y2 = if scratch.life > 0 {
                height as i32
            } else {
                fastrand::usize(..height) as i32
            };

            for _ in y1..y2 {
                if p.len() < 4 {
                    break;
                }
                let (a_b, a_g, a_r) = (p[0], p[1], p[2]);
                let (a_b, a_g, a_r) = (a_b & 0xFE, a_g & 0xFE, a_r & 0xFE);
                let (a_b, a_g, a_r) = (a_b.wrapping_add(0x20), a_g.wrapping_add(0x20), a_r.wrapping_add(0x20));
                let (b_b, b_g, b_r) = (a_b & 0x10, a_g & 0x10, a_r & 0x10);
                p[0] = a_b | ((b_b - (b_b >> 4)) as u8);
                p[1] = a_g | ((b_g - (b_g >> 4)) as u8);
                p[2] = a_r | ((b_r - (b_r >> 4)) as u8);
                
                if p.len() > width * 4 {
                    p = &mut p[width * 4..];
                } else {
                    break;
                }
            }
            
            
        } else {
            if fastrand::u32(..) & 0xF0000000 == 0 {
                scratch.life = 2 + (fastrand::u32(..) >> 27) as i32;
                scratch.x = (fastrand::i32(..) % (width as i32)) * 256;
                scratch.dx = fastrand::i32(..) >> 23;
                scratch.init = (fastrand::i32(..) % (height as i32 - 1)) + 1;
            }
        }
    }
}

impl VideoFilterImpl for GstAgingTv {
    fn transform_frame(
        &self,
        in_frame: &gst_video::VideoFrameRef<&gst::BufferRef>,
        out_frame: &mut gst_video::VideoFrameRef<&mut gst::BufferRef>,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        let settings = *self.settings.lock().unwrap();

        let width = in_frame.width() as usize;
        let height = in_frame.height() as usize;
        let in_stride = in_frame.plane_stride()[0] as usize;
        let in_data = in_frame.plane_data(0).unwrap();
        let out_stride = out_frame.plane_stride()[0] as usize;
        let out_format = out_frame.format();
        let mut out_data = out_frame.plane_data_mut(0).unwrap();

        let mut area_scale = (width * height) as f64 / (640 * 480) as f64;
        if area_scale <= 0.0 {
            area_scale = 1.0;
        }

        let video_size = in_stride * height / 4;

        // color aging
        if settings.color_aging {
            coloraging (in_data, out_data, video_size, &mut 0x18);
        }
        else{
            out_data.copy_from_slice(in_data);
        }

        // vertical scratches
        let mut scratches = self.scratches.lock().unwrap();
        scratching(&mut scratches, out_data, width, height);

        // horizontal scratches: todo functionize
        let scratch_lines: Vec<usize> = (0..settings.scratch_lines)
            .map(|_| fastrand::usize(..height))
            .collect();

        if out_format == gst_video::VideoFormat::Bgrx {
            assert_eq!(in_data.len() % 4, 0);
            assert_eq!(out_data.len() % 4, 0);
            assert_eq!(out_data.len() / out_stride, in_data.len() / in_stride);

            let in_line_bytes = width * 4;
            let out_line_bytes = width * 4;

            assert!(in_line_bytes <= in_stride);
            assert!(out_line_bytes <= out_stride);

            for (y, (in_line, out_line)) in in_data.chunks_exact(in_stride).zip(out_data.chunks_exact_mut(out_stride)).enumerate().take(height)
            {
                if scratch_lines.contains(&y) {
                    for (x, pixel) in out_line[..out_line_bytes].chunks_exact_mut(4).enumerate() {
                        // Randomly adjust the color of the pixel
                        let noise = fastrand::u8(16..235);
                        pixel[0] = noise;
                        pixel[1] = noise;
                        pixel[2] = noise;
                    }
                }

            }
        }

        // if pits true, we need to add pits
        if settings.pits {}

        Ok(gst::FlowSuccess::Ok)
    }
}
