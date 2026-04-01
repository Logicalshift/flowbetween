use super::source::*;
use super::target::*;
use super::super::traits::*;

impl PathComponent {
    ///
    /// Generates a serialized version of this path component on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::PathComponent::*;

        match self {
            Move(p)             => { data.write_chr('M'); data.write_f64(p.position.0); data.write_f64(p.position.1); }
            Line(p)             => { data.write_chr('L'); data.write_f64(p.position.0); data.write_f64(p.position.1); }
            Bezier(p1, p2, p3)  => { 
                data.write_chr('C'); 
                data.write_f64(p1.position.0); data.write_f64(p1.position.1); 
                data.write_f64(p2.position.0); data.write_f64(p2.position.1); 
                data.write_f64(p3.position.0); data.write_f64(p3.position.1);
            }
            Close               => { data.write_chr('X'); }
        }
    }

    ///
    /// Generates a serialized version of this path component on the specified data target
    ///
    pub fn serialize_next<Tgt: AnimationDataTarget>(&self, last: &PathPoint, data: &mut Tgt) -> PathPoint {
        use self::PathComponent::*;

        match self {
            Move(p)             => { data.write_chr('M'); data.write_next_f64(last.position.0, p.position.0); data.write_next_f64(last.position.1, p.position.1); p.clone() }
            Line(p)             => { data.write_chr('L'); data.write_next_f64(last.position.0, p.position.0); data.write_next_f64(last.position.1, p.position.1); p.clone() }
            Bezier(p1, p2, p3)  => { 
                data.write_chr('C');
                data.write_next_f64(last.position.0, p1.position.0); data.write_next_f64(last.position.1, p1.position.1);
                data.write_next_f64(p1.position.0, p2.position.0);   data.write_next_f64(p1.position.1, p2.position.1);
                data.write_next_f64(p2.position.0, p3.position.0);   data.write_next_f64(p2.position.1, p3.position.1);
                p3.clone()
            }
            Close               => { data.write_chr('X'); last.clone() }
        }
    }

    ///
    /// Deserializes this path component from a source stream
    ///
    pub fn deserialize<Src: AnimationDataSource>(data: &mut Src) -> Option<PathComponent> {
        match data.next_chr() {
            'M' => { Some(PathComponent::Move(PathPoint { position: (data.next_f64(), data.next_f64()) })) }
            'L' => { Some(PathComponent::Line(PathPoint { position: (data.next_f64(), data.next_f64()) })) }
            'C' => { Some(PathComponent::Bezier(PathPoint { position: (data.next_f64(), data.next_f64()) }, PathPoint { position: (data.next_f64(), data.next_f64()) }, PathPoint { position: (data.next_f64(), data.next_f64()) })) }
            'X' => { Some(PathComponent::Close) }
            _   => { None }
        }
    }

    ///
    /// Deserializes this path component from a source stream
    ///
    pub fn deserialize_next<Src: AnimationDataSource>(last: &PathPoint, data: &mut Src) -> Option<(PathComponent, PathPoint)> {
        match data.next_chr() {
            'M' => { let p = PathPoint { position: (data.next_f64_offset(last.position.0), data.next_f64_offset(last.position.1)) }; Some((PathComponent::Move(p), p)) }
            'L' => { let p = PathPoint { position: (data.next_f64_offset(last.position.0), data.next_f64_offset(last.position.1)) }; Some((PathComponent::Line(p), p)) }
            'C' => { 
                let p1 = PathPoint { position: (data.next_f64_offset(last.position.0), data.next_f64_offset(last.position.1)) };
                let p2 = PathPoint { position: (data.next_f64_offset(p1.position.0), data.next_f64_offset(p1.position.1)) };
                let p3 = PathPoint { position: (data.next_f64_offset(p2.position.0), data.next_f64_offset(p2.position.1)) };

                Some((PathComponent::Bezier(p1, p2, p3), p3)) 
            }
            'X' => { Some((PathComponent::Close, *last)) }
            _   => { None }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn close_1() {
        let mut encoded = String::new();
        PathComponent::Close.serialize(&mut encoded);

        assert!(PathComponent::deserialize(&mut encoded.chars()) == Some(PathComponent::Close));
    }

    #[test]
    fn move_1() {
        let mut encoded = String::new();
        PathComponent::Move(PathPoint::new(10.0, 15.0)).serialize(&mut encoded);

        assert!(PathComponent::deserialize(&mut encoded.chars()) == Some(PathComponent::Move(PathPoint::new(10.0, 15.0))));
    }

    #[test]
    fn move_next_1() {
        let mut encoded = String::new();
        let last        = PathPoint::new(50.0, 60.0);
        PathComponent::Move(PathPoint::new(10.0, 15.0)).serialize_next(&last, &mut encoded);

        assert!(PathComponent::deserialize_next(&last, &mut encoded.chars()) == Some((PathComponent::Move(PathPoint::new(10.0, 15.0)), PathPoint::new(10.0, 15.0))));
    }

    #[test]
    fn line_1() {
        let mut encoded = String::new();
        PathComponent::Line(PathPoint::new(10.0, 15.0)).serialize(&mut encoded);

        assert!(PathComponent::deserialize(&mut encoded.chars()) == Some(PathComponent::Line(PathPoint::new(10.0, 15.0))));
    }

    #[test]
    fn line_next_1() {
        let mut encoded = String::new();
        let last        = PathPoint::new(50.0, 60.0);
        PathComponent::Line(PathPoint::new(10.0, 15.0)).serialize_next(&last, &mut encoded);

        assert!(PathComponent::deserialize_next(&last, &mut encoded.chars()) == Some((PathComponent::Line(PathPoint::new(10.0, 15.0)), PathPoint::new(10.0, 15.0))));
    }

    #[test]
    fn bezier_1() {
        let mut encoded = String::new();
        PathComponent::Bezier(PathPoint::new(10.0, 15.0), PathPoint::new(11.0, 16.0), PathPoint::new(12.0, 17.0)).serialize(&mut encoded);

        assert!(PathComponent::deserialize(&mut encoded.chars()) == Some(PathComponent::Bezier(PathPoint::new(10.0, 15.0), PathPoint::new(11.0, 16.0), PathPoint::new(12.0, 17.0))));
    }

    #[test]
    fn bezier_next_1() {
        let mut encoded = String::new();
        let last        = PathPoint::new(50.0, 60.0);
        PathComponent::Bezier(PathPoint::new(10.0, 15.0), PathPoint::new(11.0, 16.0), PathPoint::new(12.0, 17.0)).serialize_next(&last, &mut encoded);

        assert!(PathComponent::deserialize_next(&last, &mut encoded.chars()) == Some((PathComponent::Bezier(PathPoint::new(10.0, 15.0), PathPoint::new(11.0, 16.0), PathPoint::new(12.0, 17.0)), PathPoint::new(12.0, 17.0))));
    }
}
