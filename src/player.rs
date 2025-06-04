pub struct Player {
    pub money: usize,
    pub display_money: usize,
}

impl Player {
    pub fn new() -> Self {
        Self {
            money: 100,
            display_money: 0,
        }
    }

    pub fn update_money(&mut self) {
        let money_diff = self.money as isize - self.display_money as isize;
        if money_diff != 0 {
            self.display_money =
                (self.display_money as isize + money_diff / money_diff.abs()) as usize;
        }
    }
}
