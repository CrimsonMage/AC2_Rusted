//! An ACE world-database weenie (one `9 WeenieDefaults/*.sql` patch file of ACE-World-16PY): the
//! properties that decide how a creature looks, read from its `INSERT` statements. ACE builds
//! the `ObjDesc` it sends from the same properties (`WorldObject.CalculateObjDesc`): the Setup,
//! the ClothingBase's object effects for that Setup, the PaletteTemplate's sub-palettes at the
//! weenie's shade, and the weenie's own anim parts, texture maps and palettes. Its HeartBeat
//! emotes are the idle actions ACE's `EmoteManager` plays every heartbeat.

use std::path::Path;

/// A weenie's appearance.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AceWeenie {
    pub class_id: u32,
    pub name: String,
    /// `PropertyDataId.Setup`.
    pub setup: u32,
    /// `PropertyDataId.MotionTable`.
    pub motion_table: u32,
    /// `PropertyDataId.ClothingBase`.
    pub clothing_base: u32,
    /// `PropertyDataId.PaletteBase`: the `ObjDesc` palette the sub-palettes go over.
    pub palette_base: u32,
    /// `PropertyDataId.Icon`: the item's icon (a `RenderSurface`).
    pub icon: u32,
    /// `PropertyInt.PaletteTemplate`.
    pub palette_template: Option<u32>,
    /// `PropertyFloat.Shade` (0 when unset).
    pub shade: f64,
    /// Whether the weenie sets `PropertyFloat.Shade`.
    pub has_shade: bool,
    /// `PropertyFloat.DefaultScale` (1 when unset).
    pub scale: f32,
    /// `weenie_properties_anim_part`: (part, GfxObj).
    pub anim_parts: Vec<(u32, u32)>,
    /// `weenie_properties_texture_map`: (part, old SurfaceTexture, new SurfaceTexture).
    pub texture_maps: Vec<(u32, u32, u32)>,
    /// `weenie_properties_palette`: (palette or palette set, offset, length), in units of 8.
    pub palettes: Vec<(u32, u32, u32)>,
    /// `PropertyFloat.HeartbeatInterval` in seconds (5 when unset).
    pub heartbeat_interval: f64,
    /// The HeartBeat emote sets (`EmoteCategory.HeartBeat`), in the file's order.
    pub heartbeat_emotes: Vec<AceEmoteSet>,
}

/// One emote set: the roll it answers, the stance it applies in, and its Motion actions.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AceEmoteSet {
    /// `probability`: the set answers a roll under it (`EmoteManager.GetEmoteSet` takes the
    /// first set, by ascending probability, above the roll).
    pub probability: f64,
    /// `style` and `substyle` (`MotionStance` and the substate); `None` for any.
    pub style: Option<u32>,
    pub substyle: Option<u32>,
    /// `EmoteType.Motion` actions in order: (delay in seconds, motion command).
    pub motions: Vec<(f64, u32)>,
}

/// `EmoteCategory.HeartBeat` and `EmoteType.Motion`.
const EMOTE_HEARTBEAT: u32 = 5;
const EMOTE_ACTION_MOTION: u32 = 5;

/// One SQL value.
#[derive(Debug, Clone, PartialEq)]
enum Value {
    Number(f64),
    Text(String),
    Null,
}

impl Value {
    fn u32(&self) -> Option<u32> {
        match self {
            Value::Number(n) => Some(*n as i64 as u32),
            _ => None,
        }
    }

    fn f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }
}

