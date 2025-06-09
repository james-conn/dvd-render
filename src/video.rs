use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video as gst_vid;
use gstreamer_pbutils as gst_pbutils;
use gst::prelude::{Cast, GstBinExtManual, ElementExt, ObjectExt};
use std::path::Path;
use std::sync::Arc;
use crate::render::VideoSrc;

pub struct DvdEncoder<S> {
	src: S
}

#[derive(Clone)]
struct ArcFrame(Arc<Vec<u8>>);

impl ArcFrame {
	fn new(frame: image::RgbaImage) -> Self {
		Self(Arc::new(frame.to_vec()))
	}
}

impl AsRef<[u8]> for ArcFrame {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl<S: VideoSrc> DvdEncoder<S> {
	pub fn new(src: S) -> Self {
		Self { src }
	}

	// mostly stitched together example code from the gstreamer gitlab
	pub fn save_video_to<P: AsRef<Path>>(self, path: P) {
		gst::init().unwrap();

		let pipeline = gst::Pipeline::default();

		let vid_info = gst_vid::VideoInfo::builder(
			gst_vid::VideoFormat::Rgba,
			self.src.width(),
			self.src.height()
		).fps(gst::Fraction::new(self.src.framerate().get() as i32, 1)).build().unwrap();

		let appsrc = gst_app::AppSrc::builder()
			.caps(&vid_info.to_caps().unwrap())
			.format(gst::Format::Time)
			.build();

		let encodebin = gst::ElementFactory::make("encodebin").build().unwrap();
		encodebin.set_property("profile", gst_pbutils::EncodingContainerProfile::builder(
			&gst::Caps::builder("video/x-matroska").build()
		).add_profile(
			gst_pbutils::EncodingVideoProfile::builder(&gst::Caps::builder("video/x-vp9").build()).build()
		).build());
		let sink = gst::ElementFactory::make("filesink")
			.property("location", path.as_ref())
			.build().unwrap();

		pipeline.add_many([appsrc.upcast_ref(), &encodebin, &sink]).unwrap();
		gst::Element::link_many([appsrc.upcast_ref(), &encodebin, &sink]).unwrap();

		let frametime = gst::ClockTime::SECOND / self.src.framerate().get() as u64;
		let mut frames_iter = self.src.flat_map(|frame| {
			let img = ArcFrame::new(frame.img);
			std::iter::repeat_n(img, frame.frame_hold.get() as usize)
		});
		let mut n: u64 = 0;
		appsrc.set_callbacks(gst_app::AppSrcCallbacks::builder().need_data(move |appsrc, _| {
			let Some(frame) = frames_iter.next() else {
				let _ = appsrc.end_of_stream();
				return;
			};

			let mut buffer = gst::Buffer::from_slice(ArcFrame::clone(&frame));
			buffer.get_mut().unwrap().set_pts(frametime * n);
			appsrc.push_buffer(buffer).unwrap();
			n += 1;
		}).build());

		pipeline.set_state(gst::State::Playing).unwrap();

		let bus = pipeline.bus().unwrap();

		for msg in bus.iter_timed(gst::ClockTime::NONE) {
			match msg.view() {
				gst::MessageView::Eos(..) => break,
				gst::MessageView::Error(err) => {
					pipeline.set_state(gst::State::Null).unwrap();
					panic!("pipeline failed {err}");
				},
				_ => ()
			}
		}

		pipeline.set_state(gst::State::Null).unwrap();
	}
}
