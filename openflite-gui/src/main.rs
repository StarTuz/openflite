use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, vertical_space,
};
use iced::{
    executor, Alignment, Application, Color, Command, Element, Length, Settings, Subscription,
    Theme,
};
use openflite_core::{Core, Event};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub fn main() -> iced::Result {
    env_logger::init();
    OpenFliteApp::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(900.0, 600.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

struct OpenFliteApp {
    devices: Vec<String>,
    error_msg: Option<String>,
    core: Arc<Core>,
    event_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<Event>>>>,
    is_scanning: bool,
    sim_status: String,
}

#[derive(Debug, Clone)]
enum Message {
    ScanDevices,
    ScanResult(Result<(), String>),
    ConnectSim,
    SimResult(Result<(), String>),
    CoreEvent(Event),
}

impl Application for OpenFliteApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let (core, event_rx) = Core::new();
        let core = Arc::new(core);

        let core_clone = core.clone();
        tokio::spawn(async move {
            let _ = core_clone.run().await;
        });

        (
            Self {
                devices: Vec::new(),
                error_msg: None,
                core,
                event_rx: Arc::new(Mutex::new(Some(event_rx))),
                is_scanning: false,
                sim_status: "Disconnected".to_string(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("OpenFlite | MobiFlight for Linux")
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ScanDevices => {
                self.is_scanning = true;
                let core = self.core.clone();
                return Command::perform(
                    async move { core.scan_devices().map_err(|e| e.to_string()) },
                    Message::ScanResult,
                );
            }
            Message::ScanResult(result) => {
                self.is_scanning = false;
                match result {
                    Ok(_) => {
                        self.devices = self.core.get_devices();
                        self.error_msg = None;
                    }
                    Err(e) => {
                        self.error_msg = Some(format!("Scan failed: {}", e));
                    }
                }
            }
            Message::CoreEvent(event) => match event {
                Event::DeviceDetected(_) => {
                    self.devices = self.core.get_devices();
                }
                Event::SimConnected(status) => {
                    self.sim_status = status;
                }
                _ => {}
            },
            Message::ConnectSim => {
                self.sim_status = "Connecting...".to_string();
                let core = self.core.clone();
                return Command::perform(
                    async move {
                        let client = Box::new(openflite_connect::xplane::XPlaneClient::new(
                            "127.0.0.1:49000",
                        ));
                        core.set_sim_client(client).map_err(|e| e.to_string())
                    },
                    Message::SimResult,
                );
            }
            Message::SimResult(result) => {
                if let Err(e) = result {
                    self.sim_status = format!("Error: {}", e);
                }
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct CoreSubscription;
        let event_rx = self.event_rx.clone();
        iced::subscription::channel(
            std::any::TypeId::of::<CoreSubscription>(),
            100,
            move |mut output| async move {
                let rx = event_rx.lock().unwrap().take();
                if let Some(mut rx) = rx {
                    while let Some(event) = rx.recv().await {
                        let _ = output.try_send(Message::CoreEvent(event));
                    }
                }
                futures::future::pending::<()>().await;
                unreachable!()
            },
        )
    }

    fn view(&self) -> Element<'_, Message> {
        let is_sim_connected = self.sim_status == "Connected";

        // Header
        let header = container(
            row![
                text("OPENFLITE")
                    .size(30)
                    .style(Color::from_rgb(0.0, 0.8, 1.0)),
                horizontal_space().width(Length::Fill),
                text("SYSTEM STATUS: OK")
                    .size(14)
                    .style(Color::from_rgb(0.0, 1.0, 0.0)),
            ]
            .align_items(Alignment::Center)
            .padding(20),
        )
        .style(header_style);

        // Hardware Card
        let hardware_card = container(
            column![
                text("HARDWARE DASHBOARD")
                    .size(18)
                    .style(Color::from_rgb(0.7, 0.7, 0.7)),
                vertical_space().height(20),
                if self.is_scanning {
                    button(text("SCANNING...").size(14))
                        .padding(10)
                        .style(iced::theme::Button::Primary)
                } else {
                    button(text("SCAN FOR DEVICES").size(14))
                        .on_press(Message::ScanDevices)
                        .padding(10)
                        .style(iced::theme::Button::Primary)
                },
                vertical_space().height(20),
                scrollable(
                    column(
                        self.devices
                            .iter()
                            .map(|dev| {
                                row![
                                    container(horizontal_space().width(8))
                                        .width(8)
                                        .height(8)
                                        .style(|_t: &Theme| container::Appearance {
                                            background: Some(iced::Background::Color(
                                                Color::from_rgb(0.0, 1.0, 0.5)
                                            )),
                                            border: iced::Border {
                                                radius: 4.0.into(),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        }),
                                    horizontal_space().width(10),
                                    text(dev).size(16),
                                ]
                                .align_items(Alignment::Center)
                                .padding(5)
                                .into()
                            })
                            .collect::<Vec<_>>()
                    )
                    .spacing(5)
                )
                .height(Length::Fill),
            ]
            .padding(20),
        )
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(card_style);

        // Simulator Card
        let sim_card = container(
            column![
                text("SIMULATION BRIDGE")
                    .size(18)
                    .style(Color::from_rgb(0.7, 0.7, 0.7)),
                vertical_space().height(20),
                row![
                    text("STATUS: ").size(16),
                    text(&self.sim_status).size(16).style(if is_sim_connected {
                        Color::from_rgb(0.0, 1.0, 0.0)
                    } else if self.sim_status == "Connecting..." {
                        Color::from_rgb(1.0, 0.8, 0.0)
                    } else {
                        Color::from_rgb(1.0, 0.3, 0.3)
                    }),
                ],
                vertical_space().height(20),
                button(text("CONNECT TO X-PLANE").size(14))
                    .on_press(Message::ConnectSim)
                    .padding(10)
                    .style(if is_sim_connected {
                        iced::theme::Button::Secondary
                    } else {
                        iced::theme::Button::Primary
                    }),
                vertical_space().height(40),
                text("NETWORK SPECS")
                    .size(14)
                    .style(Color::from_rgb(0.4, 0.4, 0.4)),
                text("Local IP: 127.0.0.1")
                    .size(12)
                    .style(Color::from_rgb(0.4, 0.4, 0.4)),
                text("UDP Port: 49000")
                    .size(12)
                    .style(Color::from_rgb(0.4, 0.4, 0.4)),
            ]
            .padding(20),
        )
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .style(card_style);

        let main_content = container(
            row![hardware_card, horizontal_space().width(20), sim_card]
                .padding(20)
                .spacing(20),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        let footer = if let Some(err) = &self.error_msg {
            container(text(err).size(14).style(Color::from_rgb(1.0, 0.3, 0.3)))
                .padding(10)
                .width(Length::Fill)
                .style(footer_style)
        } else {
            container(vertical_space().height(0)).padding(0)
        };

        column![header, main_content, footer].into()
    }
}

fn header_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.05, 0.05, 0.07))),
        border: iced::Border {
            color: Color::from_rgb(0.1, 0.1, 0.15),
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn footer_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.05, 0.05))),
        ..Default::default()
    }
}

fn card_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.1))),
        border: iced::Border {
            color: Color::from_rgb(0.15, 0.15, 0.2),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
