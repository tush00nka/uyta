pub struct Player {
    pub money: usize,
    pub display_money: usize, 
}

impl Player {
    pub fn new() -> Self {
        Self {
            money: 100,
            display_money: 0
        }
    }
}