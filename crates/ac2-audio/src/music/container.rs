//! DirectMusic containers (`RIFF 'DMCN'`) and scripts (`RIFF 'DMSC'`): a script's AudioVBScript
//! source with the container of objects its names refer to.

use super::band::{read_reference, Reference};
use super::riff::{Chunk, Guid};
use super::MusicObject;

/// An object a container holds, under its alias (`coba`): embedded, or a reference.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainedObject {
    pub alias: String,
    pub class: Guid,
    pub object: Option<MusicObject>,
    pub reference: Option<Reference>,
}

/// A container.
#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    pub guid: Option<Guid>,
    pub objects: Vec<ContainedObject>,
}

impl Container {
    /// The object named `alias` (case-insensitive, as AudioVBScript names are).
    pub fn find(&self, alias: &str) -> Option<&ContainedObject> {
        self.objects
            .iter()
            .find(|o| o.alias.eq_ignore_ascii_case(alias))
    }
}

/// A script.
#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    pub guid: Option<Guid>,
    pub name: Option<String>,
    /// `scla`: "AudioVBScript".
    pub language: String,
    /// `scsr`.
    pub source: String,
    pub container: Option<Container>,
}

/// Reads a container from its `RIFF 'DMCN'` chunk.
pub fn read_container(top: &Chunk<'_>) -> Container {
    let objects = top.list(b"cosl").map_or_else(Vec::new, |list| {
        list.children()
            .filter(|c| c.is_list(b"cobl"))
            .map(|c| {
                let class = c.child(b"cobh").map_or(Guid::NULL, |h| h.reader().guid());
                let embedded = c
                    .children()
                    .find(|o| &o.id == b"RIFF")
                    .and_then(|o| super::read_object_chunk(&o).ok());
                ContainedObject {
                    alias: c.child(b"coba").map(|a| a.text()).unwrap_or_default(),
                    class,
                    object: embedded,
                    reference: c.list(b"DMRF").map(|r| read_reference(&r)),
                }
            })
            .collect()
    });
    Container {
        guid: top.child(b"guid").map(|c| c.reader().guid()),
        objects,
    }
}

/// Reads a script from its `RIFF 'DMSC'` chunk.
pub fn read_script(top: &Chunk<'_>) -> Script {
    Script {
        guid: top.child(b"guid").map(|c| c.reader().guid()),
        name: top
            .list(b"UNFO")
            .and_then(|u| u.child(b"UNAM"))
            .map(|c| c.text()),
        language: top.child(b"scla").map(|c| c.text()).unwrap_or_default(),
        source: top.child(b"scsr").map(|c| c.text()).unwrap_or_default(),
        container: top
            .children()
            .find(|c| c.is_list(b"DMCN"))
            .map(|c| read_container(&c)),
    }
}
