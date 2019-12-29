use flo_canvas;
use nanovg::*;

///
/// Possible NanoVgPaint instructions
///
pub enum NanoVgPaint {
    Color(Color),
    Gradient(Gradient)
}

impl From<flo_canvas::Color> for NanoVgPaint {
    fn from(item: flo_canvas::Color) -> NanoVgPaint {
        let (r, g, b, a) = item.to_rgba_components();

        NanoVgPaint::Color(Color::new(r, g, b, a))
    }
}

impl Paint for NanoVgPaint {
    #[inline]
    fn fill(&self, context: &Context) {
        use self::NanoVgPaint::*;
        match self {
            &Color(ref c)       => c.fill(context),
            &Gradient(ref g)    => g.fill(context)
        }
    }

    #[inline]
    fn stroke(&self, context: &Context) {
        use self::NanoVgPaint::*;
        match self {
            &Color(ref c)       => c.stroke(context),
            &Gradient(ref g)    => g.stroke(context)
        }
    }
}


impl<'a> Paint for &'a NanoVgPaint {
    #[inline]
    fn fill(&self, context: &Context) {
        use self::NanoVgPaint::*;
        match *self {
            &Color(ref c)       => c.fill(context),
            &Gradient(ref g)    => g.fill(context)
        }
    }

    #[inline]
    fn stroke(&self, context: &Context) {
        use self::NanoVgPaint::*;
        match *self {
            &Color(ref c)       => c.stroke(context),
            &Gradient(ref g)    => g.stroke(context)
        }
    }
}
