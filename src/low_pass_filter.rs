struct LpFilter {
    cutoff: f32,
}

impl LpFilter {
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
    }
}
