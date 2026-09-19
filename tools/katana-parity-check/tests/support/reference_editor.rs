use egui::{
    Context, Event, Id, RawInput, TextEdit,
    text::{CCursor, CCursorRange},
    widgets::text_edit::TextEditState,
};

const SCREEN_RECT: egui::Rect =
    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(640.0, 480.0));
const STABLE_TIME_SECS: f64 = 1.0;
const FRAME_TIME_SECS: f64 = 0.25;

#[derive(Debug, PartialEq)]
pub struct FrameObservation {
    pub text: String,
    pub cursor: CCursorRange,
    pub changed: bool,
}

pub struct ReferenceEditor {
    ctx: Context,
    edit_id: Id,
    text: String,
    interactive: bool,
    time: f64,
}

impl ReferenceEditor {
    pub fn new(
        edit_id_source: &'static str,
        initial_text: &str,
        cursor: CCursorRange,
        interactive: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let ctx = Context::default();
        let edit_id = Id::new(edit_id_source);
        let mut text = initial_text.to_owned();

        let mut frame = ctx.run_ui(raw_input(0.0, Vec::new()), |ui| {
            ui.add(
                TextEdit::multiline(&mut text)
                    .id(edit_id)
                    .interactive(interactive),
            );
        });
        frame.textures_delta.clear();

        ctx.memory_mut(|memory| memory.request_focus(edit_id));
        let mut state =
            TextEditState::load(&ctx, edit_id).ok_or("initial TextEditState missing")?;
        state.cursor.set_char_range(Some(cursor));
        state.store(&ctx, edit_id);

        Ok(Self {
            ctx,
            edit_id,
            text,
            interactive,
            time: 0.0,
        })
    }

    pub fn frame(
        &mut self,
        events: Vec<Event>,
    ) -> Result<FrameObservation, Box<dyn std::error::Error>> {
        self.time += FRAME_TIME_SECS;
        self.run_frame(events)
    }

    pub fn stable_frame(&mut self) -> Result<FrameObservation, Box<dyn std::error::Error>> {
        self.time += STABLE_TIME_SECS;
        self.run_frame(Vec::new())
    }

    fn run_frame(
        &mut self,
        events: Vec<Event>,
    ) -> Result<FrameObservation, Box<dyn std::error::Error>> {
        let mut changed = false;
        let mut frame = self.ctx.run_ui(raw_input(self.time, events), |ui| {
            let response = ui.add(
                TextEdit::multiline(&mut self.text)
                    .id(self.edit_id)
                    .interactive(self.interactive),
            );
            changed = response.changed();
        });
        frame.textures_delta.clear();

        let state = TextEditState::load(&self.ctx, self.edit_id).ok_or("TextEditState missing")?;
        let cursor = state.cursor.char_range().ok_or("cursor missing")?;
        Ok(FrameObservation {
            text: self.text.clone(),
            cursor,
            changed,
        })
    }
}

fn raw_input(time: f64, events: Vec<Event>) -> RawInput {
    RawInput {
        screen_rect: Some(SCREEN_RECT),
        time: Some(time),
        events,
        ..RawInput::default()
    }
}

pub fn cursor(index: usize) -> CCursorRange {
    CCursorRange::one(CCursor::new(index))
}
