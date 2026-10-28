//! the library as a dashboard: each game a card that says how far it is from
//! being played, with one button for the next step, and a welcome that
//! walks someone with no games folder yet through it.

use std::collections::HashMap;
use std::path::PathBuf;

use egui::{Align, Color32, Frame, Layout, Margin, RichText, Vec2};

use super::{cover, stage_text, Action, Menus};
use crate::library::{Game, Library, ICON_SIZE};
use crate::recompile::{Job, Stage};
use crate::settings::Settings;

/// the card's width at the most, and the space between cards.
const CARD_WIDTH: f32 = 360.0;
const GAP: f32 = 12.0;
/// the space inside a card.
const PADDING: f32 = 14.0;

/// where a game is on its way to being played.
enum Status<'a> {
    /// something about the game keeps it from being played.
    Problem(&'a str),
    Preparing,
    Failed(String),
    Ready,
    Waiting,
}

impl Status<'_> {
    /// the group of the dashboard the game is in.
    fn group(&self) -> usize {
        match self {
            Status::Preparing => 0,
            Status::Ready => 1,
            _ => 2,
        }
    }
}

fn status<'a>(game: &'a Game, job: Option<&Job>) -> Status<'a> {
    if let Some(problem) = &game.problem {
        return Status::Problem(problem);
    }
    match job.map(Job::stage) {
        Some(Stage::Failed(error)) => return Status::Failed(error),
        Some(stage) if !stage.finished() => return Status::Preparing,
        _ => {}
    }
    if game.recompiled {
        Status::Ready
    } else {
        Status::Waiting
    }
}

/// the colors of the kinds of status, lighter on a dark window.
struct Tones {
    ready: Color32,
    waiting: Color32,
    failed: Color32,
    busy: Color32,
}

impl Tones {
    fn of(ui: &egui::Ui) -> Tones {
        if ui.visuals().dark_mode {
            Tones {
                ready: Color32::from_rgb(110, 200, 120),
                waiting: Color32::from_rgb(230, 170, 80),
                failed: Color32::from_rgb(240, 110, 100),
                busy: Color32::from_rgb(100, 160, 255),
            }
        } else {
            Tones {
                ready: Color32::from_rgb(36, 138, 61),
                waiting: Color32::from_rgb(176, 110, 0),
                failed: Color32::from_rgb(200, 40, 40),
                busy: Color32::from_rgb(0, 100, 220),
            }
        }
    }
}

/// a rounded label that says what state something is in.
fn pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    Frame::new().fill(color.gamma_multiply(0.16)).corner_radius(10).inner_margin(Margin::symmetric(8, 2)).show(ui, |ui| {
        ui.label(RichText::new(text).small().strong().color(color));
    });
}

/// the button of a card's next step, across the card.
fn primary(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(RichText::new(text).size(15.0).strong().color(Color32::WHITE)).fill(ui.visuals().selection.bg_fill).corner_radius(8);
    ui.add_sized([ui.available_width(), 32.0], button)
}

