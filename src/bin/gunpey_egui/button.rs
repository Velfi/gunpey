pub struct Button<T>
where
    T: Fn() -> (),
{
    is_hovered: bool,
    is_active: bool,
    disabled: bool,
    on_click: T,
    label: String,
}

impl<T> Button<T>
where
    T: Fn() -> (),
{
    pub fn new(label: String, on_click: T) -> Self {
        Self {
            is_active: false,
            is_hovered: false,
            disabled: false,
            label,
            on_click,
        }
    }
}
