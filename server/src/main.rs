use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
    thread::spawn,
};

use neurojam24_core::{GameResult, GameStatus, NetBlob, PlayerAction};
use tungstenite::{accept, Message};

pub struct Server {
    game_status: GameStatus,
    inputs: [Option<PlayerAction>; 2],
    needs_send: [bool; 2],
    player: [bool; 2],
    result: Option<GameResult>,
}
impl Server {
    fn new() -> Self {
        let game_status = GameStatus::initial();
        let inputs = [None; 2];
        let needs_send = [true; 2];
        let player = [false; 2];
        let result = None;
        Self {
            game_status,
            inputs,
            needs_send,
            player,
            result,
        }
    }

    pub fn set_input(&mut self, player_id: u8, action: PlayerAction) {
        self.inputs[player_id as usize] = Some(action);
    }
}

const TICK_LENGTH: std::time::Duration = std::time::Duration::from_millis(5);

fn main() {
    let game_server = Arc::new(Mutex::new(Server::new()));
    let server = TcpListener::bind("0.0.0.0:4444").unwrap();
    server.set_nonblocking(true);
    for stream in server.incoming() {
        if stream.is_ok() {
            let server_ref = game_server.clone();
            spawn(move || {
                let mut socket = accept(stream.unwrap()).unwrap();

                let mut player_id = None;
                let mut frame_time;

                loop {
                    frame_time = std::time::Instant::now();
                    while let Ok(msg) = socket.read() {
                        // dbg!(&msg);
                        if msg.is_text() {
                            match NetBlob::deser(msg.into_text().expect("It should be").as_str()) {
                                Ok(blob) => match blob {
                                    NetBlob::Join => {
                                        println!("Join requested");
                                        let mut game_server = server_ref.lock().unwrap();
                                        if let Some(id) = (0..2).find(|&i| !game_server.player[i]) {
                                            if !game_server.player[id as usize] {
                                                player_id = Some(id as u8);
                                                game_server.player[id as usize] = true;
                                                println!("Assigned id {}", id);
                                                socket.send(Message::Text(
                                                    NetBlob::Assign(id as u8).ser().into(),
                                                ));
                                            }
                                        }
                                    }
                                    NetBlob::Assign(_) => todo!(),
                                    NetBlob::Action(action) => {
                                        if let Some(player_id) = player_id {
                                            println!("Received move for player {}", player_id);
                                            let mut game_server = server_ref.lock().unwrap();
                                            game_server.set_input(player_id, action);
                                            if game_server.inputs.iter().all(|i| i.is_some()) {
                                                let inputs = game_server
                                                    .inputs
                                                    .map(|i| i.expect("verified"));
                                                let res =
                                                    game_server.game_status.push_actions(inputs);
                                                match res {
                                                    Ok(game_result) => {
                                                        game_server.result = game_result;
                                                        for i in 0..2 {
                                                            game_server.needs_send[i] = true;
                                                        }
                                                    }
                                                    Err(error) => match error {
                                                        neurojam24_core::Error::InvalidMove(
                                                            reason,
                                                        ) => {
                                                            println!(
                                                                "Invalid move (Reason: {})",
                                                                reason
                                                            );
                                                            game_server.inputs = [None; 2];
                                                        }
                                                    },
                                                }
                                            }
                                        }
                                    }
                                    NetBlob::Leave => {
                                        if let Some(player_id) = &mut player_id {
                                            println!("Player {} left", player_id);
                                            server_ref.lock().unwrap().player
                                                [*player_id as usize] = false;
                                        }
                                        player_id = None
                                    }
                                    NetBlob::Display(_) => todo!(),
                                    NetBlob::Stati(_) => todo!(),
                                    NetBlob::Result(_) => todo!(),
                                    NetBlob::Start => todo!(),
                                },
                                Err(_) => {
                                    dbg!("Bad message");
                                }
                            };
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    let mut game_server = server_ref.lock().unwrap();
                    if let Some(player_id) = player_id {
                        if game_server.needs_send[player_id as usize] {
                            socket.send(Message::Text(
                                NetBlob::Display(game_server.game_status.display())
                                    .ser()
                                    .into(),
                            ));
                            socket.send(Message::Text(
                                NetBlob::Stati(game_server.game_status.player_stati)
                                    .ser()
                                    .into(),
                            ));
                            if let Some(result) = game_server.result {
                                match result {
                                    GameResult::Win(player_id) => {
                                        println!("Player {} wins", player_id)
                                    }
                                    GameResult::Draw => println!("Draw"),
                                }
                                socket.send(Message::Text(NetBlob::Result(result).ser().into()));
                            }
                            socket.send(Message::Text(NetBlob::Start.ser().into()));
                            game_server.needs_send[player_id as usize] = false;
                            game_server.inputs[player_id as usize] = None;
                        }
                    }
                    drop(game_server);
                    if frame_time.elapsed() < TICK_LENGTH {
                        std::thread::sleep(TICK_LENGTH - frame_time.elapsed());
                    }
                }
            });
        } else {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
}
