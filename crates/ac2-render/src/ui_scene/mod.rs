//! The UI scenes' scene assembly, backend-neutral (retail `UIScene`, `UIScenePhysObjEntity` and
//! `SceneTool`): the data a viewport's scene reads ([`data`]: entities, cameras, lights and the
//! objects' shapes), the view its camera makes ([`camera`]) and the figures a scene's owner
//! places in it ([`figure`]). The panels decide what stands where; the GPU draws it (`ac2-gpu`).

pub mod camera;
pub mod data;
pub mod figure;
