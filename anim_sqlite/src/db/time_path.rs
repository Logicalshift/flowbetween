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
