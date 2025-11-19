use anyhow::Result;
use shared::{read_message, send_message, ClientMessage, ServerMessage};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey, SmolStr},
    window::{Fullscreen, Window},
};

use crate::{
    client_player::ClientPlayer,
    engine::{init_textures, Graphics, TexInfo, Texture, PLAYER_TEXTURES},
    map::{ClientTileManager, Drawable},
};

struct GameManager {
    last_frame: Instant,
    target_frame_duration: Duration,

    window: Option<Arc<Window>>,
    fullscreen: bool,

    graphics: Option<Graphics>,

    pressed_named_keys: HashSet<NamedKey>,
    pressed_keys: HashSet<SmolStr>,

    // camera: Option<Camera>,
    player: Option<ClientPlayer>,
    other_players: HashMap<String, ClientPlayer>,

    tile_manager: Option<ClientTileManager>,

    incoming_rx: UnboundedReceiver<ServerMessage>,
    outgoing_tx: UnboundedSender<ClientMessage>,
}

impl GameManager {
    pub async fn new() -> Result<Self> {
        let stream = TcpStream::connect("game-server.local:5250").await?;
        println!("Connected to server");
        let (mut reader, mut writer) = stream.into_split();

        let (incoming_tx, incoming_rx) = unbounded_channel::<ServerMessage>();
        let incoming_tx_clone = incoming_tx.clone();

        tokio::spawn(async move {
            loop {
                match read_message(&mut reader).await {
                    Ok(msg) => {
                        let _ = incoming_tx_clone.send(msg);
                    }
                    Err(e) => {
                        println!("Disconnected from server: {e}");
                        break;
                    }
                }
            }
        });

        let (outgoing_tx, mut outgoing_rx) = unbounded_channel::<ClientMessage>();

        tokio::spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                if let Err(e) = send_message(&mut writer, &msg).await {
                    println!("Writer task failed: {e}");
                    break;
                }
            }
        });

        Ok(Self {
            last_frame: Instant::now(),
            target_frame_duration: Duration::from_secs_f64(1.0 / 120.0),

            window: None,
            fullscreen: false,

            graphics: None,

            pressed_named_keys: HashSet::new(),
            pressed_keys: HashSet::new(),

            player: None,
            other_players: HashMap::new(),
            tile_manager: None,

            incoming_rx,
            outgoing_tx,
        })
    }

    pub fn handle_named_key(&mut self, key: NamedKey, pressed: bool) {
        if pressed {
            self.pressed_named_keys.insert(key);
        } else {
            self.pressed_named_keys.remove(&key);
        }
    }

    pub fn handle_key(&mut self, key: SmolStr, pressed: bool) {
        if pressed {
            self.pressed_keys.insert(key);
        } else {
            self.pressed_keys.remove(&key);
        }
    }

    pub fn update_window(&mut self) {
        if self.pressed_named_keys.contains(&NamedKey::F11) {
            if let Some(ref window) = self.window {
                if self.fullscreen {
                    window.set_fullscreen(None);
                } else if let Some(monitor) = window.current_monitor() {
                    window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
                }
                self.fullscreen = !self.fullscreen;
            }
        }
        if self.pressed_named_keys.contains(&NamedKey::Escape) {
            if let Some(ref window) = self.window {
                if self.fullscreen {
                    window.set_fullscreen(None);
                    self.fullscreen = false;
                } else {
                    if window.is_maximized() {
                        window.set_maximized(false);
                    } else {
                        window.set_maximized(true);
                    }
                }
            }
        }
    }

    pub fn update_player(&mut self) {
        if let Some(player) = &self.player {
            if self.pressed_keys.contains("w") {
                let _ = self.outgoing_tx.send(ClientMessage::MoveRequest {
                    player: player.id.clone(),
                    direction: [0.0, 1.0, 0.0],
                });
            }
            if self.pressed_keys.contains("s") {
                let _ = self.outgoing_tx.send(ClientMessage::MoveRequest {
                    player: player.id.clone(),
                    direction: [0.0, -1.0, 0.0],
                });
            }
            if self.pressed_keys.contains("a") {
                let _ = self.outgoing_tx.send(ClientMessage::MoveRequest {
                    player: player.id.clone(),
                    direction: [-1.0, 0.0, 0.0],
                });
            }
            if self.pressed_keys.contains("d") {
                let _ = self.outgoing_tx.send(ClientMessage::MoveRequest {
                    player: player.id.clone(),
                    direction: [1.0, 0.0, 0.0],
                });
            }
        }
    }

    pub fn update_camera(&mut self) {
        // if let (Some(graphics), Some(player), Some(camera)) =
        //     (&mut self.graphics, &mut self.player, &mut self.camera)
        // {
        //     let Graphics { queue, .. } = graphics;
        //     camera.update(queue, player.tile.world_position);
        // }
    }

    pub fn update_game(&mut self) {}

    pub fn process_server_input(&mut self) {
        while let Ok(msg) = self.incoming_rx.try_recv() {
            match msg {
                ServerMessage::OtherPlayer(p) => {
                    if let Some(ref graphics) = self.graphics {
                        let Graphics {
                            device,
                            tile_bind_group_layout,
                            queue,
                            ..
                        } = graphics;
                        self.other_players
                            .entry(p.id.clone())
                            .and_modify(|player| {
                                player.update_player(queue, p.clone());
                            })
                            .or_insert_with(|| match PLAYER_TEXTURES.read() {
                                Ok(textures) => {
                                    let tex_info = if let Some(tex_info) =
                                        textures.get(&crate::engine::PlayerTexture::South)
                                    {
                                        tex_info
                                    } else {
                                        &TexInfo {
                                            texture: Arc::new(Texture::from_color(
                                                device,
                                                queue,
                                                [255, 255, 255, 255],
                                            )),
                                            index: [0, 0],
                                        }
                                    };
                                    ClientPlayer::new(
                                        device,
                                        tile_bind_group_layout,
                                        p.id,
                                        p.position,
                                        tex_info,
                                        0.25,
                                        p.speed,
                                    )
                                }
                                Err(e) => {
                                    panic!("Could not get PLAYER_TEXTURES for reading: {e}");
                                }
                            });
                    }
                }
                ServerMessage::Player(p) => {
                    if let Some(ref graphics) = self.graphics {
                        let Graphics {
                            device,
                            queue,
                            tile_bind_group_layout,
                            ..
                        } = graphics;
                        if let Some(player) = &mut self.player {
                            player.update_player(queue, p);
                        } else {
                            match PLAYER_TEXTURES.read() {
                                Ok(textures) => {
                                    if let Some(tex_info) =
                                        textures.get(&crate::engine::PlayerTexture::South)
                                    {
                                        let player = ClientPlayer::new(
                                            device,
                                            tile_bind_group_layout,
                                            p.id,
                                            p.position,
                                            &tex_info,
                                            0.25,
                                            p.speed,
                                        );
                                        self.player = Some(player);
                                    }
                                }
                                Err(e) => {
                                    println!("Could not get PLAYER_TEXTURES for reading: {e}")
                                }
                            }
                        }
                    }
                }
                ServerMessage::Map(m) => {
                    if let Some(ref graphics) = self.graphics {
                        let Graphics {
                            device,
                            tile_bind_group_layout,
                            ..
                        } = graphics;
                        let size = m.size;
                        self.tile_manager = Some(ClientTileManager::from_server(
                            m,
                            &device,
                            &tile_bind_group_layout,
                            size,
                            0.25,
                        ));
                    }
                }
                _ => {}
            }
        }
    }
}

