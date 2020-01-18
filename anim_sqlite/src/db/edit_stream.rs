use super::*;
use super::db_enum::*;
use super::flo_query::*;

use futures::task;
use futures::task::{Poll, Context};

use std::pin::*;
use std::ops::Range;
use std::time::Duration;
use std::collections::VecDeque;

const BUFFER_SIZE: usize = 100;
const INVALID_LAYER: u64 = 0xffffffffffffffff;

///
/// Provides the editlog trait for the animation DB
///
pub struct EditStream<TFile: FloFile+Unpin+Send> {
    /// The database core
    core: Arc<Desync<AnimationDbCore<TFile>>>,

    /// Buffer of items that have been read from the databasse
    buffer: Arc<Mutex<EditStreamBuffer>>,

    /// The range of items that have been read
    range: Range<usize>
}

struct EditStreamBuffer {
    /// Items waiting to be supplied to the stream
    loaded: VecDeque<AnimationEdit>,

    /// The next item to read
    next: usize,

    /// Set to true if the buffer has a 'fill' operation queued
    filling: bool
}

impl<TFile: Unpin+FloFile+Send> EditStream<TFile> {
    ///
    /// Creates a new edit log for an animation database
    ///
    pub fn new(core: &Arc<Desync<AnimationDbCore<TFile>>>, range: Range<usize>) -> EditStream<TFile> {
        // Create an empty buffer at the start of the range
        let buffer = EditStreamBuffer {
            loaded:     VecDeque::new(),
            next:       range.start,
            filling:    false
        };

        // Stream over the specified range of IDs
        EditStream {
            core:   Arc::clone(core),
            buffer: Arc::new(Mutex::new(buffer)),
            range:  range
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
        let drawing_style = drawing_style.into();

        // Paint edits create elements, so there may be an element ID
        // (These are optional, but should have been assigned during the commit process)
        let element_id = ElementId::from(entry.element_id);

        LayerEdit::Paint(when, PaintEdit::SelectBrush(element_id, brush, drawing_style))
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

        // Paint edits create elements, so there may be an element ID
        // (These are optional, but should have been assigned during the commit process)
        let element_id = ElementId::from(entry.element_id);

        LayerEdit::Paint(when, PaintEdit::BrushProperties(element_id, brush_properties))
    }

    ///
    /// Retrieves the raw points associated with an entry
    ///
    fn raw_points_for_entry(core: &mut AnimationDbCore<TFile>, edit_id: i64) -> Arc<Vec<RawPoint>> {
        let points = core.db.query_edit_log_raw_points(edit_id).unwrap_or_else(|_err| vec![]);

        Arc::new(points)
    }

    ///
    /// Retrieves the path components associated with a particular edit log ID
    ///
    fn path_components_for_entry(core: &mut AnimationDbCore<TFile>, edit_id: i64) -> Result<Vec<PathComponent>> {
        let path_id     = core.db.query_edit_log_path_id(edit_id)?;
        let components  = core.db.query_path_components(path_id)?;

        Ok(components)
    }

    ///
    /// Decodes a brush stroke entry
    ///
    fn brush_stroke_for_entry(core: &mut AnimationDbCore<TFile>, entry: EditLogEntry) -> LayerEdit {
        // Fetch the points for this entry
        let points = Self::raw_points_for_entry(core, entry.edit_id);

        // This is a paint edit, so we need the 'when' too
        let when = entry.when.unwrap_or(Duration::from_millis(0));

        // Paint edits create elements, so there may be an element ID
        // (These are optional, but should have been assigned during the commit process)
        let element_id = ElementId::from(entry.element_id);

        // Turn into a set of points
        LayerEdit::Paint(when, PaintEdit::BrushStroke(element_id, points))
    }

    ///
    /// Decodes a 'create path' entry
    ///
    fn create_path_for_entry(core: &mut AnimationDbCore<TFile>, entry: EditLogEntry) -> LayerEdit {
        // A create path is just a 'when' and a bunch of points
        let points      = Self::path_components_for_entry(core, entry.edit_id).unwrap_or_else(|_err| vec![]);
        let when        = entry.when.unwrap_or(Duration::from_millis(0));
        let element_id  = ElementId::from(entry.element_id);

        LayerEdit::Path(when, PathEdit::CreatePath(element_id, Arc::new(points)))
    }

    ///
    /// Decodes a path 'brush properties' entry
    ///
    fn path_properties_for_entry(core: &mut AnimationDbCore<TFile>, entry: EditLogEntry) -> LayerEdit {
        // Decode the brush properties
        let properties  = entry.brush_properties_id
            .map(|brush_properties_id| AnimationDbCore::get_brush_properties(&mut core.db, brush_properties_id).unwrap_or(BrushProperties::new()))
            .unwrap_or(BrushProperties::new());
        let when        = entry.when.unwrap_or(Duration::from_millis(0));
        let element_id  = ElementId::from(entry.element_id);

        LayerEdit::Path(when, PathEdit::BrushProperties(element_id, properties))
    }

    ///
    /// Decodes a path 'brush' entry
    ///
    fn path_brush_for_entry(core: &mut AnimationDbCore<TFile>, entry: EditLogEntry) -> LayerEdit {
        // Decode the brush to use for this path
        let (brush, drawing_style) = entry.brush
            .map(|(brush_id, drawing_style)|        (AnimationDbCore::get_brush_definition(&mut core.db, brush_id), drawing_style))
            .map(|(brush_or_error, drawing_style)|  (brush_or_error.unwrap_or(BrushDefinition::Simple), drawing_style))
            .unwrap_or((BrushDefinition::Simple, DrawingStyleType::Draw));
        let when        = entry.when.unwrap_or(Duration::from_millis(0));
        let element_id  = ElementId::from(entry.element_id);

        LayerEdit::Path(when, PathEdit::SelectBrush(element_id, brush, drawing_style.into()))
    }

    ///
    /// Reads the elements for an entry
    ///
    fn elements_for_entry(core: &mut AnimationDbCore<TFile>, entry: EditLogEntry) -> Vec<ElementId> {
        core.db.query_edit_log_elements(entry.edit_id)
            .unwrap()
            .into_iter()
            .map(|element_id| ElementId::from(Some(element_id)))
            .collect()
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
            LayerSetName                => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), LayerEdit::SetName(core.db.query_edit_log_string(entry.edit_id, 0).unwrap())),
            LayerSetOrdering            => unimplemented!(),

