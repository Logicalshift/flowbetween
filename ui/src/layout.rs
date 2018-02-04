use super::control::*;
use super::property::*;

///
/// Converts Positions to actual coordinates
///
pub struct PositionLayout {
    /// Most recent position
    last_pos: f32,

    /// The starting position in this layout
    start: f32,

    /// The length of the axis that this layout is along
    length: f32,
}

impl PositionLayout {
    ///
    /// Creates a new position layout
    ///
    pub fn new(length: f32) -> PositionLayout {
        PositionLayout { last_pos: 0.0, start: 0.0, length: length }
    }

    ///
    /// Converts a point to an absolute position within this layout
    ///
    pub fn to_abs(&mut self, pos: &Position) -> f32 {
        use Position::*;

        let next_pos = match pos {
            &At(pos)            => pos,
            &Offset(pos)        => self.last_pos + pos,
            &Stretch(_ratio)    => self.last_pos,
            &Start              => self.start,
            &End                => self.start + self.length,
            &After              => self.last_pos,

            &Floating(Property::Float(pos), offset) => (pos as f32) + offset,
            &Floating(_, _)                         => 0.0
        };

        self.last_pos = next_pos;

        next_pos
    }
}

///
/// Perfoms layout with a bounding box
///
pub struct BoundsLayout {
    x_layout: PositionLayout,
    y_layout: PositionLayout
}

impl BoundsLayout {
    ///
    /// Creates a new bounds layout that acts within a particular bounding box
    ///
    pub fn new(width: f32, height: f32) -> BoundsLayout {
        BoundsLayout {
            x_layout: PositionLayout::new(width),
            y_layout: PositionLayout::new(height)
        }
    }

    ///
    /// Lays out a single point within this bounding box
    ///
    pub fn to_abs(&mut self, x: &Position, y: &Position) -> (f32, f32) {
        (self.x_layout.to_abs(x), self.y_layout.to_abs(y))
    }

    ///
    /// Lays out some rectangular bounds within this bounding box
    ///
    pub fn to_abs_bounds(&mut self, bounds: &Bounds) -> ((f32, f32), (f32, f32)) {
        (self.to_abs(&bounds.x1, &bounds.y1), self.to_abs(&bounds.x2, &bounds.y2))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Position::*;

    #[test]
    pub fn position_layout_begins_at_start() {
        let mut layout = PositionLayout::new(100.0);

        assert!(layout.to_abs(&Start) == 0.0);
    }

    #[test]
    pub fn position_layout_after_pos_is_initially_zero() {
        let mut layout = PositionLayout::new(100.0);

        assert!(layout.to_abs(&After) == 0.0);
    }

    #[test]
    pub fn position_layout_absolute_is_absolute() {
        let mut layout = PositionLayout::new(100.0);

        assert!(layout.to_abs(&At(50.0)) == 50.0);
    }

    #[test]
    pub fn position_layout_end_is_end() {
        let mut layout = PositionLayout::new(100.0);

        assert!(layout.to_abs(&End) == 100.0);
    }

    #[test]
    pub fn position_layout_sequence() {
        let mut layout = PositionLayout::new(100.0);

        assert!(layout.to_abs(&Start) == 0.0);
        assert!(layout.to_abs(&Offset(20.0)) == 20.0);

        assert!(layout.to_abs(&After) == 20.0);
        assert!(layout.to_abs(&Offset(20.0)) == 40.0);

        assert!(layout.to_abs(&After) == 40.0);
        assert!(layout.to_abs(&End) == 100.0);
    }

    #[test]
    pub fn bounds_layout_begins_at_start() {
        let mut layout = BoundsLayout::new(200.0, 100.0);

        assert!(layout.to_abs(&Start, &Start) == (0.0, 0.0));
    }

    #[test]
    pub fn bounds_layout_after_pos_is_initially_zero() {
        let mut layout = BoundsLayout::new(200.0, 100.0);

        assert!(layout.to_abs(&After, &After) == (0.0, 0.0));
    }

    #[test]
    pub fn bounds_layout_absolute_is_absolute() {
        let mut layout = BoundsLayout::new(200.0, 100.0);

        assert!(layout.to_abs(&At(20.0), &At(50.0)) == (20.0, 50.0));
    }

    #[test]
    pub fn bounds_layout_end_is_end() {
        let mut layout = BoundsLayout::new(200.0, 100.0);

        assert!(layout.to_abs(&End, &End) == (200.0, 100.0));
    }

    #[test]
    pub fn bounds_layout_can_layout_controls_vertical() {
        let mut layout = BoundsLayout::new(20.0, 200.0);

        // Like we're laying out a vertical tool pane
        assert!(layout.to_abs_bounds(&Bounds::next_vert(20.0)) == ((0.0, 0.0), (20.0, 20.0)));
        assert!(layout.to_abs_bounds(&Bounds::next_vert(20.0)) == ((0.0, 20.0), (20.0, 40.0)));
        assert!(layout.to_abs_bounds(&Bounds::next_vert(20.0)) == ((0.0, 40.0), (20.0, 60.0)));

        assert!(layout.to_abs_bounds(&Bounds::fill_vert()) == ((0.0, 60.0), (20.0, 200.0)));
    }

    #[test]
    pub fn bounds_layout_can_layout_controls_horizontal() {
        let mut layout = BoundsLayout::new(200.0, 20.0);

        // Like we're laying out a vertical tool pane
        assert!(layout.to_abs_bounds(&Bounds::next_horiz(20.0)) == ((0.0, 0.0), (20.0, 20.0)));
        assert!(layout.to_abs_bounds(&Bounds::next_horiz(20.0)) == ((20.0, 0.0), (40.0, 20.0)));
        assert!(layout.to_abs_bounds(&Bounds::next_horiz(20.0)) == ((40.0, 0.0), (60.0, 20.0)));

        assert!(layout.to_abs_bounds(&Bounds::fill_horiz()) == ((60.0, 0.0), (200.0, 20.0)));
    }
}
