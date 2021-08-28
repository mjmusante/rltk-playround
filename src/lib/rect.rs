pub struct Rect {
    pub top: i32,
    pub lft: i32,
    pub bot: i32,
    pub rht: i32,
}

impl Rect {
    pub fn new(lft: i32, top: i32, wid: i32, hgt: i32) -> Rect {
        Rect {
            top,
            lft,
            bot: top + hgt,
            rht: lft + wid,
        }
    }

    pub fn intersect(&self, other: &Rect) -> bool {
        self.lft <= other.rht
            && self.rht >= other.lft
            && self.top <= other.bot
            && self.bot >= other.top
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.lft + self.rht) / 2, (self.top + self.bot) / 2)
    }
}
