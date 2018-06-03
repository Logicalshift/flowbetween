use super::flo_query::*;

use animation::*;

use itertools::*;

impl TimePointEntry {
    #[inline]
    fn to_time_point(&self) -> TimePoint {
        TimePoint(self.x, self.y, self.milliseconds)
    }
}

///
/// Converts a set of TimePointEntries into a TimeCurve
/// 
pub fn time_curve_from_time_points<PointIterator: IntoIterator<Item=TimePointEntry>>(points: PointIterator) -> TimeCurve { 
    // The control points should be stored in groups of three, representing the point and the past and future control points
    let points = points.into_iter()
        .tuples::<(_, _, _)>()
        .map(|(point, past, future)| (point.to_time_point(), past.to_time_point(), future.to_time_point()))
        .map(|(point, past, future)| TimeControlPoint { point, past, future })
        .collect();

    // Result is a time curve
    TimeCurve { points }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_curve_from_points() {
        let curve = time_curve_from_time_points(vec![
            TimePointEntry { x: 10.0, y: 20.0, milliseconds: 30.0 },
            TimePointEntry { x: 20.0, y: 40.0, milliseconds: 60.0 },
            TimePointEntry { x: 30.0, y: 60.0, milliseconds: 90.0 },

            TimePointEntry { x: 40.0, y: 80.0, milliseconds: 120.0 },
            TimePointEntry { x: 50.0, y: 100.0, milliseconds: 150.0 },
            TimePointEntry { x: 60.0, y: 120.0, milliseconds: 180.0 },
        ]);

        assert!(curve.points.len() == 2);
        assert!(curve.points[0].point == TimePoint(10.0, 20.0, 30.0));
        assert!(curve.points[1].point == TimePoint(40.0, 80.0, 120.0));
        assert!(curve.points[0].past == TimePoint(20.0, 40.0, 60.0));
        assert!(curve.points[1].future == TimePoint(60.0, 120.0, 180.0));
    }
}