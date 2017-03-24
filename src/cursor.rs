use cairo;
use ui_model::Color;
use ui::UI;
use shell::{Shell, NvimMode};
use nvim::{RepaintMode, RedrawEvents};
use std::sync::{Arc, Mutex};

use glib;

struct Alpha(f64);

impl Alpha {
    pub fn show(&mut self, step: f64) -> bool {
        self.0 += step;
        if self.0 > 1.0 {
            self.0 = 1.0;
            false
        } else {
            true
        }
    }
    pub fn hide(&mut self, step: f64) -> bool {
        self.0 -= step;
        if self.0 < 0.0 {
            self.0 = 0.0;
            false
        } else {
            true
        }
    }
}

enum AnimPhase {
    Shown(i32),
    Hide,
    Hidden(i32),
    Show,
}

impl AnimPhase {
    fn sub(&mut self, step: i32) -> bool {
        match self {
            &mut AnimPhase::Shown(ref mut val) |
                &mut AnimPhase::Hidden(ref mut val) => {
                    *val -= step;
                    if *val <= 0 {
                        false
                    } else {
                        true
                    }
                }
            _ => false,
        }
    }
}

pub struct State {
    alpha: Alpha,
    anim_phase: AnimPhase,
}

impl State {
    pub fn new() -> State {
        State {
            alpha: Alpha(1.0),
            anim_phase: AnimPhase::Shown(500),
        }
    }
}

pub struct Cursor {
    timer: Option<glib::SourceId>,

    state: Arc<Mutex<State>>,
}

impl Cursor {
    pub fn new() -> Cursor {

        Cursor {
            timer: None,
            state: Arc::new(Mutex::new(State::new())),
        }

    }

    pub fn start(&mut self) {
        if self.timer.is_none() {
            let state = self.state.clone();
            self.timer = Some(glib::timeout_add(100, move || {
                let mut mut_state = state.lock().unwrap();

                // wait
                // [TODO]: Implement wait through new timeout - 2017-03-24 10:21
                if mut_state.anim_phase.sub(100) {
                    return glib::Continue(true);
                }

                match mut_state.anim_phase {
                    AnimPhase::Shown(_) => {
                        mut_state.anim_phase = AnimPhase::Hide;
                    }
                    AnimPhase::Hide => {
                        if !mut_state.alpha.hide(0.3) {
                            mut_state.anim_phase = AnimPhase::Hidden(300);
                        }
                    }
                    AnimPhase::Hidden(_) => {
                        mut_state.anim_phase = AnimPhase::Show;
                    }
                    AnimPhase::Show => {
                        if !mut_state.alpha.show(0.3) {
                            mut_state.anim_phase = AnimPhase::Shown(500);
                        }
                    }
                }

                SHELL!(&shell = {
                    let point = shell.model.cur_point();
                    shell.on_redraw(&RepaintMode::Area(point));
                });
                glib::Continue(true)
            }));
        }
    }

    pub fn draw(&self,
                ctx: &cairo::Context,
                shell: &Shell,
                char_width: f64,
                line_height: f64,
                line_y: f64,
                double_width: bool,
                bg: &Color) {


        let current_point = ctx.get_current_point();
        let state = self.state.lock().unwrap();
        ctx.set_source_rgba(1.0 - bg.0, 1.0 - bg.1, 1.0 - bg.2, 0.6 * state.alpha.0);

        let cursor_width = if shell.mode == NvimMode::Insert {
            char_width / 5.0
        } else {
            if double_width {
                char_width * 2.0
            } else {
                char_width
            }
        };

        ctx.rectangle(current_point.0, line_y, cursor_width, line_height);
        ctx.fill();
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        if let Some(timer_id) = self.timer {
            glib::source_remove(timer_id);
        }
    }
}