use iced::widget::{button, column, container, row, text, vertical_space};
use iced::{
    executor, Alignment, Application, Command, Element, Length, Settings, Subscription, Theme,
};
use openflite_core::{Core, Event};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub fn main() -> iced::Result {
    env_logger::init();
    OpenFliteApp::run(Settings::default())
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

        // Spawn core loop
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
        String::from("OpenFlite")
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
            Message::CoreEvent(event) => {
                log::info!("Received Core Event: {:?}", event);
                match event {
                    Event::DeviceDetected(_) => {
                        self.devices = self.core.get_devices();
                    }
                    Event::SimConnected(status) => {
                        self.sim_status = status;
                    }
                    _ => {}
                }
            }
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
        let title = text("OpenFlite").size(50);
        let subtitle = text("MobiFlight for Linux").size(20);

        let scan_btn = if self.is_scanning {
            button("Scanning...").padding(10)
        } else {
            button("Scan for Devices")
                .on_press(Message::ScanDevices)
                .padding(10)
        };

        let mut devices_list = column![text("Connected Devices:").size(24)].spacing(10);
        if self.devices.is_empty() {
            devices_list = devices_list.push(text("No devices found."));
        } else {
            for dev in &self.devices {
                devices_list = devices_list
                    .push(row![text("• "), text(dev).size(18),].align_items(Alignment::Center));
            }
        }

        let mut content = column![
            title,
            subtitle,
            vertical_space().height(20.0),
            scan_btn,
            vertical_space().height(20.0),
            devices_list,
            vertical_space().height(40.0),
            text(format!("Simulator: {}", self.sim_status)).size(24),
            button("Connect to X-Plane")
                .on_press(Message::ConnectSim)
                .padding(10),
        ]
        .spacing(10)
        .padding(20)
        .align_items(Alignment::Center);

        if let Some(err) = &self.error_msg {
            content = content.push(
                text(err)
                    .style(iced::Color::from_rgb(1.0, 0.0, 0.0))
                    .size(16),
            );
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
