//! Fonts and pictures, embedded in the binary (the fonts are under the SIL Open Font License, see
//! `assets/fonts/`).

use std::borrow::Cow;

use gpui::{App, AssetSource, SharedString};

/// The pictures gpui loads by path (`svg().path(…)`).
pub struct Assets;

/// A tile of dots, 240 px a side, that repeats without a seam: the fill of a plot to review.
pub const DOTS: &str = "dots.svg";
pub const DOTS_SIDE: f32 = 240.;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok((path == DOTS).then(|| Cow::Borrowed(&include_bytes!("../../assets/dots.svg")[..])))
    }

    fn list(&self, _: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(vec![DOTS.into()])
    }
}

/// The interface.
pub const SANS: &str = "Inter";
/// The large figures.
pub const DISPLAY: &str = "Inter Display";
/// Sizes, the dimension line and the title block.
pub const MONO: &str = "IBM Plex Mono";

pub fn init_fonts(cx: &mut App) {
    let fonts = [
        &include_bytes!("../../assets/fonts/Inter-Regular.otf")[..],
        include_bytes!("../../assets/fonts/Inter-Medium.otf"),
        include_bytes!("../../assets/fonts/Inter-SemiBold.otf"),
        include_bytes!("../../assets/fonts/Inter-Light.otf"),
        include_bytes!("../../assets/fonts/InterDisplay-Light.otf"),
        include_bytes!("../../assets/fonts/IBMPlexMono-Regular.ttf"),
        include_bytes!("../../assets/fonts/IBMPlexMono-Light.ttf"),
    ];
    cx.text_system().add_fonts(fonts.into_iter().map(Cow::Borrowed).collect()).expect("embedded fonts");
}
