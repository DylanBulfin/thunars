use std::ops::Add;

use crate::{
    components::{Component, FileList},
    tui::Tui,
    Result,
};
use ratatui::{
    layout::Rect,
    widgets::{self, Widget},
};

#[derive(Clone, Copy)]
struct RelRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Clone)]
pub struct Segment {
    component: Component,
    area: RelRect,
    visible: bool,
}

pub struct Browser {
    segments: Vec<Segment>,
    terminal: Tui,
    exit: bool,
}

impl Browser {
    pub fn init(terminal: Tui) -> Result<Browser> {
        unimplemented!()
    }

    pub fn run(&self) -> Result<()> {
        unimplemented!()
    }

    fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let mut buf = f.buffer_mut();
            for segment in self.segments.iter() {
                segment.component.render(segment.area, buf);
            }
        })?;

        unimplemented!()
    }
}