impl ApplicationHandler for GameManager {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        let now = Instant::now();
        let next_frame_time = self.last_frame + self.target_frame_duration;

        self.process_server_input();

        self.update_game();
        self.update_player();
        self.update_camera();

        if now >= next_frame_time || matches!(cause, StartCause::Init) {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            self.last_frame = now;
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            self.last_frame + self.target_frame_duration,
        ));
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            if let Ok(window) = event_loop.create_window(
                Window::default_attributes()
                    .with_title("Isometric Game!")
                    .with_maximized(true),
            ) {
                self.window = Some(Arc::new(window));
            }
        }

        if self.graphics.is_none() {
            if let Some(ref window) = self.window {
                match pollster::block_on(Graphics::new(window)) {
                    Ok(graphics) => {
                        let Graphics { device, queue, .. } = &graphics;
                        match init_textures(device, queue) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Could not init textures: {e}");
                            }
                        }
                        self.graphics = Some(graphics);
                        let _ = self.outgoing_tx.send(ClientMessage::ConnectionRequest);
                    }
                    Err(e) => {
                        println!("Could not create graphics: {e}");
                    }
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let Key::Named(key) = event.logical_key {
                    self.handle_named_key(key, event.state.is_pressed());
                }
                if let Key::Character(ch) = event.logical_key {
                    self.handle_key(ch, event.state.is_pressed());
                }
                self.update_window();
            }
            WindowEvent::RedrawRequested => {
                if let Some(graphics) = &mut self.graphics {
                    if let Some(ref tile_manager) = self.tile_manager {
                        let mut drawables: Vec<&dyn Drawable> = vec![tile_manager];
                        if let Some(ref player) = self.player {
                            drawables.push(player);
                        };
                        drawables.extend(self.other_players.iter().map(|(id, player)| player as &dyn Drawable));
                        
                        if let Err(e) = graphics.render(drawables) {
                            println!("Could not render frame: {e}");
                        }
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(ref mut graphics) = self.graphics {
                    graphics.resize(size.width, size.height);
                }
                // if let (Some(window), Some(camera)) = (&mut self.window, &mut self.camera) {
                //     let PhysicalSize { width, height } = window.inner_size();
                //     camera.update_projection(width as f32 / height as f32);
                // }
            }
            _ => {}
        }
    }
}

pub struct Game {
    event_loop: EventLoop<()>,
    game_manager: GameManager,
}

impl Game {
    pub fn new() -> Result<Self> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        let game_manager = pollster::block_on(GameManager::new())?;
        Ok(Self {
            event_loop,
            game_manager,
        })
    }

    pub fn run(mut self) -> Result<()> {
        self.event_loop.run_app(&mut self.game_manager)?;
        Ok(())
    }
}
