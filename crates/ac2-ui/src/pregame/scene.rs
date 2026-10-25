//! The pregame screens' 3D viewports. A `UIElement_Viewport` root names the `UIScene` it shows
//! (`uiscene`, `SetAttribute_DataID`) and the scene's camera entity (`camera`,
//! `SetAttribute_ASCII`, mapped through `EnumMapper` 0x4000002E to the entity id in
//! `SetCameraID`); its surface object (`UIViewportObject::Render`) sets the device viewport to
//! the element's rectangle, renders the scene (`UIScene::Render`) and restores the viewport, in
//! the root's place among the UI's surfaces.

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::token::tokenize;

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

/// Every composed root that shows a scene, in the order [`UiTree::compose`] draws the roots.
pub(crate) fn viewports(tree: &UiTree) -> Vec<SceneViewport> {
    let (uiscene, camera) = (tokenize("uiscene"), tokenize("camera"));
    tree.composed_roots()
        .filter_map(|root| {
            let el = tree.el(root);
            let Some(AttributeValue::DataId(scene)) = el.attribute(uiscene) else {
                return None;
            };
            if *scene == 0 || !el.shown {
                return None;
            }
            let camera = match el.attribute(camera) {
                Some(AttributeValue::Ascii(name)) => Some(
                    String::from_utf8_lossy(name)
                        .trim_end_matches('\0')
                        .to_owned(),
                ),
                _ => None,
            };
            let (x, y) = el.object.unwrap_or((0, 0));
            Some(SceneViewport {
                root,
                element: el.id(),
                x,
                y,
                width: el.rect.width() as u32,
                height: el.rect.height() as u32,
                scene: *scene,
                camera,
            })
        })
        .collect()
}
