use hsluv::*;

///
/// Possible formats of a colour value
///
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum ColorFormat {
    Rgba,
    Hsluv
}

///
/// Representation of a colour
///
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Color {
    Rgba(f32, f32, f32, f32),
    Hsluv(f32, f32, f32, f32)
}

impl PartialEq for Color {
    fn eq(&self, col: &Color) -> bool { 
        use self::Color::*;

        // Colors are equal if they're 'close enough'
        let distance = match (self, col) {
            (Rgba(r1, g1, b1, a1), Rgba(r2, g2, b2, a2))    => { (r1-r2)*(r1-r2) + (g1-g2)*(g1-g2) + (b1-b2)*(b1-b2) + (a1-a2)*(a1-a2) }
            (Hsluv(h1, s1, l1, a1), Hsluv(h2, s2, l2, a2))  => { (h1-h2)*(h1-h2) + (s1-s2)*(s1-s2) + (l1-l2)*(l1-l2) + (a1-a2)*(a1-a2) }
            _                                               => { return false; }
        };

        distance < (0.0001 * 0.0001)
    }
}

impl Color {
    ///
    /// Returns this colour as RGBA components
    ///
    pub fn to_rgba_components(&self) -> (f32, f32, f32, f32) {
        match self {
            &Color::Rgba(r, g, b, a) => (r, g, b, a),

            &Color::Hsluv(h, s, l, a) => {
                let (r, g, b) = hsluv_to_rgb((h as f64, s as f64, l as f64));
                (r as f32, g as f32, b as f32, a)
            }
        }
    }

    ///
    /// Returns this colour as HSLUV components
    ///
    pub fn to_hsluv_components(&self) -> (f32, f32, f32, f32) {
        match self {
            &Color::Hsluv(h, s, l, a) => (h, s, l, a),

            &Color::Rgba(r, g, b, a) => {
                let (h, s, l) = rgb_to_hsluv((r as f64, g as f64, b as f64));
                let s = if l <= 0.0 { 100.0 } else { s };
                (h as f32, s as f32, l as f32, a)
            }
        }
    }

    ///
    /// Converts this colour to another format
    ///
    #[inline]
    pub fn to_format(&self, format: ColorFormat) -> Color {
        let (r, g, b, a) = self.to_rgba_components();

        match format {
            ColorFormat::Rgba   => Color::Rgba(r, g, b, a),
            ColorFormat::Hsluv  => {
                let (h, s, l) = rgb_to_hsluv((r as f64, g as f64, b as f64));
                let s = if l <= 0.0 { 100.0 } else { s };
                Color::Hsluv(h as f32, s as f32, l as f32, a)
            }
        }
    }

    ///
    /// Returns a new colour that's the same as this one except with a different alpha value
    ///
    pub fn with_alpha(&self, new_alpha: f32) -> Color {
        match self {
            &Color::Rgba(r, g, b, _)    => Color::Rgba(r, g, b, new_alpha),
            &Color::Hsluv(h, s, l, _)   => Color::Hsluv(h, s, l, new_alpha)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_convert_rgba_to_hsluv() {
        let rgb     = Color::Rgba(0.5, 0.7, 0.2, 0.9);
        let hsluv   = rgb.to_format(ColorFormat::Hsluv);

        if let Color::Hsluv(h, s, l, a) = hsluv {
            assert!((h-110.3).abs() < 0.1);
            assert!((s-89.5).abs() < 0.1);
            assert!((l-67.1).abs() < 0.1);
            assert!(a == 0.9);
        } else {
            assert!(false)
        }
    }

    #[test]
    fn can_convert_hsluv_to_rgba() {
        let hsluv   = Color::Hsluv(24.0, 66.0, 60.0, 0.8);
        let rgb     = hsluv.to_format(ColorFormat::Rgba);

        if let Color::Rgba(r, g, b, a) = rgb {
            assert!((r-0.89) < 0.1);
            assert!((g-0.43) < 0.1);
            assert!((b-0.38) < 0.1);
            assert!(a == 0.8);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn can_get_rgba_components_from_hsluv() {
        let hsluv           = Color::Hsluv(24.0, 66.0, 60.0, 0.8);
        let (r, g, b, a)    = hsluv.to_rgba_components();

        assert!((r-0.89) < 0.1);
        assert!((g-0.43) < 0.1);
        assert!((b-0.38) < 0.1);
        assert!(a == 0.8);
    }
}
