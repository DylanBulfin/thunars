use ratatui::{
    prelude::{Buffer, Rect},
    text::{Line, Text},
    widgets::{block::Title, Block, Paragraph, Widget},
};

#[derive(Clone)]
pub struct FileList {
    pub(crate) files: Vec<String>,
    pub(crate) height: usize,
    pub(crate) pos: usize,
}

impl FileList {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let title = Title::from("Files");
        let text = Text::from(
            self.files[self.pos..self.pos + self.height]
                .iter()
                .map(|f| Line::from(f.as_str()))
                .collect::<Vec<_>>(),
        );

        let block = Block::bordered().title(title);

        Paragraph::new(text).block(block).render(area, buf);
    }
}

#[derive(Clone)]
pub enum Component {
    Files(FileList),
}

impl Widget for Component {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self {
            Component::Files(f) => f.render(area, buf),
        }
    }
}
