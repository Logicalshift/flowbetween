#[allow(dead_code)]

///
/// Errors that can happen when trying to initialise the renderer
///
#[derive(Clone, Debug, PartialEq)]
pub enum RenderInitError {
    /// Indicates that the graphics device could not be opened
    CannotOpenGraphicsDevice,

    /// Indicates that the graphics device could not be attached to
    CannotCreateGraphicsDevice,

    /// The graphics driver failed to initialise
    CannotStartGraphicsDriver,

    /// The graphics display is not available
    DisplayNotAvailable,

    /// A required extension was missing
    MissingRequiredExtension,

    /// Unable to configure the display
    CouldNotConfigureDisplay,

    /// The context failed to create
    CouldNotCreateContext,

    /// Could not set the active context
    ContextDidNotStart
}
