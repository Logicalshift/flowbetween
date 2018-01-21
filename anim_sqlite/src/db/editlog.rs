use super::*;
use super::db_enum::*;
use super::flo_query::*;

use std::time::Duration;

const INVALID_LAYER: u64 = 0xffffffffffffffff;

///
/// Provides the editlog trait for the animation DB
/// 
pub struct DbEditLog<TFile: FloFile+Send> {
    core: Arc<Desync<AnimationDbCore<TFile>>>
}

impl<TFile: FloFile+Send> DbEditLog<TFile> {
    ///
    /// Creates a new edit log for an animation database
    /// 
    pub fn new(core: &Arc<Desync<AnimationDbCore<TFile>>>) -> DbEditLog<TFile> {
        DbEditLog {
            core: Arc::clone(core)
        }
    }

    ///
    /// Generates a set_size entry
    /// 
    fn set_size_for_entry(core: &mut AnimationDbCore<TFile>, entry: EditLogEntry) -> AnimationEdit {
        let (width, height) = core.db.query_edit_log_size(entry.edit_id).unwrap_or((0.0, 0.0));
        AnimationEdit::SetSize(width, height)
    }

    ///
    /// Generates a SelectBrush entry
    /// 
    fn select_brush_for_entry(core: &mut AnimationDbCore<TFile>, entry: EditLogEntry) -> LayerEdit {
        // Fetch the definition from the database
        let (brush, drawing_style) = entry.brush
            .map(|(brush_id, drawing_style)|        (AnimationDbCore::get_brush_definition(&mut core.db, brush_id), drawing_style))
            .map(|(brush_or_error, drawing_style)|  (brush_or_error.unwrap_or(BrushDefinition::Simple), drawing_style))
            .unwrap_or((BrushDefinition::Simple, DrawingStyleType::Draw));
        
        // This is a paint edit, so we need the 'when' too
        let when = entry.when.unwrap_or(Duration::from_millis(0));

        // Convert drawing style
        let drawing_style = match drawing_style {
            DrawingStyleType::Draw  => BrushDrawingStyle::Draw,
            DrawingStyleType::Erase => BrushDrawingStyle::Erase
        };

        LayerEdit::Paint(when, PaintEdit::SelectBrush(brush, drawing_style))
    }

    ///
    /// Generates a BrushProperties entry
    /// 
    fn brush_properties_for_entry(core: &mut AnimationDbCore<TFile>, entry: EditLogEntry) -> LayerEdit {
        // Fetch the brush properties from the database
        let brush_properties = entry.brush_properties_id
            .map(|brush_properties_id| AnimationDbCore::get_brush_properties(&mut core.db, brush_properties_id).unwrap_or(BrushProperties::new()))
            .unwrap_or(BrushProperties::new());
        
        // This is a paint edit, so we need the 'when' too
        let when = entry.when.unwrap_or(Duration::from_millis(0));

        LayerEdit::Paint(when, PaintEdit::BrushProperties(brush_properties))
    }

    ///
    /// Retrieves the raw points associated with an entry 
    /// 
    fn raw_points_for_entry(core: &mut AnimationDbCore<TFile>, edit_id: i64) -> Arc<Vec<RawPoint>> {
        let points = core.db.query_edit_log_raw_points(edit_id).unwrap_or_else(|_err| vec![]);

        Arc::new(points)
    }

    ///
    /// Decodes a brush stroke entry
    /// 
    fn brush_stroke_for_entry(core: &mut AnimationDbCore<TFile>, entry: EditLogEntry) -> LayerEdit {
        // Fetch the points for this entry
        let points = Self::raw_points_for_entry(core, entry.edit_id);
        
        // This is a paint edit, so we need the 'when' too
        let when = entry.when.unwrap_or(Duration::from_millis(0));

        // Turn into a set of points
        LayerEdit::Paint(when, PaintEdit::BrushStroke(points))
    }

    ///
    /// Turns an edit log entry into an animation edit
    /// 
    fn animation_edit_for_entry(core: &mut AnimationDbCore<TFile>, entry: EditLogEntry) -> AnimationEdit {
        use self::EditLogType::*;

        match entry.edit_type {
            SetSize                     => Self::set_size_for_entry(core, entry),
            AddNewLayer                 => AnimationEdit::AddNewLayer(entry.layer_id.unwrap_or(INVALID_LAYER)),
            RemoveLayer                 => AnimationEdit::RemoveLayer(entry.layer_id.unwrap_or(INVALID_LAYER)),

            LayerAddKeyFrame            => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), LayerEdit::AddKeyFrame(entry.when.unwrap_or(Duration::from_millis(0)))),
            LayerRemoveKeyFrame         => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), LayerEdit::RemoveKeyFrame(entry.when.unwrap_or(Duration::from_millis(0)))),

            LayerPaintSelectBrush       => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), Self::select_brush_for_entry(core, entry)),
            LayerPaintBrushProperties   => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), Self::brush_properties_for_entry(core, entry)),
            LayerPaintBrushStroke       => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), Self::brush_stroke_for_entry(core, entry))
        }
    }
}

impl<TFile: FloFile+Send+'static> EditLog<AnimationEdit> for DbEditLog<TFile> {
    ///
    /// Retrieves the number of edits in this log
    ///
    fn length(&self) -> usize {
        self.core.sync(|core| {
            core.db.query_edit_log_length().unwrap() as usize
        })
    }

    ///
    /// Reads a range of edits from this log
    /// 
    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<AnimationEdit> {
        // Turn the indices into ranges (so we can fetch from the database)
        let current_range   = indices.next().map(|pos| pos..(pos+1));

        if let Some(mut current_range) = current_range {
            // Collect the index ranges together
            let mut ranges      = vec![];

            for next_index in indices {
                if next_index == current_range.end {
                    current_range.end += 1;
                } else {
                    ranges.push(current_range);
                    current_range = next_index..(next_index+1);
                }
            }

            ranges.push(current_range);

            // Read the edit entries
            self.core.sync(move |core| {
                let mut edits = vec![];

                for edit_range in ranges {
                    // Fetch the entries in this range
                    let entries = core.db.query_edit_log_values(edit_range.start as i64, edit_range.end as i64);

                    match entries {
                        Ok(entries) => {
                            // Extend the set of existing entries
                            edits.extend(entries.into_iter().map(|entry| Self::animation_edit_for_entry(core, entry)));
                        },

                        Err(erm) => {
                            // Whoops, got an error: pass out of this function
                            return Err(erm);
                        }
                    }
                }

                // Result is the edits we found
                Ok(edits)
            }).unwrap()
        } else {
            // Base case: no indices. We don't run anything sync here so this is always fast even if the database is busy
            vec![]
        }
    }
}
