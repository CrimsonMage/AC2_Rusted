//! The animated parts of a UI state: `MediaDesc` (what a state shows or does while it is current)
//! and `TransitionDesc` (how an element moves, resizes, fades or rotates into a state).
//!
//! `StateDesc::UnPack` reads each media item's type, makes it with `MediaDesc::CreateMediaType`
//! and unpacks it through `StreamPackObj::UnPack`, whose `StreamPack` starts with
//! `MediaDesc::StreamPack` (the type again) before the type's own fields.

use ac2_dat::{ArchiveCursor, ArchiveResult};

/// One `MediaDesc`, by `MediaDesc::CreateMediaType`'s type number.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaDesc {
    /// 1, `MD_Data_Anim`: image DIDs shown `duration` seconds in turn.
    Anim {
        duration: f32,
        draw_mode: u32,
        offset: (i32, i32),
        frames: Vec<u32>,
    },
    /// 2, `MD_Data_Pause`: wait a random time in `min..max` seconds.
    Pause { min: f32, max: f32 },
    /// 3, `MD_Data_Image`.
    Image {
        file: u32,
        draw_mode: u32,
        offset: (i32, i32),
    },
    /// 4, `MD_Data_Alpha`: an alpha mask image.
    Alpha { file: u32, offset: (i32, i32) },
    /// 5, `MD_Data_Sound`.
    Sound { sound: u32 },
    /// 6, `MD_Data_Jump`: jump to media item `index` with `probability`.
    Jump { index: u32, probability: f32 },
    /// 7, `MD_Data_Message`: send `message` with `probability`.
    Message { message: u32, probability: f32 },
    /// 8, `MD_Data_State`: switch the element to `state`.
    State { state: u32 },
    /// 9, `MD_Data_AVI`: a movie file, by name.
    Avi { offset: (i32, i32), file: Vec<u8> },
    /// 10, `MD_Data_Cursor`: the pointer image and its hotspot.
    Cursor { file: u32, hotspot: (i32, i32) },
}

/// `TransitionDesc::type`'s data (`TD_Data`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionData {
    /// 1, `TD_Data_Move`: offsets (end x, end y, start x, start y).
    Move {
        end: (i32, i32),
        start: (i32, i32),
        duration: f64,
        flags: u32,
    },
    /// 2, `TD_Data_Resize`: (end height, end width, start height, start width).
    Resize {
        end: (i32, i32),
        start: (i32, i32),
        duration: f64,
        flags: u32,
    },
    /// 3, `TD_Data_Fade`.
    Fade {
        end_alpha: f32,
        start_alpha: f32,
        duration: f64,
        flags: u32,
    },
    /// 4, `TD_Data_Rotate`.
    Rotate {
        end: [i32; 3],
        duration: f64,
        flags: u32,
    },
    /// Any other type packs no data.
    None(u32),
}

/// One `TransitionDesc`: `data` runs `when` the element enters `state`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransitionDesc {
    pub when: u32,
    pub state: u32,
    pub data: TransitionData,
}

/// `PStringBase<char>::UnPack`'s bytes: a u16 length (`0xFFFF` escapes to a u32), the
/// characters, then 4-byte alignment.
pub(crate) fn pstring_bytes(cursor: &mut ArchiveCursor<'_>) -> ArchiveResult<Vec<u8>> {
    let len = cursor.read_packed_string_len()? as usize;
    let bytes = cursor.read_bytes(len)?.to_vec();
    cursor.align_to(4)?;
    Ok(bytes)
}

fn offset(cursor: &mut ArchiveCursor<'_>) -> ArchiveResult<(i32, i32)> {
    Ok((cursor.read_i32()?, cursor.read_i32()?))
}

/// `MediaDesc::CreateMediaType(kind)` then its `StreamPack`; `None` for a type retail cannot make.
pub(crate) fn read_media(
    cursor: &mut ArchiveCursor<'_>,
    kind: u32,
    max_items: usize,
) -> ArchiveResult<Option<MediaDesc>> {
    if !(1..=10).contains(&kind) {
        return Ok(None);
    }
    // `MediaDesc::StreamPack`: `m_type`, the same value `StateDesc::UnPack` just read.
    if cursor.read_u32()? != kind {
        return Ok(None);
    }
    Ok(Some(match kind {
        1 => {
            let duration = cursor.read_f32()?;
            let draw_mode = cursor.read_u32()?;
            let offset = offset(cursor)?;
            // `PSmartArray<unsigned long>::StreamPack`: a count, then the items.
            let n = cursor.read_u32()? as usize;
            if n > max_items || n * 4 > cursor.remaining() {
                return Ok(None);
            }
            let frames = (0..n)
                .map(|_| cursor.read_u32())
                .collect::<ArchiveResult<_>>()?;
            MediaDesc::Anim {
                duration,
                draw_mode,
                offset,
                frames,
            }
        }
        2 => MediaDesc::Pause {
            min: cursor.read_f32()?,
            max: cursor.read_f32()?,
        },
        3 => MediaDesc::Image {
            file: cursor.read_u32()?,
            draw_mode: cursor.read_u32()?,
            offset: offset(cursor)?,
        },
        4 => MediaDesc::Alpha {
            file: cursor.read_u32()?,
            offset: offset(cursor)?,
        },
        5 => MediaDesc::Sound {
            sound: cursor.read_u32()?,
        },
        6 => MediaDesc::Jump {
            index: cursor.read_u32()?,
            probability: cursor.read_f32()?,
        },
        7 => MediaDesc::Message {
            message: cursor.read_u32()?,
            probability: cursor.read_f32()?,
        },
        8 => MediaDesc::State {
            state: cursor.read_u32()?,
        },
        9 => MediaDesc::Avi {
            offset: offset(cursor)?,
            file: pstring_bytes(cursor)?,
        },
        _ => MediaDesc::Cursor {
            file: cursor.read_u32()?,
            hotspot: offset(cursor)?,
        },
    }))
}

/// `TransitionDesc::StreamPack`: type, when, state, then the type's data, the duration a
/// 64-bit value.
pub(crate) fn read_transition(cursor: &mut ArchiveCursor<'_>) -> ArchiveResult<TransitionDesc> {
    let kind = cursor.read_u32()?;
    let when = cursor.read_u32()?;
    let state = cursor.read_u32()?;
    let data = match kind {
        1 | 2 => {
            let end = offset(cursor)?;
            let start = offset(cursor)?;
            let duration = cursor.read_f64()?;
            let flags = cursor.read_u32()?;
            if kind == 1 {
                TransitionData::Move {
                    end,
                    start,
                    duration,
                    flags,
                }
            } else {
                TransitionData::Resize {
                    end,
                    start,
                    duration,
                    flags,
                }
            }
        }
        3 => TransitionData::Fade {
            end_alpha: cursor.read_f32()?,
            start_alpha: cursor.read_f32()?,
            duration: cursor.read_f64()?,
            flags: cursor.read_u32()?,
        },
        4 => TransitionData::Rotate {
            end: [cursor.read_i32()?, cursor.read_i32()?, cursor.read_i32()?],
            duration: cursor.read_f64()?,
            flags: cursor.read_u32()?,
        },
        other => TransitionData::None(other),
    };
    Ok(TransitionDesc { when, state, data })
}
