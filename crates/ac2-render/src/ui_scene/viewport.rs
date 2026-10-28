//! The UI's 3D viewports as the scene assembly reads them: one shown viewport root, the
//! `UIScene` it names and the camera entity. `ac2_ui` reads them off the element tree.

/// One shown viewport root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneViewport {
    /// The root element (its place in the composed surfaces, [`crate::UiSurface::root`]).
    pub root: usize,
    /// The element id (`Viewport`, `AvatarSelectionViewport`, `<race>Viewport`).
    pub element: u32,
    /// Screen rectangle in pixels.
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    /// `uiscene`: the `UIScene` DID.
    pub scene: u32,
    /// `camera`: the camera entity's workbench name (`Camera_1`).
    pub camera: Option<String>,
}
