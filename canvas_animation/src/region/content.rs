use crate::path::*;

use flo_canvas::*;

use std::sync::*;
use std::time::Duration;
use std::slice;

// TODO: add a way to add a 2D transformation for the content of a region

#[derive(Clone, Copy, PartialEq, Debug)]
enum FillStyle {
    Solid(Color),
    Texture(TextureId, (f32, f32), (f32, f32)),
    Gradient(GradientId, (f32, f32), (f32, f32))
}

///
/// Describes what's in a particular animation region
///
pub struct AnimationRegionContent {
    /// The region content that precedes this region (the content of this region is added to this one)
    prefix: Option<Arc<AnimationRegionContent>>,

    /// The paths tht appear in this region
    paths: Vec<AnimationPath>,

    /// The region content that follows this region (this will be added after the content of this region)
    suffix: Option<Arc<AnimationRegionContent>>
}

impl Default for AnimationRegionContent {
    ///
    /// AnimationRegionContents have a default empty value
    ///
    fn default() -> Self {
        AnimationRegionContent {
            prefix: None,
            paths:  vec![],
            suffix: None
        }
    }
}

impl AnimationRegionContent {
    ///
    /// Creates a new animation region content item from a list of paths
    ///
    pub fn from_paths<PathIter: IntoIterator<Item=AnimationPath>>(paths: PathIter) -> AnimationRegionContent {
        AnimationRegionContent {
            prefix: None,
            paths:  paths.into_iter().collect(),
            suffix: None
        }
    }

    ///
    /// Adds a set of content before this region
    ///
    pub fn with_prefix(self, prefix: Arc<AnimationRegionContent>) -> AnimationRegionContent {
        // If there's an existing prefix, make a new region content with both prefixes
        let prefix = if let Some(existing_prefix) = self.prefix {
            Arc::new(AnimationRegionContent::default()
                .with_prefix(prefix)
                .with_suffix(existing_prefix))
        } else {
            prefix
        };

        AnimationRegionContent {
            prefix: Some(prefix),
            paths:  self.paths,
            suffix: self.suffix
        }
    }

    ///
    /// Adds a set of content after this region
    ///
    pub fn with_suffix(self, suffix: Arc<AnimationRegionContent>) -> AnimationRegionContent {
        // If there's an existing suffix, make a new region content with both suffixes
        let suffix = if let Some(existing_suffix) = self.suffix {
            Arc::new(AnimationRegionContent::default()
                .with_prefix(existing_suffix)
                .with_suffix(suffix))
        } else {
            suffix
        };

        AnimationRegionContent {
            prefix: self.prefix,
            paths:  self.paths,
            suffix: Some(suffix)
        }
    }

