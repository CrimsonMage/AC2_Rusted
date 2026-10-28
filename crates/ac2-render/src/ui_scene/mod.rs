//! The UI's 3D scenes, backend-neutral (retail `UIScene`, `UIScenePhysObjEntity` and
//! `SceneTool`): the data a viewport's scene reads ([`data`]: entities, cameras, lights and the
//! objects' shapes), the view its camera makes ([`camera`]), the figures a scene's owner places in
//! it ([`figure`]) and the viewports' state ([`views`]: which scene each shown viewport root
//! shows, its figures, bodies, camera and frame). The panels decide what stands where; the GPU
//! draws it (`ac2-gpu`).

pub mod camera;
pub mod data;
pub mod figure;
pub mod viewport;
pub mod views;

pub use viewport::SceneViewport;
pub use views::ScenePick;
pub use views::{SceneLoads, UiViews, ViewFrame, ViewSetup};