impl AceWeenie {
    /// Reads the weenie a patch file defines.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::parse(&text).ok_or_else(|| format!("{}: not an ACE weenie", path.display()))
    }

    /// The weenie of one patch file's SQL text; `None` without a `weenie` row and a Setup.
    pub fn parse(sql: &str) -> Option<Self> {
        let mut weenie = Self {
            scale: 1.0,
            heartbeat_interval: 5.0,
            ..Self::default()
        };
        let mut found = false;
        // The emote set the following `weenie_properties_emote_action` rows belong to
        // (`SET @parent_id = LAST_INSERT_ID()`), when it is a HeartBeat set.
        let mut current: Option<usize> = None;
        for (table, rows) in inserts(sql) {
            for row in rows {
                let at = |i: usize| row.get(i).cloned().unwrap_or(Value::Null);
                match table.as_str() {
                    "weenie" => {
                        weenie.class_id = at(0).u32()?;
                        found = true;
                    }
                    "weenie_properties_string" if at(1).u32() == Some(1) => {
                        if let Value::Text(name) = at(2) {
                            weenie.name = name;
                        }
                    }
                    "weenie_properties_d_i_d" => match at(1).u32() {
                        Some(1) => weenie.setup = at(2).u32().unwrap_or(0),
                        Some(2) => weenie.motion_table = at(2).u32().unwrap_or(0),
                        Some(6) => weenie.palette_base = at(2).u32().unwrap_or(0),
                        Some(8) => weenie.icon = at(2).u32().unwrap_or(0),
                        Some(7) => weenie.clothing_base = at(2).u32().unwrap_or(0),
                        _ => {}
                    },
                    "weenie_properties_int" if at(1).u32() == Some(3) => {
                        weenie.palette_template = at(2).u32();
                    }
                    "weenie_properties_float" => match at(1).u32() {
                        Some(1) => weenie.heartbeat_interval = at(2).f64().unwrap_or(5.0),
                        Some(12) => {
                            weenie.shade = at(2).f64().unwrap_or(0.0);
                            weenie.has_shade = true;
                        }
                        Some(39) => weenie.scale = at(2).f64().unwrap_or(1.0) as f32,
                        _ => {}
                    },
                    "weenie_properties_anim_part" => {
                        weenie.anim_parts.push((at(1).u32()?, at(2).u32()?));
                    }
                    "weenie_properties_texture_map" => {
                        weenie
                            .texture_maps
                            .push((at(1).u32()?, at(2).u32()?, at(3).u32()?));
                    }
                    "weenie_properties_emote" => {
                        current = (at(1).u32() == Some(EMOTE_HEARTBEAT)).then(|| {
                            weenie.heartbeat_emotes.push(AceEmoteSet {
                                probability: at(2).f64().unwrap_or(1.0),
                                style: at(4).u32(),
                                substyle: at(5).u32(),
                                motions: Vec::new(),
                            });
                            weenie.heartbeat_emotes.len() - 1
                        });
                    }
                    "weenie_properties_emote_action" => {
                        if let (Some(set), Some(EMOTE_ACTION_MOTION), Some(motion)) =
                            (current, at(2).u32(), at(5).u32())
                        {
                            let delay = at(3).f64().unwrap_or(0.0);
                            weenie.heartbeat_emotes[set].motions.push((delay, motion));
                        }
                    }
                    "weenie_properties_palette" => {
                        weenie
                            .palettes
                            .push((at(1).u32()?, at(2).u32()?, at(3).u32()?));
                    }
                    _ => {}
                }
            }
        }
        (found && weenie.setup != 0).then_some(weenie)
    }
}

/// Every `INSERT INTO `table` (...) VALUES (...), (...);` of `sql`, comments dropped.
fn inserts(sql: &str) -> Vec<(String, Vec<Vec<Value>>)> {
    let tokens = tokenize(sql);
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let is_insert = matches!(&tokens[i], Token::Word(w) if w.eq_ignore_ascii_case("INSERT"));
        if !is_insert {
            i += 1;
            continue;
        }
        // INSERT INTO `table` ( columns ) VALUES ( row ) , ( row ) ;
        let table = match tokens.get(i + 2) {
            Some(Token::Name(name)) => name.clone(),
            _ => {
                i += 1;
                continue;
            }
        };
        i += 3;
        while i < tokens.len()
            && !matches!(&tokens[i], Token::Word(w) if w.eq_ignore_ascii_case("VALUES"))
        {
            i += 1;
        }
        i += 1;
        let mut rows = Vec::new();
        while i < tokens.len() {
            match &tokens[i] {
                Token::Open => {
                    let mut row = Vec::new();
                    i += 1;
                    while i < tokens.len() && tokens[i] != Token::Close {
                        if let Token::Value(value) = &tokens[i] {
                            row.push(value.clone());
                        } else if let Token::Word(_) = &tokens[i] {
                            // `@parent_id` and other variables stand in for a value.
                            row.push(Value::Null);
                        }
                        i += 1;
                    }
                    rows.push(row);
                    i += 1;
                }
                Token::Comma => i += 1,
                _ => break,
            }
        }
        out.push((table, rows));
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    Name(String),
    Value(Value),
    Open,
    Close,
    Comma,
    Other,
}

