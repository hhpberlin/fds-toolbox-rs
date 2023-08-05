struct CheckList {
    items: Vec<CheckListItem>,
}

struct CheckListItem {
    text: Cow<String>,
    checked: bool,
}