impl Menus {
    /// the library, filling the window.
    pub(super) fn dashboard(&mut self, ui: &mut egui::Ui, library: &Library, settings: &Settings, jobs: &[Job]) -> Vec<Action> {
        let mut actions = Vec::new();
        if library.changed {
            self.icons.clear();
        }
        let margin = Margin::symmetric(20, 14);
        egui::Panel::top("dashboard bar").frame(Frame::side_top_panel(ui.style()).inner_margin(margin)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Zakuro").size(24.0).strong());
                    let ready = library.games.iter().filter(|game| game.recompiled).count();
                    let summary = match (&settings.games, library.games.len()) {
                        (None, _) => "Your 3DS games, on your Mac".to_owned(),
                        (Some(_), 1) => format!("1 game, {ready} ready"),
                        (Some(_), games) => format!("{games} games, {ready} ready"),
                    };
                    ui.label(RichText::new(summary).weak());
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Settings").clicked() {
                        self.settings_open = true;
                    }
                    if ui.button("Multiplayer").clicked() {
                        self.multiplayer_open = true;
                    }
                    if settings.games.is_some() {
                        if ui.button("Refresh").clicked() {
                            actions.push(Action::Rescan);
                        }
                        if ui.button("Change folder…").clicked() {
                            actions.push(Action::ChooseFolder);
                        }
                    }
                });
            });
        });
        let background = match self.background.get(ui.ctx(), settings.background.as_deref()) {
            Ok(texture) => texture.map(|texture| (texture.id(), texture.size())),
            Err(error) => {
                self.message = Some(error);
                None
            }
        };
        egui::CentralPanel::default().frame(Frame::central_panel(ui.style()).inner_margin(margin)).show(ui, |ui| {
            if let Some((texture, size)) = background {
                let area = ui.clip_rect();
                let alpha = (settings.background_opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
                ui.painter().image(texture, area, cover(area.size(), size), Color32::from_white_alpha(alpha));
            }
            if settings.games.is_none() {
                welcome(ui, &mut actions);
                return;
            }
            if library.is_scanning() {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Looking for games…");
                });
                return;
            }
            if library.games.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 4.0);
                    ui.label(RichText::new("No games in that folder").size(18.0).strong());
                    ui.label(RichText::new("Zakuro looks for decrypted .3ds, .cci, .cxi and .cia files.").weak());
                    ui.add_space(8.0);
                    if ui.button("Choose another folder…").clicked() {
                        actions.push(Action::ChooseFolder);
                    }
                });
                return;
            }
            let tones = Tones::of(ui);
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                let columns = ((ui.available_width() + GAP) / (CARD_WIDTH + GAP)).floor().max(1.0);
                let width = ((ui.available_width() - GAP * (columns - 1.0)) / columns).floor() - 1.0;
                ui.spacing_mut().item_spacing = Vec2::splat(GAP);
                let statuses: Vec<_> = library.games.iter().map(|game| status(game, jobs.iter().find(|job| job.program_id == game.program_id))).collect();
                for (group, title) in ["Getting ready", "Ready to play", "Needs preparing"].into_iter().enumerate() {
                    let shown: Vec<usize> = (0..library.games.len()).filter(|&index| statuses[index].group() == group).collect();
                    if shown.is_empty() {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(title).size(16.0).strong());
                        ui.label(RichText::new(shown.len().to_string()).weak());
                    });
                    ui.horizontal_wrapped(|ui| {
                        for index in shown {
                            let game = &library.games[index];
                            let job = jobs.iter().find(|job| job.program_id == game.program_id);
                            card(ui, &mut self.icons, &tones, (index, game, &statuses[index], job), (width, background.is_some()), &mut actions);
                        }
                    });
                    ui.add_space(8.0);
                }
            });
        });
        actions
    }
}

/// what someone with no games folder yet is told, with the way to pick one.
fn welcome(ui: &mut egui::Ui, actions: &mut Vec<Action>) {
    const STEPS: [(&str, &str); 3] = [
        ("Choose your games", "Pick the folder with the 3DS games you own."),
        ("Get ready", "Zakuro prepares each game once, which takes around ten minutes."),
        ("Play", "From then on the game runs natively on your Mac."),
    ];
    const STEP_WIDTH: f32 = 200.0;
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() / 6.0).min(80.0));
        ui.label(RichText::new("Welcome to Zakuro").size(30.0).strong());
        ui.label(RichText::new("Play your Nintendo 3DS games on your Mac.").size(15.0).weak());
        ui.add_space(24.0);
        ui.horizontal(|ui| {
            let total = STEPS.len() as f32 * (STEP_WIDTH + 2.0 * PADDING + GAP) - GAP;
            ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
            ui.spacing_mut().item_spacing.x = GAP;
            for (number, (title, text)) in STEPS.into_iter().enumerate() {
                Frame::new()
                    .fill(ui.visuals().window_fill)
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .corner_radius(12)
                    .inner_margin(PADDING)
                    .show(ui, |ui| {
                        // a frame takes the layout it is in, which is a row here
                        ui.vertical(|ui| {
                            ui.set_width(STEP_WIDTH);
                            ui.set_min_height(110.0);
                            ui.label(RichText::new((number + 1).to_string()).size(22.0).strong().color(ui.visuals().selection.bg_fill));
                            ui.label(RichText::new(title).strong());
                            ui.label(RichText::new(text).small().weak());
                        });
                    });
            }
        });
        ui.add_space(28.0);
        let button = egui::Button::new(RichText::new("Choose games folder…").size(16.0).strong().color(Color32::WHITE))
            .fill(ui.visuals().selection.bg_fill)
            .corner_radius(8);
        if ui.add_sized([240.0, 38.0], button).clicked() {
            actions.push(Action::ChooseFolder);
        }
        ui.add_space(8.0);
        ui.label(RichText::new("Games have to be decrypted .3ds, .cci, .cxi or .cia files that you dumped yourself.").small().weak());
    });
}

