use egui::Color32;

pub struct Colors {
    pub exe: Color32,
}

pub struct Style {
    pub colors: Colors,
    pub row_height: f32,
}

impl Style {
    pub fn default() -> Self {
        Self {
            colors: Colors {
                exe: Color32::GREEN,
            },
            row_height: 16.0,
        }
    }
}