fn tokenize(sql: &str) -> Vec<Token> {
    let chars: Vec<char> = sql.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '/' if chars.get(i + 1) == Some(&'*') => {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            '-' if chars.get(i + 1) == Some(&'-') => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '`' => {
                let start = i + 1;
                i = start;
                while i < chars.len() && chars[i] != '`' {
                    i += 1;
                }
                tokens.push(Token::Name(
                    chars[start..i.min(chars.len())].iter().collect(),
                ));
                i += 1;
            }
            '\'' => {
                let mut text = String::new();
                i += 1;
                while i < chars.len() {
                    match chars[i] {
                        '\\' if i + 1 < chars.len() => {
                            text.push(chars[i + 1]);
                            i += 2;
                        }
                        '\'' if chars.get(i + 1) == Some(&'\'') => {
                            text.push('\'');
                            i += 2;
                        }
                        '\'' => break,
                        other => {
                            text.push(other);
                            i += 1;
                        }
                    }
                }
                tokens.push(Token::Value(Value::Text(text)));
                i += 1;
            }
            '(' => {
                tokens.push(Token::Open);
                i += 1;
            }
            ')' => {
                tokens.push(Token::Close);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            c if c.is_ascii_digit()
                || (c == '-' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit())) =>
            {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '.') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let value = match text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
                    Some(hex) => u64::from_str_radix(hex, 16).map(|v| v as f64).ok(),
                    None => text.parse::<f64>().ok(),
                };
                tokens.push(value.map_or(Token::Other, |v| Token::Value(Value::Number(v))));
            }
            c if c.is_ascii_alphabetic() || c == '@' || c == '_' => {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '@')
                {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                tokens.push(match word.to_ascii_uppercase().as_str() {
                    "TRUE" => Token::Value(Value::Number(1.0)),
                    "FALSE" => Token::Value(Value::Number(0.0)),
                    "NULL" => Token::Value(Value::Null),
                    _ => Token::Word(word),
                });
            }
            c if c.is_whitespace() => i += 1,
            _ => {
                tokens.push(Token::Other);
                i += 1;
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    // A made-up weenie in the patch files' shape (no game data).
    const SQL: &str = "DELETE FROM `weenie` WHERE `class_Id` = 90001;

INSERT INTO `weenie` (`class_Id`, `class_Name`, `type`, `last_Modified`)
VALUES (90001, 'testbeast', 10, '2022-08-22 03:09:27') /* Creature */;

INSERT INTO `weenie_properties_int` (`object_Id`, `type`, `value`)
VALUES (90001,   1,         16) /* ItemType - Creature */
     , (90001,   3,         48) /* PaletteTemplate - SnowyWhite */;

INSERT INTO `weenie_properties_float` (`object_Id`, `type`, `value`)
VALUES (90001,  12,     0.5) /* Shade */
     , (90001,  39,    0.95) /* DefaultScale */;

INSERT INTO `weenie_properties_string` (`object_Id`, `type`, `value`)
VALUES (90001,   1, 'Test''s Beast') /* Name */;

INSERT INTO `weenie_properties_d_i_d` (`object_Id`, `type`, `value`)
VALUES (90001,   1, 0x02000001) /* Setup */
     , (90001,   2, 0x09000002) /* MotionTable */
     , (90001,   6, 0x04000005) /* PaletteBase */
     , (90001,   7, 0x10000003) /* ClothingBase */
     , (90001,  32,         80) /* WieldedTreasureType -
                                   |  16.00% chance of (319) */;

INSERT INTO `weenie_properties_palette` (`object_Id`, `sub_Palette_Id`, `offset`, `length`)
VALUES (90001, 0x0F000004, 0, 12);

INSERT INTO `weenie_properties_emote` (`object_Id`, `category`, `probability`, `weenie_Class_Id`, `style`, `substyle`, `quest`)
VALUES (90001,  3 /* Death */,   0.5, NULL, NULL, NULL, NULL);

SET @parent_id = LAST_INSERT_ID();

INSERT INTO `weenie_properties_emote_action` (`emote_Id`, `order`, `type`, `delay`, `extent`, `motion`)
VALUES (@parent_id,  0,   5 /* Motion */, 0, 1, 0x10000099);

INSERT INTO `weenie_properties_emote` (`object_Id`, `category`, `probability`, `weenie_Class_Id`, `style`, `substyle`, `quest`)
VALUES (90001,  5 /* HeartBeat */,   0.1, NULL, 0x8000003D /* NonCombat */, 0x41000003 /* Ready */, NULL);

SET @parent_id = LAST_INSERT_ID();

INSERT INTO `weenie_properties_emote_action` (`emote_Id`, `order`, `type`, `delay`, `extent`, `motion`)
VALUES (@parent_id,  0,   5 /* Motion */, 0, 1, 0x10000051 /* Twitch1 */)
     , (@parent_id,  1,  10 /* Tell */, 0, 1, NULL)
     , (@parent_id,  2,   5 /* Motion */, 0.5, 1, 0x41000003 /* Ready */);
";

    #[test]
    fn a_patch_file_gives_the_weenies_appearance() {
        let weenie = AceWeenie::parse(SQL).expect("weenie");
        assert_eq!(weenie.class_id, 90001);
        assert_eq!(weenie.name, "Test's Beast");
        assert_eq!(weenie.setup, 0x0200_0001);
        assert_eq!(weenie.motion_table, 0x0900_0002);
        assert_eq!(weenie.clothing_base, 0x1000_0003);
        assert_eq!(weenie.palette_base, 0x0400_0005);
        assert_eq!(weenie.palette_template, Some(48));
        assert_eq!(weenie.shade, 0.5);
        assert_eq!(weenie.scale, 0.95);
        assert_eq!(weenie.palettes, vec![(0x0F00_0004, 0, 12)]);
        assert_eq!(weenie.heartbeat_interval, 5.0);
        assert_eq!(
            weenie.heartbeat_emotes,
            vec![AceEmoteSet {
                probability: 0.1,
                style: Some(0x8000_003D),
                substyle: Some(0x4100_0003),
                motions: vec![(0.0, 0x1000_0051), (0.5, 0x4100_0003)],
            }]
        );
        assert!(AceWeenie::parse("SELECT 1;").is_none());
    }
}
