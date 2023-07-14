pub(crate) struct Led {
    red: u8,
    green: u8,
    blue: u8,
}

impl Led {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
        }
    }

}