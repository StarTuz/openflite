use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, scrollable, text, text_input,
    vertical_space,
};
use iced::{
    executor, Alignment, Application, Color, Command, Element, Length, Settings, Subscription,
    Theme,
};
use openflite_core::{Core, Event};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

mod styles;

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
    data_cache: HashMap<String, f64>,
    config_loaded: bool,
    // Config Editor State
    show_editor: bool,
    editor: EditorState,
    output_mappings: Vec<OutputMappingDraft>,
    input_mappings: Vec<InputMappingDraft>,
}

#[derive(Debug, Clone, Default)]
struct EditorState {
    dataref: String,
    comparison_op: Option<String>,
    comparison_value: String,
    if_value: String,
    else_value: String,
    target_device: Option<String>,
    target_pin: String,
    display_type: Option<String>,
}

#[derive(Debug, Clone)]
struct OutputMappingDraft {
    dataref: String,
    comparison_op: String,
    comparison_value: String,
    if_value: String,
    else_value: String,
    device: String,
    pin: String,
    display_type: String,
}

#[derive(Debug, Clone)]
struct InputMappingDraft {
    name: String,
    input_type: String, // "Button" or "Encoder"
    on_press_cmd: String,
    on_left_cmd: String,
    on_right_cmd: String,
}

#[derive(Debug, Clone)]
enum Message {
    ScanDevices,
    ScanResult(Result<(), String>),
    ConnectSim,
    DisconnectSim,
    SimResult(Result<(), String>),
    ConnectDemo,
    LoadDemoConfig,
    TriggerDemoButton,
    TriggerEncoderLeft,
    TriggerEncoderRight,
    CoreEvent(Event),
    Tick,
    // Config Editor Messages
    ToggleEditor,
    EditorDatarefChanged(String),
    EditorComparisonOpSelected(String),
    EditorComparisonValueChanged(String),
    EditorIfValueChanged(String),
    EditorElseValueChanged(String),
    EditorDeviceSelected(String),
    EditorPinChanged(String),
    EditorDisplayTypeSelected(String),
    AddOutputMapping,
    ApplyMappings,
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
                data_cache: HashMap::new(),
                config_loaded: false,
                show_editor: false,
                editor: EditorState::default(),
                output_mappings: Vec::new(),
                input_mappings: Vec::new(),
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
                Event::SimDisconnected => {
                    self.sim_status = "Disconnected".to_string();
                    self.data_cache.clear();
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
                        let res = core.set_sim_client(client).map_err(|e| e.to_string());
                        if res.is_ok() {
                            core.broadcast(Event::SimConnected("Connected".to_string()));
                        }
                        res
                    },
                    Message::SimResult,
                );
            }
            Message::DisconnectSim => {
                self.core.disconnect_sim();
            }
            Message::SimResult(result) => {
                if let Err(e) = result {
                    self.sim_status = format!("Error: {}", e);
                }
            }
            Message::ConnectDemo => {
                self.sim_status = "Demo Mode".to_string();
                let core = self.core.clone();
                return Command::perform(
                    async move {
                        let client = Box::new(openflite_connect::dummy::DummyClient::new());
                        let res = core.set_sim_client(client).map_err(|e| e.to_string());
                        if res.is_ok() {
                            core.broadcast(Event::SimConnected("Demo Mode".to_string()));
                        }
                        res
                    },
                    Message::SimResult,
                );
            }
            Message::LoadDemoConfig => {
                let xml = r#"
                    <MobiFlightProject>
                        <Outputs>
                            <Config guid="demo-altitude" active="true">
                                <Description>Altitude LED</Description>
                                <Settings>
                                    <Source type="SimConnect" name="sim/flightmodel/position/altitude" />
                                    <Comparison active="true" value="1050" operand=">" ifValue="1" elseValue="0" />
                                    <Display type="Pin" serial="DEMO-BOARD" trigger="OnChange" pin="13" />
                                </Settings>
                            </Config>
                        </Outputs>
                        <Inputs>
                            <Config guid="demo-gear" active="true">
                                <Description>GearToggle</Description>
                                <Settings>
                                    <Button>
                                        <OnPress type="XplaneAction" cmd="sim/annunciator/gear_unsafe" />
                                    </Button>
                                </Settings>
                            </Config>
                            <Config guid="demo-heading" active="true">
                                <Description>HeadingDial</Description>
                                <Settings>
                                    <Encoder>
                                        <OnLeft type="XplaneAction" cmd="sim/autopilot/heading_down" />
                                        <OnRight type="XplaneAction" cmd="sim/autopilot/heading_up" />
                                    </Encoder>
                                </Settings>
                            </Config>
                        </Inputs>
                    </MobiFlightProject>
                "#;
                if self.core.load_config(xml).is_ok() {
                    self.config_loaded = true;
                    self.error_msg = None;
                } else {
                    self.error_msg = Some("Failed to load demo config".to_string());
                }
            }
            Message::TriggerDemoButton => {
                use openflite_core::protocol::Response;
                self.core.inject_hardware_response(
                    "DEMO-BOARD",
                    Response::InputEvent {
                        name: "GearToggle".to_string(),
                        value: "1".to_string(),
                    },
                );
            }
            Message::TriggerEncoderLeft => {
                use openflite_core::protocol::Response;
                self.core.inject_hardware_response(
                    "DEMO-BOARD",
                    Response::InputEvent {
                        name: "HeadingDial".to_string(),
                        value: "0".to_string(),
                    },
                );
            }
            Message::TriggerEncoderRight => {
                use openflite_core::protocol::Response;
                self.core.inject_hardware_response(
                    "DEMO-BOARD",
                    Response::InputEvent {
                        name: "HeadingDial".to_string(),
                        value: "1".to_string(),
                    },
                );
            }
            Message::Tick => {
                self.data_cache = self.core.get_all_variables();
            }
            // Config Editor Message Handlers
            Message::ToggleEditor => {
                self.show_editor = !self.show_editor;
            }
            Message::EditorDatarefChanged(val) => {
                self.editor.dataref = val;
            }
            Message::EditorComparisonOpSelected(val) => {
                self.editor.comparison_op = Some(val);
            }
            Message::EditorComparisonValueChanged(val) => {
                self.editor.comparison_value = val;
            }
            Message::EditorIfValueChanged(val) => {
                self.editor.if_value = val;
            }
            Message::EditorElseValueChanged(val) => {
                self.editor.else_value = val;
            }
            Message::EditorDeviceSelected(val) => {
                self.editor.target_device = Some(val);
            }
            Message::EditorPinChanged(val) => {
                self.editor.target_pin = val;
            }
            Message::EditorDisplayTypeSelected(val) => {
                self.editor.display_type = Some(val);
            }
            Message::AddOutputMapping => {
                if !self.editor.dataref.is_empty() {
                    self.output_mappings.push(OutputMappingDraft {
                        dataref: self.editor.dataref.clone(),
                        comparison_op: self.editor.comparison_op.clone().unwrap_or_default(),
                        comparison_value: self.editor.comparison_value.clone(),
                        if_value: self.editor.if_value.clone(),
                        else_value: self.editor.else_value.clone(),
                        device: self.editor.target_device.clone().unwrap_or_default(),
                        pin: self.editor.target_pin.clone(),
                        display_type: self
                            .editor
                            .display_type
                            .clone()
                            .unwrap_or("Pin".to_string()),
                    });
                    self.editor = EditorState::default();
                }
            }
            Message::ApplyMappings => {
                let xml = self.generate_config_xml();
                if self.core.load_config(&xml).is_ok() {
                    self.config_loaded = true;
                    self.error_msg = None;
                } else {
                    self.error_msg = Some("Failed to apply config".to_string());
                }
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct CoreSubscription;
        let event_rx = self.event_rx.clone();
        let events = iced::subscription::channel(
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
        );

        let tick = iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::Tick);

        Subscription::batch(vec![events, tick])
    }

    fn view(&self) -> Element<'_, Message> {
        let is_sim_connected = self.sim_status == "Connected";
        let is_demo_mode = self.sim_status == "Demo Mode";

        column![
            self.view_header(),
            self.view_main_content(is_sim_connected, is_demo_mode),
            if self.show_editor {
                self.view_editor_panel()
            } else {
                vertical_space().height(0).into()
            },
            self.view_footer()
        ]
        .into()
    }
}

