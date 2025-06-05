pub struct Player {
    pub money: usize,
    pub display_money: usize,
    pub level: usize,
    pub exp: usize,
    pub exp_to_lvl_up: usize,
}

impl Player {
    pub fn new() -> Self {
        Self {
            money: 100,
            display_money: 0,
            level: 1,
            exp: 0,
            exp_to_lvl_up: 20,
        }
    }

    pub fn update_money(&mut self) {
        let money_diff = self.money as isize - self.display_money as isize;
        if money_diff == 0 {
            return;
        }

        self.display_money =
            (self.display_money as isize + money_diff / money_diff.abs()) as usize;
    }

    pub fn update_exp(&mut self) {
        if self.exp >= self.exp_to_lvl_up {
            self.level+=1;
            self.exp = 0;
            self.exp_to_lvl_up *= 3;
        }
    }
}
