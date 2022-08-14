

struct Sidebar {
    search: String,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            search: String::new(),
        }
    }

    pub fn view(&self) {}
}
