//! `OptionExpander::Refit`, shared by the three options pages (character, chat, client): after an
//! expander is opened (`Construct`) or closed (`Dismantle`) its element is as tall as the header
//! plus the options it lists, and its list is as tall as those options.

use crate::element::UiTree;

/// `OptionExpander::Refit`: the list's items' heights are summed, the expander's element is
/// resized to its own width and the header's height plus that sum, and the list to its own width
/// and the sum. Nothing changes without both the list and the header.
pub(crate) fn refit(tree: &mut UiTree, element: usize, header: Option<usize>, list: Option<usize>) {
    let (Some(header), Some(list)) = (header, list) else {
        return;
    };
    let (Some(header_box), Some(element_box), Some(list_box)) =
        (tree.get(header), tree.get(element), tree.get(list))
    else {
        return;
    };
    let header_height = header_box.rect().height();
    let element_width = element_box.rect().width();
    let list_width = list_box.rect().width();
    let items: i32 = tree
        .list_items(list)
        .iter()
        .filter_map(|&item| tree.get(item))
        .map(|item| item.rect().height())
        .sum();
    tree.resize_to(element, element_width, header_height + items);
    tree.resize_to(list, list_width, items);
}