            LayerPaintSelectBrush       => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), Self::select_brush_for_entry(core, entry)),
            LayerPaintBrushProperties   => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), Self::brush_properties_for_entry(core, entry)),
            LayerPaintBrushStroke       => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), Self::brush_stroke_for_entry(core, entry)),

            LayerPathCreatePath         => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), Self::create_path_for_entry(core, entry)),
            LayerPathSelectBrush        => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), Self::path_brush_for_entry(core, entry)),
            LayerPathBrushProperties    => AnimationEdit::Layer(entry.layer_id.unwrap_or(INVALID_LAYER), Self::path_properties_for_entry(core, entry)),

            MotionCreate                => AnimationEdit::Motion(Self::elements_for_entry(core, entry)[0], MotionEdit::Create),
            MotionDelete                => AnimationEdit::Motion(Self::elements_for_entry(core, entry)[0], MotionEdit::Delete),
            MotionSetType               => unimplemented!(),
            MotionSetOrigin             => unimplemented!(),
            MotionSetPath               => unimplemented!(),

            ElementAddAttachment        => unimplemented!(),
            ElementRemoveAttachment     => unimplemented!(),
            ElementSetControlPoints     => unimplemented!(),
            ElementSetPath              => unimplemented!(),
            ElementOrderInFront         => unimplemented!(),
            ElementOrderBehind          => unimplemented!(),
            ElementOrderToTop           => unimplemented!(),
            ElementOrderToBottom        => unimplemented!(),
            ElementOrderBefore          => unimplemented!(),
            ElementDelete               => AnimationEdit::Element(Self::elements_for_entry(core, entry), ElementEdit::Delete),
            ElementDetachFromFrame      => unimplemented!()
        }
    }

    ///
    /// Reads a range of edits from the SQLite edit log
    ///
    fn read(core: &mut AnimationDbCore<TFile>, indices: &mut dyn Iterator<Item=usize>) -> Result<Vec<AnimationEdit>> {
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
        } else {
            // Base case: no indices. We don't run anything sync here so this is always fast even if the database is busy
            Ok(vec![])
        }
    }
}

impl<TFile: Unpin+FloFile+Send+'static> Stream for EditStream<TFile>
where Self: Unpin {
    type Item = AnimationEdit;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<AnimationEdit>> {
        let range       = &self.range;
        let buffer_ref  = Arc::clone(&self.buffer);
        let mut buffer  = self.buffer.lock().unwrap();

        if let Some(next_item) = buffer.loaded.pop_front() {
            // Trigger filling the buffer (if a filling operation is not already queued)
            if !buffer.filling {
                let range = range.clone();

                buffer.filling = true;
                self.core.desync(move |core| {
                    EditStreamBuffer::fill(&*buffer_ref, core, range, None);
                });
            }

            // If there is already an entry in the buffer, return it
            Poll::Ready(Some(next_item))
        } else if buffer.next >= self.range.end {
            // Stop if we've reached then end of the stream
            Poll::Ready(None)
        } else {
            // Trigger filling the buffer
            let range   = range.clone();
            let waker   = context.waker().clone();

            buffer.filling = true;
            self.core.desync(move |core| {
                EditStreamBuffer::fill(&*buffer_ref, core, range, Some(waker));
            });

            // Buffer is not ready yet
            Poll::Pending
        }
    }
}

impl EditStreamBuffer {
    ///
    /// Fills a buffer stored in a mutex
    ///
    fn fill<TFile: FloFile+Unpin+Send>(buffer: &Mutex<EditStreamBuffer>, core: &mut AnimationDbCore<TFile>, range: Range<usize>, notify: Option<task::Waker>) {
        // Note that the locking behaviour here assumes we're only running one fill in parallel (possibly with a stream reader)
        // This allows us to do the DB read while the buffer is unlocked

        // Function to retrieve the number of loaded elements
        let num_loaded  = || { buffer.lock().unwrap().loaded.len() };

        // Function to retrieve the index of the next item to retrieve
        let get_next    = || { buffer.lock().unwrap().next };

        // Iterate until we reach the end of the range or fill the buffer
        while get_next() < range.end && num_loaded() < BUFFER_SIZE {
            // Want to read exactly enough entries to fill the buffer
            let num_missing = BUFFER_SIZE - num_loaded();
            let next        = get_next();
            let end         = (next + num_missing).min(range.end);

            // Turn into the range of IDs to load
            let load_range  = next..end;

            // Load the edits from the database
            let loaded      = EditStream::read(core, &mut load_range.into_iter()).unwrap();

            {
                let mut buffer = buffer.lock().unwrap();

                // Store in the buffer
                loaded.into_iter()
                    .for_each(|entry| buffer.loaded.push_back(entry));

                // Read from the end of the current range
                buffer.next       = end;
            }
        }

        // Note that the buffer has been filled
        buffer.lock().unwrap().filling = false;

        // Notify the task, if there is one
        notify.map(|task| task.wake());
    }
}