/// one game: what it is, where it stands and the next step.
fn card(
    ui: &mut egui::Ui,
    icons: &mut HashMap<PathBuf, egui::TextureHandle>,
    tones: &Tones,
    (index, game, status, job): (usize, &Game, &Status, Option<&Job>),
    (width, backed): (f32, bool),
    actions: &mut Vec<Action>,
) {
    // over a picture, each game gets a backing to keep it readable
    let fill = if backed { ui.visuals().window_fill.gamma_multiply(0.9) } else { ui.visuals().window_fill };
    Frame::new().fill(fill).stroke(ui.visuals().widgets.noninteractive.bg_stroke).corner_radius(12).inner_margin(PADDING).show(ui, |ui| {
        // a frame takes the layout it is in, which is a row here
        ui.vertical(|ui| {
            ui.set_width(width - 2.0 * PADDING);
            ui.set_min_height(150.0);
            ui.horizontal(|ui| {
                let icon = game.icon.as_ref().map(|pixels| {
                    icons.entry(game.path.clone()).or_insert_with(|| {
                        let image = egui::ColorImage::from_rgba_unmultiplied([ICON_SIZE, ICON_SIZE], pixels);
                        ui.ctx().load_texture(game.path.display().to_string(), image, egui::TextureOptions::LINEAR)
                    })
                });
                match icon {
                    Some(texture) => {
                        ui.add(egui::Image::new(&*texture).fit_to_exact_size(Vec2::splat(ICON_SIZE as f32)).corner_radius(8));
                    }
                    None => {
                        ui.allocate_space(Vec2::splat(ICON_SIZE as f32));
                    }
                }
                ui.vertical(|ui| {
                    ui.add(egui::Label::new(RichText::new(&game.name).strong().size(16.0)).truncate());
                    if !game.publisher.is_empty() {
                        ui.add(egui::Label::new(RichText::new(&game.publisher).weak()).truncate());
                    }
                    match status {
                        Status::Problem(_) => pill(ui, "Can't be played", tones.waiting),
                        Status::Preparing => pill(ui, "Getting ready", tones.busy),
                        Status::Failed(_) => pill(ui, "Failed", tones.failed),
                        Status::Ready => pill(ui, "Ready", tones.ready),
                        Status::Waiting => pill(ui, "Needs preparing", tones.waiting),
                    }
                });
            });
            ui.add_space(6.0);
            match status {
                Status::Problem(problem) => {
                    ui.label(RichText::new(*problem).small().color(tones.waiting));
                }
                Status::Preparing => {
                    if let Some(job) = job {
                        let stage = job.stage();
                        let elapsed = job.elapsed().as_secs();
                        let text = format!("{}, {}:{:02}", stage_text(&stage), elapsed / 60, elapsed % 60);
                        ui.add(egui::ProgressBar::new(stage.fraction()).desired_width(ui.available_width()).text(text));
                        if ui.small_button("Cancel").clicked() {
                            actions.push(Action::CancelRecompile(job.program_id));
                        }
                    }
                }
                Status::Ready => {
                    if primary(ui, "▶ Play").clicked() {
                        actions.push(Action::Play(game.path.clone()));
                    }
                    ui.horizontal(|ui| {
                        mods(ui, game, actions);
                        if ui.small_button("Prepare again").clicked() {
                            actions.push(Action::Recompile(index));
                        }
                    });
                }
                Status::Failed(_) | Status::Waiting => {
                    if let Status::Failed(error) = status {
                        ui.label(RichText::new(format!("failed, {error}")).small().color(tones.failed));
                    }
                    let label = if matches!(status, Status::Failed(_)) { "Try again" } else { "Get ready to play" };
                    let hint = "Turn the game's code into native code with 3dsrecomp, which makes it run faster. It takes around ten minutes and needs a C compiler, gcc or clang, or Zig, which Zakuro offers to download when the computer has neither.";
                    if primary(ui, label).on_hover_text(hint).clicked() {
                        actions.push(Action::Recompile(index));
                    }
                    ui.horizontal(|ui| {
                        if ui.small_button("Play anyway").on_hover_text("Runs slower, in the interpreter.").clicked() {
                            actions.push(Action::Play(game.path.clone()));
                        }
                        mods(ui, game, actions);
                    });
                }
            }
        });
    });
}

fn mods(ui: &mut egui::Ui, game: &Game, actions: &mut Vec<Action>) {
    let hint = "Open the game's mods folder. Files put in its romfs folder replace the game's, the way Luma3DS and Citra take mods, and a texture pack made for Citra or Azahar goes in its textures folder.";
    if ui.small_button("Mods").on_hover_text(hint).clicked() {
        actions.push(Action::Mods(game.program_id));
    }
}
