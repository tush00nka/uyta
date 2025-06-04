const MAX_ITEM_STACK: usize = 99; 

#[derive(Copy, Clone)]
pub enum ToolFunction {
    Hoe,
    Chop,
    Mine,
}

#[derive(Clone, Copy)]
pub enum ItemType {
    Tool(ToolFunction),
    Seed
}

#[derive(Clone, Copy)]
pub struct InventorySlot {
    item: Option<ItemType>,
    amount: usize,
}

impl InventorySlot {
    fn empty() -> Self {
        Self {
            item: None,
            amount: 0
        }
    }
}

pub struct Inventory {
    pub slots: [InventorySlot; 8]
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            slots: [InventorySlot::empty(); 8]
        }
    }

    pub fn add_item(&mut self, item: ItemType) {
        for slot in self.slots.iter() {
            
        }
    }
}