    ///
    /// Creates an iterator through the paths that make up this region
    ///
    pub fn paths<'a>(&'a self) -> impl 'a+Iterator<Item=&AnimationPath> {
        AnimationContentIterator::from_region_content(self)
    }

    ///
    /// Converts the content of a region into some drawing instructions
    ///
    pub fn to_drawing(&self, time: Duration) -> Vec<Draw> {
        // Initial state is that all attributes are unknown
        let mut stroke_colour       = None;
        let mut stroke_join         = None;
        let mut stroke_cap          = None;
        let mut fill_windingrule    = None;
        let mut fill_style          = None;

        // Drawing initially pushes the canvas state
        let mut drawing = vec![Draw::PushState];

        // Draw the paths in order
        for path in self.paths.iter() {
            // Paths that are not visible at this time are skipped
            if path.appearance_time > time { continue; }

            // Load the path
            drawing.push(Draw::Path(PathOp::NewPath));
            drawing.extend(path.to_path_ops().map(|pathop| Draw::Path(pathop)));

            // Apply any changed attributes for these paths, and render them
            use self::AnimationPathAttribute::*;

            match path.attributes {
                Stroke(width, colour, join, cap) => {
                    if stroke_colour != Some(colour) {
                        stroke_colour = Some(colour);
                        drawing.push(Draw::StrokeColor(colour));
                    }

                    if stroke_join != Some(join) {
                        stroke_join = Some(join);
                        drawing.push(Draw::LineJoin(join));
                    }

                    if stroke_cap != Some(cap) {
                        stroke_cap = Some(cap);
                        drawing.push(Draw::LineCap(cap));
                    }

                    drawing.push(Draw::LineWidth(width));
                    drawing.push(Draw::Stroke);
                }

                StrokePixels(width_pixels, colour, join, cap) => {
                    if stroke_colour != Some(colour) {
                        stroke_colour = Some(colour);
                        drawing.push(Draw::StrokeColor(colour));
                    }

                    if stroke_join != Some(join) {
                        stroke_join = Some(join);
                        drawing.push(Draw::LineJoin(join));
                    }

                    if stroke_cap != Some(cap) {
                        stroke_cap = Some(cap);
                        drawing.push(Draw::LineCap(cap));
                    }

                    drawing.push(Draw::LineWidthPixels(width_pixels));
                    drawing.push(Draw::Stroke);
                }

                Fill(colour, windingrule) => { 
                    let new_style = FillStyle::Solid(colour);

                    if fill_windingrule != Some(windingrule) {
                        fill_windingrule = Some(windingrule);
                        drawing.push(Draw::WindingRule(windingrule));
                    }

                    if fill_style != Some(new_style) {
                        fill_style = Some(new_style);
                        drawing.push(Draw::FillColor(colour));
                    }

                    drawing.push(Draw::Fill);
                }

                FillTexture(texture_id, (x1, y1), (x2, y2), fill_transform, windingrule) => {
                    let new_style = FillStyle::Texture(texture_id, (x1, y1), (x2, y2));

                    if fill_windingrule != Some(windingrule) {
                        fill_windingrule = Some(windingrule);
                        drawing.push(Draw::WindingRule(windingrule));
                    }

                    if fill_style != Some(new_style) {
                        fill_style = Some(new_style);
                        drawing.push(Draw::FillTexture(texture_id, (x1, y1), (x2, y2)));
                    }

                    if let Some(fill_transform) = fill_transform {
                        drawing.push(Draw::FillTransform(fill_transform));
                    }
                    drawing.push(Draw::Fill);
                }

                FillGradient(gradient_id, (x1, y1), (x2, y2), fill_transform, windingrule) => {
                    let new_style = FillStyle::Gradient(gradient_id, (x1, y1), (x2, y2));

                    if fill_windingrule != Some(windingrule) {
                        fill_windingrule = Some(windingrule);
                        drawing.push(Draw::WindingRule(windingrule));
                    }

                    if fill_style != Some(new_style) {
                        fill_style = Some(new_style);
                        drawing.push(Draw::FillGradient(gradient_id, (x1, y1), (x2, y2)));
                    }

                    if let Some(fill_transform) = fill_transform {
                        drawing.push(Draw::FillTransform(fill_transform));
                    }
                    drawing.push(Draw::Fill);
                }
            }
        }

        // Finished drawing the paths: pop the layer state we pushed earlier
        drawing.push(Draw::PopState);
        drawing
    }
}

///
/// Iterator for the paths in an `AnimationRegionContent`
///
struct AnimationContentIterator<'a> {
    /// Stack of region contents that should be animated
    pending_content_stack: Vec<&'a AnimationRegionContent>,

    /// The iterator for the paths from the current animation region content
    current_iterator: slice::Iter<'a, AnimationPath>
}

impl<'a> AnimationContentIterator<'a> {
    ///
    /// Pushes the enumeration order of a region content to a stack for iteration
    ///
    fn push_region_to_stack(stack: &mut Vec<&'a AnimationRegionContent>, region: &'a AnimationRegionContent) {
        // Suffix is process last (so pushed to the stack first)
        if let Some(suffix) = region.suffix.as_ref() {
            Self::push_region_to_stack(stack, &*suffix);
        }

        // Then this region is processed
        stack.push(region);

        // Then the prefix is processed first
        if let Some(prefix) = region.prefix.as_ref() {
            Self::push_region_to_stack(stack, &*prefix);
        }
    }

    ///
    /// Creates a path iterator from an animation region content item
    ///
    pub fn from_region_content(content: &'a AnimationRegionContent) -> AnimationContentIterator<'a> {
        // Recursively process the content of the regions
        let mut pending_content_stack = vec![];
        Self::push_region_to_stack(&mut pending_content_stack, content);

        // Create the initial iterator
        let initial_iterator = pending_content_stack.pop().unwrap().paths.iter();

        AnimationContentIterator {
            pending_content_stack:  pending_content_stack,
            current_iterator:       initial_iterator
        }
    }
}

impl<'a> Iterator for AnimationContentIterator<'a> {
    type Item = &'a AnimationPath;

    fn next(&mut self) -> Option<&'a AnimationPath> {
        loop {
            // Try to get the next path from the current iterator
            if let Some(next) = self.current_iterator.next() {
                return Some(next);
            }

            // Try to get the next region from the stack
            if let Some(next_region) = self.pending_content_stack.pop() {
                // Iterate over the next region (and continue the loop)
                self.current_iterator = next_region.paths.iter();
            } else {
                // No more regions left
                return None;
            }
        }
    }
}
