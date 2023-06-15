use gst::glib;
use gst::prelude::*;

mod imp;

// The public Rust wrapper type for our element
glib::wrapper! {
    pub struct GstAgingTv(ObjectSubclass<imp::GstAgingTv>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

// Registers the type for our element, and then registers in GStreamer under
// the name "rsidentity" for being able to instantiate it via e.g.
// gst::ElementFactory::make().
pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        "gstagingtv",
        gst::Rank::None,
        GstAgingTv::static_type(),
    )
}