impl OpenFliteApp {
    fn view_header(&self) -> Element<'_, Message> {
        container(
            row![
                text("OPENFLITE")
                    .size(30)
                    .style(Color::from_rgb(0.0, 0.8, 1.0)),
                horizontal_space().width(Length::Fill),
                button(
                    text(if self.show_editor {
                        "CLOSE EDITOR"
                    } else {
                        "CONFIG EDITOR"
                    })
                    .size(12)
                )
                .on_press(Message::ToggleEditor)
                .padding(8)
                .style(if self.show_editor {
                    iced::theme::Button::Secondary
                } else {
                    iced::theme::Button::Primary
                }),
                horizontal_space().width(15),
                text("SYSTEM STATUS: OK")
                    .size(14)
                    .style(Color::from_rgb(0.0, 1.0, 0.0)),
            ]
            .align_items(Alignment::Center)
            .padding(20),
        )
        .style(styles::header_style)
        .into()
    }

    fn view_footer(&self) -> Element<'_, Message> {
        if let Some(err) = &self.error_msg {
            container(text(err).size(14).style(Color::from_rgb(1.0, 0.3, 0.3)))
                .padding(10)
                .width(Length::Fill)
                .style(styles::footer_style)
                .into()
        } else {
            vertical_space().height(0).into()
        }
    }

    fn view_main_content(
        &self,
        is_sim_connected: bool,
        is_demo_mode: bool,
    ) -> Element<'_, Message> {
        container(
            column![
                row![
                    self.view_hardware_card(),
                    horizontal_space().width(20),
                    self.view_sim_card(is_sim_connected, is_demo_mode)
                ]
                .height(Length::FillPortion(1)),
                vertical_space().height(20),
                self.view_data_card(),
            ]
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_hardware_card(&self) -> Element<'_, Message> {
        container(
            column![
                text("HARDWARE DASHBOARD")
                    .size(18)
                    .style(Color::from_rgb(0.7, 0.7, 0.7)),
                vertical_space().height(20),
                row![
                    button(text("SCAN FOR DEVICES").size(14))
                        .on_press(Message::ScanDevices)
                        .padding(10)
                        .style(iced::theme::Button::Primary),
                    horizontal_space().width(10),
                    button(
                        text(if self.config_loaded {
                            "CONFIG LOADED"
                        } else {
                            "LOAD DEMO LOGIC"
                        })
                        .size(14)
                    )
                    .on_press(Message::LoadDemoConfig)
                    .padding(10)
                    .style(if self.config_loaded {
                        iced::theme::Button::Secondary
                    } else {
                        iced::theme::Button::Primary
                    }),
                ],
                if self.config_loaded {
                    Element::from(
                        column![
                            vertical_space().height(10),
                            button(text("TRIGGER GEAR BUTTON").size(14))
                                .on_press(Message::TriggerDemoButton)
                                .padding(10)
                                .style(iced::theme::Button::Destructive),
                            row![
                                button(text("ENCODER L").size(12))
                                    .on_press(Message::TriggerEncoderLeft)
                                    .padding(8)
                                    .style(iced::theme::Button::Secondary),
                                horizontal_space().width(5),
                                button(text("ENCODER R").size(12))
                                    .on_press(Message::TriggerEncoderRight)
                                    .padding(8)
                                    .style(iced::theme::Button::Secondary),
                            ]
                        ]
                        .spacing(10),
                    )
                } else {
                    vertical_space().height(0).into()
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
        .style(styles::card_style)
        .into()
    }

    fn view_sim_card(&self, is_sim_connected: bool, is_demo_mode: bool) -> Element<'_, Message> {
        let is_any_connected = is_sim_connected || is_demo_mode;
        container(
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
                if is_sim_connected {
                    button(text("DISCONNECT FROM X-PLANE").size(14))
                        .on_press(Message::DisconnectSim)
                        .padding(10)
                        .style(iced::theme::Button::Secondary)
                } else if !is_any_connected {
                    button(text("CONNECT TO X-PLANE").size(14))
                        .on_press(Message::ConnectSim)
                        .padding(10)
                        .style(iced::theme::Button::Primary)
                } else {
                    button(text("SIM DISCONNECTED").size(14))
                        .padding(10)
                        .style(iced::theme::Button::Secondary)
                },
                vertical_space().height(10),
                if is_demo_mode {
                    button(text("STOP DEMO MODE").size(14))
                        .on_press(Message::DisconnectSim)
                        .padding(10)
                        .style(iced::theme::Button::Secondary)
                } else if !is_any_connected {
                    button(text("START DEMO MODE").size(14))
                        .on_press(Message::ConnectDemo)
                        .padding(10)
                        .style(iced::theme::Button::Secondary)
                } else {
                    button(text("DEMO INACTIVE").size(14))
                        .padding(10)
                        .style(iced::theme::Button::Secondary)
                },
                vertical_space().height(30),
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
        .style(styles::card_style)
        .into()
    }

    fn view_data_card(&self) -> Element<'_, Message> {
        container(
            column![
                text("LIVE DATA MONITOR")
                    .size(18)
                    .style(Color::from_rgb(0.7, 0.7, 0.7)),
                vertical_space().height(20),
                scrollable(
                    column({
                        let mut data: Vec<_> = self.data_cache.iter().collect();
                        data.sort_by(|a, b| a.0.cmp(b.0));
                        data.into_iter()
                            .map(|(name, value)| {
                                row![
                                    text(name).size(14).style(Color::from_rgb(0.5, 0.5, 0.5)),
                                    horizontal_space().width(Length::Fill),
                                    text(format!("{:.4}", value))
                                        .size(14)
                                        .style(Color::from_rgb(0.0, 1.0, 0.8)),
                                ]
                                .padding(2)
                                .into()
                            })
                            .collect::<Vec<_>>()
                    })
                    .spacing(2)
                )
                .height(Length::Fill),
            ]
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::FillPortion(1))
        .style(styles::card_style)
        .into()
    }

    fn generate_config_xml(&self) -> String {
        let mut outputs_xml = String::new();
        for (i, m) in self.output_mappings.iter().enumerate() {
            outputs_xml.push_str(&format!(
                r#"<Config guid="user-{}" active="true">
                    <Description>{}</Description>
                    <Settings>
                        <Source type="SimConnect" name="{}" />
                        <Comparison active="true" value="{}" operand="{}" ifValue="{}" elseValue="{}" />
                        <Display type="{}" serial="{}" trigger="OnChange" pin="{}" />
                    </Settings>
                </Config>"#,
                i, m.dataref, m.dataref, m.comparison_value, m.comparison_op, m.if_value, m.else_value,
                m.display_type, m.device, m.pin
            ));
        }

        format!(
            r#"<MobiFlightProject>
                <Outputs>{}</Outputs>
                <Inputs></Inputs>
            </MobiFlightProject>"#,
            outputs_xml
        )
    }

    fn view_editor_panel(&self) -> Element<'_, Message> {
        let comparison_ops: Vec<String> =
            vec![">".into(), "<".into(), "=".into(), ">=".into(), "<=".into()];
        let display_types: Vec<String> = vec!["Pin".into(), "7Segment".into(), "LCD".into()];

        container(
            column![
                row![
                    text("CONFIG EDITOR")
                        .size(18)
                        .style(Color::from_rgb(0.7, 0.7, 0.7)),
                    horizontal_space().width(Length::Fill),
                    button(text("X").size(14))
                        .on_press(Message::ToggleEditor)
                        .padding(5)
                        .style(iced::theme::Button::Secondary),
                ],
                vertical_space().height(15),
                text("Output Mapping")
                    .size(14)
                    .style(Color::from_rgb(0.5, 0.5, 0.5)),
                vertical_space().height(10),
                row![
                    text("Dataref:").size(12),
                    horizontal_space().width(10),
                    text_input("sim/flightmodel/...", &self.editor.dataref)
                        .on_input(Message::EditorDatarefChanged)
                        .padding(5)
                        .width(Length::Fill),
                ]
                .align_items(Alignment::Center),
                vertical_space().height(10),
                row![
                    text("If value").size(12),
                    horizontal_space().width(5),
                    pick_list(
                        comparison_ops.clone(),
                        self.editor.comparison_op.clone(),
                        Message::EditorComparisonOpSelected
                    )
                    .placeholder("op"),
                    horizontal_space().width(5),
                    text_input("threshold", &self.editor.comparison_value)
                        .on_input(Message::EditorComparisonValueChanged)
                        .padding(5)
                        .width(60),
                    horizontal_space().width(5),
                    text("then").size(12),
                    horizontal_space().width(5),
                    text_input("1", &self.editor.if_value)
                        .on_input(Message::EditorIfValueChanged)
                        .padding(5)
                        .width(40),
                    horizontal_space().width(5),
                    text("else").size(12),
                    horizontal_space().width(5),
                    text_input("0", &self.editor.else_value)
                        .on_input(Message::EditorElseValueChanged)
                        .padding(5)
                        .width(40),
                ]
                .align_items(Alignment::Center),
                vertical_space().height(10),
                row![
                    text("Device:").size(12),
                    horizontal_space().width(5),
                    pick_list(
                        self.devices.clone(),
                        self.editor.target_device.clone(),
                        Message::EditorDeviceSelected
                    )
                    .placeholder("Select device"),
                    horizontal_space().width(10),
                    text("Pin:").size(12),
                    horizontal_space().width(5),
                    text_input("13", &self.editor.target_pin)
                        .on_input(Message::EditorPinChanged)
                        .padding(5)
                        .width(50),
                    horizontal_space().width(10),
                    text("Type:").size(12),
                    horizontal_space().width(5),
                    pick_list(
                        display_types,
                        self.editor.display_type.clone(),
                        Message::EditorDisplayTypeSelected
                    )
                    .placeholder("Pin"),
                ]
                .align_items(Alignment::Center),
                vertical_space().height(15),
                row![
                    button(text("ADD MAPPING").size(12))
                        .on_press(Message::AddOutputMapping)
                        .padding(8)
                        .style(iced::theme::Button::Primary),
                    horizontal_space().width(10),
                    button(text("APPLY ALL").size(12))
                        .on_press(Message::ApplyMappings)
                        .padding(8)
                        .style(iced::theme::Button::Positive),
                ],
                vertical_space().height(15),
                text(format!("Mappings: {}", self.output_mappings.len()))
                    .size(12)
                    .style(Color::from_rgb(0.4, 0.4, 0.4)),
            ]
            .padding(20),
        )
        .width(Length::Fill)
        .style(styles::card_style)
        .into()
    }
}
