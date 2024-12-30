use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
};

use draw::draw_board;
use eframe::egui::{self};
use neurojam24_core::{
    ActionType, Direction, DisplayData, GameResult, NetBlob, PlayerAction, PlayerStatus,
    SpatialDirection, TemporalDirection, LENGTH, SIZE,
};
use tungstenite::Message;

mod draw;

/// Native main function
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    // disable for dev
    std::panic::set_hook(Box::new(|panic_info| {
        let title = "The game crashed!";
        let backtrace = std::backtrace::Backtrace::force_capture();
        let contents = format!("{title}\n\n{panic_info}\n\n{backtrace}");
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title(title)
            .set_description(contents)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }));

    let native_options = eframe::NativeOptions {
        // follow_system_theme: false,
        ..Default::default()
    };

    eframe::run_native(
        "NeuroJam",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[derive(Debug, Default, Copy, Clone)]
struct Input {
    player_id: u8,
    spatial: Option<SpatialDirection>,
    temporal: Option<TemporalDirection>,
    action: Option<ActionType>,
    confirmed: bool,
}
impl Input {
    pub fn new(player_id: u8) -> Self {
        Self {
            player_id,
            spatial: None,
            temporal: None,
            action: None,
            confirmed: false,
        }
    }

    pub fn evaluate(&self) -> Option<PlayerAction> {
        if self.confirmed {
            Some(PlayerAction {
                direction: Direction {
                    spatial: self.spatial?,
                    temporal: self.temporal?,
                },
                action_type: self.action?,
            })
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.spatial = None;
        self.temporal = None;
        self.action = None;
        self.confirmed = false;
    }
}

#[derive(Debug, Clone)]
struct Info {
    display: Option<DisplayData>,
    player_stati: Option<[PlayerStatus; 2]>,
    inputs: [Option<Input>; 2],
    result: Option<GameResult>,
}
impl Info {
    fn new() -> Self {
        Self {
            display: None,
            player_stati: None,
            inputs: [None; 2],
            result: None,
        }
    }
}

struct App {
    game_info: Arc<Mutex<Info>>,
    view_slice: usize,
}
impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let game_info = Arc::new(Mutex::new(Info::new()));

        let info_ref = game_info.clone();
        thread::spawn(move || {
            let path = std::env::current_exe()
                .ok()
                .unwrap()
                .canonicalize()
                .unwrap()
                .parent()
                .unwrap()
                .join("ip.txt");
            let ip = std::fs::read_to_string(path)
                .unwrap()
                .trim()
                .lines()
                .next()
                .expect("No ip?")
                .to_string();
            let stream = TcpStream::connect((ip, 4444)).expect("Can't connect");
            let fnuy = tungstenite::client("ws://socket", stream);
            let (mut socket, _) = fnuy.unwrap();
            socket.get_mut().set_nonblocking(true);

            const TICK_LENGTH: std::time::Duration = std::time::Duration::from_millis(5);
            socket
                .send(Message::Text(NetBlob::Join.ser().into()))
                .unwrap();
            loop {
                let frame_time = std::time::Instant::now();
                let mut clear_input = false;
                if let Some(input) =
                    info_ref.lock().unwrap().inputs[0].and_then(|input| input.evaluate())
                {
                    socket.send(Message::Text(NetBlob::Action(input).ser().into()));
                    clear_input = true;
                }
                if clear_input {
                    if let Some(input) = &mut info_ref.lock().unwrap().inputs[0] {
                        input.clear();
                    };
                }
                while let Ok(msg) = socket.read() {
                    if msg.is_text() {
                        match NetBlob::deser(msg.into_text().expect("It should be").as_str()) {
                            Ok(blob) => match blob {
                                NetBlob::Join => todo!(),
                                NetBlob::Assign(id) => {
                                    println!("Joined as player {}", id);
                                    info_ref.lock().unwrap().inputs[0] = Some(Input::new(id));
                                }
                                NetBlob::Action(_) => todo!(),
                                NetBlob::Leave => todo!(),
                                NetBlob::Display(data) => {
                                    info_ref.lock().unwrap().display = Some(data);
                                }
                                NetBlob::Stati(stati) => {
                                    info_ref.lock().unwrap().player_stati = Some(stati);
                                }
                                NetBlob::Result(result) => {
                                    info_ref.lock().unwrap().result = Some(result);
                                }
                                NetBlob::Start => {}
                            },
                            Err(_) => todo!(),
                        }
                    }
                }
                if frame_time.elapsed() < TICK_LENGTH {
                    std::thread::sleep(TICK_LENGTH - frame_time.elapsed());
                }
            }
        });

        Self {
            game_info,
            view_slice: 0,
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("View time:");
                ui.add(egui::Slider::new(&mut self.view_slice, 0..=(LENGTH - 1)));
                // if let Some(stati) = self.game_info.lock().unwrap().player_stati {
                //     for status in stati {
                //         ui.label(status.health.to_string());
                //     }
                // }
                if let Some(result) = self.game_info.lock().unwrap().result {
                    ui.label(match result {
                        GameResult::Win(player_id) => format!("Win: {}", player_id),
                        GameResult::Draw => "Draw".to_string(),
                    });
                }
            });
            let rect = ui.available_rect_before_wrap();

            fn bind<T: Copy>(
                ui: &mut egui::Ui,
                controls: &[(eframe::egui::Key, T)],
                target: &mut Option<T>,
            ) {
                ui.input(|i| {
                    for &(key, action) in controls {
                        if i.key_pressed(key) {
                            *target = Some(action)
                        }
                    }
                })
            }
            if let Some(input) = &mut self.game_info.lock().unwrap().inputs[0] {
                let space_controls = [
                    (egui::Key::W, SpatialDirection::Up),
                    (egui::Key::S, SpatialDirection::Down),
                    (egui::Key::A, SpatialDirection::Left),
                    (egui::Key::D, SpatialDirection::Right),
                ];
                bind(ui, &space_controls, &mut input.spatial);
                let time_controls = [
                    (egui::Key::Q, TemporalDirection::Backward),
                    (egui::Key::E, TemporalDirection::Forward),
                ];
                bind(ui, &time_controls, &mut input.temporal);
                let action_controls = [
                    (egui::Key::X, ActionType::Attack),
                    (egui::Key::Z, ActionType::Move),
                ];
                bind(ui, &action_controls, &mut input.action);
            }
            if let Some(input) = &mut self.game_info.lock().unwrap().inputs[1] {
                let space_controls = [
                    (egui::Key::I, SpatialDirection::Up),
                    (egui::Key::K, SpatialDirection::Down),
                    (egui::Key::J, SpatialDirection::Left),
                    (egui::Key::L, SpatialDirection::Right),
                ];
                bind(ui, &space_controls, &mut input.spatial);
                let time_controls = [
                    (egui::Key::U, TemporalDirection::Backward),
                    (egui::Key::O, TemporalDirection::Forward),
                ];
                bind(ui, &time_controls, &mut input.temporal);
                let action_controls = [
                    (egui::Key::Comma, ActionType::Attack),
                    (egui::Key::M, ActionType::Move),
                ];
                bind(ui, &action_controls, &mut input.action);
            }

            if ui.input(|i| i.key_pressed(egui::Key::Space)) {
                // dbg!(self.game_info.lock().unwrap().inputs[0]);
                if let Some(input) = &mut self.game_info.lock().unwrap().inputs[0] {
                    input.confirmed = true;
                }
                if let Some(input) = &mut self.game_info.lock().unwrap().inputs[1] {
                    input.confirmed = true;
                }
            }
            // let display = self.game_status.display();
            let guard = self.game_info.lock().unwrap();
            let Info {
                display,
                player_stati,
                inputs,
                result,
            } = &*guard;
            if let Some(display) = display {
                draw_board(ui, rect, display, self.view_slice, inputs[0]);
            }
        });
    }
}
