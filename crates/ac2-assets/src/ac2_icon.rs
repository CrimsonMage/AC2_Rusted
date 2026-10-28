//! The icon description (`IconDesc`, a list of `IconLayerDesc`s) and the rules that merge icon
//! layers: `IconDesc::ApplyAdditiveIconLayer`, `IconLayerDesc::ApplyAdditive` and
//! `IconDesc::ApplyAdditiveIconDesc`. A `VisualDesc` carries one (`IconDesc::StreamPack`: counted
//! layers of layer id, image DID, shift colour), the parent chain merges them, and the icon
//! atoms of an appearance table (`IconModAtom`) are applied on top (`VisualDesc::GetTotalIconDesc`).

/// One `IconLayerDesc`: a layer id, the image drawn on it and a shift colour (`RGBAColor`, packed
/// as stored; `RGBAColor_Null` is all zero).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ac2IconLayer {
    pub layer_id: u32,
    pub image: u32,
    pub shift_color: u32,
}

impl Ac2IconLayer {
    /// `IconModAtom::Replaces`: the same layer, both with or both without an image, both with or
    /// both without a shift colour.
    pub(crate) fn replaces(&self, other: &Self) -> bool {
        self.layer_id == other.layer_id
            && (self.image != 0) == (other.image != 0)
            && (self.shift_color == 0) == (other.shift_color == 0)
    }

    /// `IconLayerDesc::ApplyAdditive`: a non-null image replaces ours, a non-null shift colour
    /// replaces ours.
    fn apply_additive(&mut self, layer: &Self) {
        if layer.image != 0 {
            self.image = layer.image;
        }
        if layer.shift_color != 0 {
            self.shift_color = layer.shift_color;
        }
    }
}

/// `IconDesc`: the layers ordered by layer id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ac2IconDesc {
    pub layers: Vec<Ac2IconLayer>,
}

impl Ac2IconDesc {
    /// `IconDesc::ApplyAdditiveIconLayer`: a held layer with the same id takes the new one's
    /// image and colour (`IconLayerDesc::ApplyAdditive`); otherwise the layer joins before the
    /// first held layer with a greater id, or at the end. Always succeeds here: retail's refusal
    /// (`List::find` of an identical layer) cannot happen once the id search missed.
    pub fn apply_additive_layer(&mut self, layer: &Ac2IconLayer) -> bool {
        if let Some(held) = self
            .layers
            .iter_mut()
            .find(|held| held.layer_id == layer.layer_id)
        {
            held.apply_additive(layer);
            return true;
        }
        let at = self
            .layers
            .iter()
            .position(|held| held.layer_id > layer.layer_id)
            .unwrap_or(self.layers.len());
        self.layers.insert(at, *layer);
        true
    }

    /// `IconDesc::ApplyAdditiveIconDesc`: each of `other`'s layers in order.
    pub fn apply_additive_desc(&mut self, other: &Self) -> bool {
        let mut all = true;
        for layer in &other.layers {
            all &= self.apply_additive_layer(layer);
        }
        all
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(layer_id: u32, image: u32, shift_color: u32) -> Ac2IconLayer {
        Ac2IconLayer {
            layer_id,
            image,
            shift_color,
        }
    }

    #[test]
    fn a_layer_with_a_held_id_merges_field_by_field() {
        let mut icon = Ac2IconDesc {
            layers: vec![layer(1, 0x0600_0001, 0xFF), layer(3, 0x0600_0003, 0)],
        };
        // A null image and a null colour leave the held ones.
        assert!(icon.apply_additive_layer(&layer(1, 0, 0)));
        assert_eq!(icon.layers[0], layer(1, 0x0600_0001, 0xFF));
        // A new image keeps the colour; a new colour keeps the image.
        icon.apply_additive_layer(&layer(1, 0x0600_0009, 0));
        assert_eq!(icon.layers[0], layer(1, 0x0600_0009, 0xFF));
        icon.apply_additive_layer(&layer(1, 0, 0xAA));
        assert_eq!(icon.layers[0], layer(1, 0x0600_0009, 0xAA));
    }

    #[test]
    fn a_new_layer_joins_in_id_order() {
        let mut icon = Ac2IconDesc::default();
        for id in [5, 1, 3, 3, 9] {
            icon.apply_additive_layer(&layer(id, 0x0600_0000 + id, 0));
        }
        let ids: Vec<_> = icon.layers.iter().map(|l| l.layer_id).collect();
        assert_eq!(ids, [1, 3, 5, 9]);
    }

    #[test]
    fn a_whole_icon_desc_applies_layer_by_layer() {
        let mut ours = Ac2IconDesc {
            layers: vec![layer(1, 10, 1), layer(2, 20, 2)],
        };
        let theirs = Ac2IconDesc {
            layers: vec![layer(2, 21, 3), layer(3, 30, 4)],
        };
        assert!(ours.apply_additive_desc(&theirs));
        assert_eq!(
            ours.layers,
            [layer(1, 10, 1), layer(2, 21, 3), layer(3, 30, 4)]
        );
    }
}
