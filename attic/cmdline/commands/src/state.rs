use super::storage_descriptor::*;

use flo_animation;
use flo_animation::*;
use flo_animation::storage::*;
use flo_ui_files::*;
use flo_ui_files::sqlite::*;

use futures::prelude::*;

use std::sync::*;

/// The name of the app (after our domain: flowbetween.app)
pub const APP_NAME: &str = "app.flowbetween";

/// Where we store the default user data
pub const DEFAULT_USER_FOLDER: &str = "default";

///
/// Represents the state of a command stream
///
#[derive(Clone)]
pub struct CommandState(Arc<StateValue>);

///
/// How an animation is stored within the command state
///
#[derive(Clone)]
struct AnimationState(StorageDescriptor, Arc<dyn Animation>, Arc<dyn EditableAnimation>);

///
/// The internal value of a command state
///
struct StateValue { 
    /// The file manager that this state will use
    file_manager: Arc<dyn FileManager>,

    /// The animation that this will read from
    input_animation: AnimationState,

    /// The animation that this will write to
    output_animation: AnimationState,

    /// The frame that's currently selected
    frame: Option<Arc<dyn Frame>>,

    /// A list of edits (read from an animation, or waiting to be written to one)
    edit_buffer: Arc<Vec<AnimationEdit>>
}

impl CommandState {
    ///
    /// Creates a new command state with the default settings
    ///
    pub fn new() -> CommandState {
        // Use the default file manager
        let file_manager        = SqliteFileManager::new(APP_NAME, DEFAULT_USER_FOLDER);
        let file_manager        = Arc::new(file_manager);

        // Create the input and output animation in memory
        let input_animation     = InMemoryStorage::new();
        let input_animation     = Arc::new(create_animation_editor(move |commands| input_animation.get_responses(commands).boxed()));
        let output_animation    = InMemoryStorage::new();
        let output_animation    = Arc::new(create_animation_editor(move |commands| output_animation.get_responses(commands).boxed()));
        let input_animation     = AnimationState(StorageDescriptor::InMemory, input_animation.clone(), input_animation);
        let output_animation    = AnimationState(StorageDescriptor::InMemory, output_animation.clone(), output_animation);

        // Generate the initial command state
        CommandState(Arc::new(StateValue {
            file_manager:       file_manager,
            input_animation:    input_animation,
            output_animation:   output_animation,
            frame:              None,
            edit_buffer:        Arc::new(vec![])
        }))
    }

    ///
    /// Returns the file manager currently set for this command state
    ///
    pub fn file_manager(&self) -> Arc<dyn FileManager> {
        Arc::clone(&self.0.file_manager)
    }

    ///
    /// Retrieves the current input animation for this state
    ///
    pub fn input_animation(&self) -> Arc<dyn Animation> {
        Arc::clone(&self.0.input_animation.1)
    }

    ///
    /// Retrieves the current output animation for this state
    ///
    pub fn output_animation(&self) -> Arc<dyn EditableAnimation> {
        Arc::clone(&self.0.output_animation.2)
    }

    ///
    /// Retrieves the edit buffer set in this state
    ///
    pub fn edit_buffer(&self) -> &Vec<AnimationEdit> {
        &*self.0.edit_buffer
    }

    ///
    /// Retrieves the currently selected frame, if there is one
    ///
    pub fn frame(&self) -> Option<Arc<dyn Frame>> {
        self.0.frame.clone()
    }

    ///
    /// Removes all of the edits from the current state
    ///
    pub fn clear_edit_buffer(&self) -> CommandState {
        self.set_edit_buffer(vec![])
    }

    ///
    /// Updates the file manager for this state
    ///
    pub fn set_file_manager(&self, new_file_manager: Arc<dyn FileManager>) -> CommandState {
        CommandState(Arc::new(StateValue {
            input_animation:    self.0.input_animation.clone(),
            output_animation:   self.0.output_animation.clone(),
            edit_buffer:        self.0.edit_buffer.clone(),
            frame:              self.0.frame.clone(),

            file_manager:       new_file_manager
        }))
    }

    ///
    /// Updates the output animation for this state
    ///
    pub fn set_output_animation<Anim: 'static+EditableAnimation>(&self, description: StorageDescriptor, animation: Arc<Anim>) -> CommandState {
        CommandState(Arc::new(StateValue {
            file_manager:       self.0.file_manager.clone(),
            input_animation:    self.0.input_animation.clone(),
            edit_buffer:        self.0.edit_buffer.clone(),
            frame:              self.0.frame.clone(),

            output_animation:   AnimationState(description, animation.clone(), animation.clone()),
        }))
    }

    ///
    /// Returns this state modified to have a new input file loaded from the specified storage descriptor (None if the file cannot be loaded)
    ///
    pub fn load_input_file(&self, input: StorageDescriptor) -> Option<CommandState> {
        // Ask the descriptor to open the animation it's referencing
        let new_input = input.open_animation(&self.file_manager())?;

        // Return a state with the new animation as the input
        Some(CommandState(Arc::new(StateValue {
            file_manager:       self.0.file_manager.clone(),
            output_animation:   self.0.output_animation.clone(),
            edit_buffer:        self.0.edit_buffer.clone(),
            frame:              self.0.frame.clone(),

            input_animation:    AnimationState(input, new_input.clone(), new_input.clone()),
        })))
    }

    ///
    /// Puts the current 'write' animation into the 'read' side of the state
    ///
    pub fn read_from_write_side(&self) -> CommandState {
        CommandState(Arc::new(StateValue {
            file_manager:       self.0.file_manager.clone(),
            output_animation:   self.0.output_animation.clone(),
            edit_buffer:        self.0.edit_buffer.clone(),
            frame:              self.0.frame.clone(),

            input_animation:    self.0.output_animation.clone()
        }))
    }

    ///
    /// Sets the edit buffer to a new value
    ///
    pub fn set_edit_buffer(&self, new_buffer: Vec<AnimationEdit>) -> CommandState {
        CommandState(Arc::new(StateValue {
            file_manager:       self.0.file_manager.clone(),
            input_animation:    self.0.input_animation.clone(),
            output_animation:   self.0.output_animation.clone(),
            frame:              self.0.frame.clone(),

            edit_buffer:        Arc::new(new_buffer)
        }))
    }

    ///
    /// Sets the selected frame to a new value
    ///
    pub fn set_selected_frame(&self, new_frame: Option<Arc<dyn Frame>>) -> CommandState {
        CommandState(Arc::new(StateValue {
            file_manager:       self.0.file_manager.clone(),
            input_animation:    self.0.input_animation.clone(),
            output_animation:   self.0.output_animation.clone(),
            edit_buffer:        self.0.edit_buffer.clone(),

            frame:              new_frame
        }))
    }
}
