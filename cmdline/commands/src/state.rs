use super::storage_descriptor::*;

use flo_animation;
use flo_animation::*;
use flo_anim_sqlite::*;
use flo_ui_files::*;
use flo_ui_files::sqlite::*;

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
        let input_animation     = Arc::new(SqliteAnimation::new_in_memory());
        let output_animation    = Arc::new(SqliteAnimation::new_in_memory());
        let input_animation     = AnimationState(StorageDescriptor::InMemory, input_animation.clone(), input_animation);
        let output_animation    = AnimationState(StorageDescriptor::InMemory, output_animation.clone(), output_animation);

        // Generate the initial command state
        CommandState(Arc::new(StateValue {
            file_manager:       file_manager,
            input_animation:    input_animation,
            output_animation:   output_animation,
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
    /// Updates the output animation for this state
    ///
    pub fn set_output_animation(&self, description: StorageDescriptor, animation: Arc<SqliteAnimation>) -> CommandState {
        CommandState(Arc::new(StateValue {
            file_manager:       self.0.file_manager.clone(),
            input_animation:    self.0.input_animation.clone(),
            edit_buffer:        self.0.edit_buffer.clone(),

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

            input_animation:    AnimationState(input, new_input.clone(), new_input.clone()),
        })))
    }

    ///
    /// Sets the edit buffer to a new value
    ///
    pub fn set_edit_buffer(&self, new_buffer: Vec<AnimationEdit>) -> CommandState {
        CommandState(Arc::new(StateValue {
            file_manager:       self.0.file_manager.clone(),
            input_animation:    self.0.input_animation.clone(),
            output_animation:   self.0.output_animation.clone(),

            edit_buffer:        Arc::new(new_buffer)
        }))
    }
}
