const TITLE_BAR_HEIGHT: f32 = 32.0;
const BACKGROUND_COLOR: [f32; 4] = [0., 0., 0., 0.];
const WINDOW_SIZE: [f32; 2] = [800., 700.];

pub struct DllData {
    socket: networking::Socket<shared::message::PayloadMessage, shared::message::ServerMessage>,

    // scan
    search_buffer: String,
    current_scan_info: Option<shared::data::ScanInfo>,

    // write
    write_addr: String,   // converted to usize  before being sent
    write_buffer: String, // conveted to Vec<u8> before being sent
}

#[derive(Default)]
pub struct Window {
    target_process_temp: String,
    dll_data: Option<DllData>,
    current_ui: crate::ui::tab::Tab,
}

impl eframe::App for Window {
    fn update(&mut self, egctx: &egui::Context, frame: &mut eframe::Frame) {
        egctx.request_repaint_after(std::time::Duration::from_millis(100)); // 10 updated /s
        self.listen_scanner();
        egui::CentralPanel::default()
            .frame(
                eframe::egui::Frame::none()
                    .fill(egctx.style().visuals.window_fill())
                    .rounding(10.0)
                    .stroke(egctx.style().visuals.widgets.noninteractive.fg_stroke)
                    .outer_margin(0.5),
            )
            .show(egctx, |ui| {
                let app_rect = ui.max_rect();

                // draw the title bar

                let title_bar_rect = {
                    let mut rect = app_rect;
                    rect.max.y = rect.min.y + TITLE_BAR_HEIGHT;
                    rect
                };
                self.render_title_bar(ui, egctx, title_bar_rect, "PIMS");
                const XPADDING: f32 = 10.;
                const YPADDING: f32 = 10.;

                let main_rect = {
                    let mut rect = app_rect;
                    rect.min.y = title_bar_rect.max.y;
                    rect.min.x += XPADDING;
                    rect.min.y += YPADDING;
                    rect.max.y = app_rect.max.y / 2. + TITLE_BAR_HEIGHT / 2.;
                    rect
                };

                self.main_ui(ui, main_rect, egctx);
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        BACKGROUND_COLOR
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(data) = &mut self.dll_data {
            if let Err(e) = data.socket.send(shared::message::ServerMessage::Eject) {
                error!("An error occured while requesting the ejection of the scanner dll: {e}");
            } else {
                info!("Succesfully ejected scanner dll");
            }
        }
    }
}

impl Window {
    fn listen_scanner(&mut self) {
        let Some(data) = &mut self.dll_data else {
            return;
        };

        while let Ok((_header, msg)) = data.socket.try_recv() {
            match msg {
                shared::message::PayloadMessage::Boot => info!("Scanner has succesfully booted"),
                shared::message::PayloadMessage::Info(txt) => info!("Scanner said: {txt}"),
                shared::message::PayloadMessage::ScanUpdate(scaninfo) => {
                    info!("Scan info update: {scaninfo:#?}");
                    data.current_scan_info = Some(scaninfo);
                }
                shared::message::PayloadMessage::Exit => todo!(),
                _ => (),
            }
        }
    }

    pub fn main_ui(&mut self, ui: &mut egui::Ui, main_rect: egui::Rect, egctx: &egui::Context) {
        ui.horizontal(|ui| {
            if ui.button("Select").clicked() {
                self.current_ui = crate::ui::tab::Tab::TargetSelect;
            }

            if ui.button("Scan").clicked() {
                self.current_ui = crate::ui::tab::Tab::Scan;
            }

            if ui.button("Write").clicked() {
                self.current_ui = crate::ui::tab::Tab::Write;
            }
        });

        match self.current_ui {
            crate::ui::tab::Tab::TargetSelect => self.target_select_ui(ui),
            crate::ui::tab::Tab::Scan => {
                self.scan_ui(ui);
                self.draw_scan_result(ui, egctx);
            }
            crate::ui::tab::Tab::Write => self.write_ui(ui),
        }

        // let scan_rect = ui
        //     .allocate_ui_at_rect(
        //         {
        //             let mut rect = main_rect;
        //             rect.min.x = main_rect.min.x;
        //             rect.min.y = main_rect.max.y + 10.;
        //             rect.max.x = rect.min.x + 500.;
        //             rect.max.y = rect.min.y + 500.;
        //             rect
        //         },
        //         |ui| self.scan_ui(ui),
        //     )
        //     .response
        //     .rect;

        // let scan_res_rect = ui
        //     .allocate_ui_at_rect(
        //         {
        //             let mut rect = scan_rect;
        //             rect.min.x = rect.min.x;
        //             rect.min.y = rect.max.y + 10.;
        //             rect.max.x = rect.min.x + 500.;
        //             rect.max.y = rect.min.y + 500.;
        //             rect
        //         },
        //         |ui| self.draw_scan_result(ui, egctx),
        //     )
        //     .response
        //     .rect;

        // self.write_ui(ui);
    }

    pub fn target_select_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Target process: ");
            ui.text_edit_singleline(&mut self.target_process_temp);
            ui.add_space(5.);
            if ui.button("Open process").clicked() {
                if self.dll_data.is_some() {
                    info!("Dll data is not none, another process must have already been injected");
                    let socket = &mut self.dll_data.as_mut().unwrap().socket;
                    socket.send(shared::message::ServerMessage::Eject).unwrap();

                    drop(self.dll_data.take().unwrap());
                };

                let listener = std::net::TcpListener::bind(shared::DEFAULT_ADDRESS).unwrap(); // trust :D

                if !crate::injection::inject(
                    crate::injection::DEFAULT_DLL_PATH,
                    &self.target_process_temp,
                ) {
                    error!("Injection failled, given process doens't exists");
                    return;
                }

                let (stream, _addr) = listener.accept().unwrap(); // This will hang until the dll has connected..
                stream.set_nonblocking(true).unwrap();

                let data = DllData {
                    socket: networking::Socket::new(stream),

                    search_buffer: String::new(),
                    current_scan_info: None,

                    write_addr: String::new(),
                    write_buffer: String::new(),
                };

                self.dll_data = Some(data);
            }
        });
    }

    pub fn write_ui(&mut self, ui: &mut egui::Ui) {
        let Some(data) = &mut self.dll_data else {
            return;
        };

        let Some(scan_info) = &mut data.current_scan_info else {
            return;
        };

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Target addr: ");
                ui.text_edit_singleline(&mut data.write_addr);
            });
            ui.horizontal(|ui| {
                ui.label("Patch bytes: ");
                ui.text_edit_singleline(&mut data.write_buffer);
            });

            if ui.button("Patch").clicked() {
                let Ok(addr) = u64::from_str_radix(&data.write_addr.trim_start_matches("0x"), 16)
                else {
                    error!("Could not understand {}", data.write_addr);
                    return;
                };
                info!("Parsed addr: {}", format!("0x{addr:x}"));

                let patch_bytes = data
                    .write_buffer
                    .split(" ")
                    .filter(|c| !c.is_empty())
                    .map(|c| {
                        info!("Parsing: '{c}'", c = c.trim_start_matches("0x"));
                        u8::from_str_radix(c.trim_start_matches("0x"), 16).ok()
                    })
                    .collect::<Vec<Option<u8>>>();

                if patch_bytes.iter().any(|el| (*el).is_none()) {
                    error!("Could not parse given payload, please fix");
                    return;
                }

                let patch_bytes = patch_bytes
                    .iter()
                    .flatten()
                    .map(|el| *el)
                    .collect::<Vec<u8>>();

                for addr in scan_info.found_addresses.iter() {
                    data.socket
                        .send(shared::message::ServerMessage::WriteRequest {
                            addr: *addr as usize,
                            // This could be REALLY BAD if the user patch the appication with to few or too much bytes
                            // lmao
                            data: patch_bytes.clone(),
                        })
                        .unwrap();
                }
            }
        });
    }

    pub fn scan_ui(&mut self, ui: &mut egui::Ui) {
        let Some(data) = &mut self.dll_data else {
            return;
        };

        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                ui.label("Search bytes: ");
                ui.text_edit_singleline(&mut data.search_buffer);
            });

            if ui.button("Request basic scan").clicked() {
                let target_bytes = data
                    .search_buffer
                    .split(" ")
                    .filter(|c| !c.is_empty())
                    .map(|c| {
                        info!("Parsing: '{c}'", c = c.trim_start_matches("0x"));
                        u8::from_str_radix(c.trim_start_matches("0x"), 16).ok()
                    })
                    .collect::<Vec<Option<u8>>>();

                if target_bytes.iter().any(|el| (*el).is_none()) {
                    error!("Could not parse given payload, please fix");
                    return;
                }

                let target_bytes = target_bytes
                    .iter()
                    .flatten()
                    .map(|el| *el)
                    .collect::<Vec<u8>>();

                data.socket
                    .send(shared::message::ServerMessage::ScanRequest(
                        shared::data::ScanParams {
                            value: target_bytes,
                            start_addr: None,
                            end_addr: None,
                        },
                    ))
                    .unwrap();
            }
        });
    }

    pub fn draw_scan_result(&mut self, ui: &mut egui::Ui, egctx: &egui::Context) {
        let Some(data) = &mut self.dll_data else {
            return;
        };

        let Some(scan_info) = &mut data.current_scan_info else {
            return;
        };

        // let title_size = 100;

        // ui.put(egui::Rect::from_min_size(egui::pos2(0., 0.), egui::vec2(100., 100.)), );
        ui.label(format!(
            "Found {} addreses: ",
            scan_info.found_addresses.len()
        ));
        for addr in &scan_info.found_addresses {
            ui.label(format!("0x{addr:x}")); // Nice :D
        }
    }

    fn draw_pop_up(
        egctx: &egui::Context,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        add_contents: impl Fn(&mut egui::Ui),
    ) {
        ui.with_layer_id(
            egui::LayerId::new(egui::Order::Foreground, egui::Id::new(1)),
            |ui| {
                let (_response, painter) = ui.allocate_painter(rect.size(), egui::Sense::click());
                painter.add(egui::Shape::rect_filled(
                    rect,
                    0.0,
                    egui::Color32::from_rgb(0, 0, 0),
                ));

                ui.allocate_ui_at_rect(rect, |ui| centered(ui, add_contents))
            },
        );
    }
}

impl Window {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            target_process_temp: String::new(),
            dll_data: None,
            current_ui: crate::ui::tab::Tab::default(),
        }
    }
    fn render_title_bar(
        &mut self,
        ui: &mut egui::Ui,
        egctx: &egui::Context,
        title_bar_rect: eframe::epaint::Rect,
        title: &str,
    ) {
        let painter = ui.painter();

        let title_bar_response = ui.interact(
            title_bar_rect,
            egui::Id::new("title_bar"),
            egui::Sense::click(),
        );

        // Paint the title:
        painter.text(
            title_bar_rect.center(),
            eframe::emath::Align2::CENTER_CENTER,
            title,
            eframe::epaint::FontId::proportional(20.0),
            ui.style().visuals.text_color(),
        );

        // Paint the line under the title:
        painter.line_segment(
            [
                title_bar_rect.left_bottom() + eframe::epaint::vec2(1.0, 0.0),
                title_bar_rect.right_bottom() + eframe::epaint::vec2(-1.0, 0.0),
            ],
            ui.visuals().widgets.noninteractive.bg_stroke,
        );

        // Interact with the title bar (drag to move window):
        if title_bar_response.double_clicked() {
            // frame.set_maximized(!frame.info().window_info.maximized);
        } else if title_bar_response.is_pointer_button_down_on() {
            egctx.send_viewport_cmd(egui::viewport::ViewportCommand::StartDrag);
            // frame.drag_window();
        }

        // Show toggle button for light/dark mode
        ui.allocate_ui_at_rect(title_bar_rect, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.visuals_mut().button_frame = false;
                ui.add_space(8.0);
                egui::widgets::global_dark_light_mode_switch(ui);
            });
        });

        // Show some close/maximize/minimize buttons for the native window.
        ui.allocate_ui_at_rect(title_bar_rect, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.visuals_mut().button_frame = false;
                ui.add_space(8.0);

                let button_height = 12.0;

                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("❌").size(button_height),
                    ))
                    .on_hover_text("Close the window")
                    .clicked()
                {
                    egctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close);
                }

                let (hover_text, clicked_state) =
                    if ui.input(|i| i.viewport().maximized) == Some(true) {
                        ("Restore window", false)
                    } else {
                        ("Maximize window", true)
                    };

                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("🗗").size(button_height),
                    ))
                    .on_hover_text(hover_text)
                    .clicked()
                {
                    if clicked_state {
                        egctx.send_viewport_cmd(egui::viewport::ViewportCommand::Maximized(true));
                    } else {
                        egctx.send_viewport_cmd(egui::viewport::ViewportCommand::Maximized(false));
                    }
                }

                if ui
                    .add(egui::Button::new(
                        egui::RichText::new("🗕").size(button_height),
                    ))
                    .on_hover_text("Minimize the window")
                    .clicked()
                {
                    egctx.send_viewport_cmd(egui::viewport::ViewportCommand::Minimized(true));
                }
            });
        });
    }
}

pub fn centered(ui: &mut eframe::egui::Ui, add_contents: impl FnOnce(&mut eframe::egui::Ui)) {
    ui.horizontal(|ui| {
        let id = ui.id().with("_centerer");
        let last_width: Option<f32> = ui.memory_mut(|mem| mem.data.get_temp(id));
        if let Some(last_width) = last_width {
            ui.add_space(ui.max_rect().width() / 2.0 - last_width);
        }
        let res = ui.scope(|ui| ui.vertical(|ui| add_contents(ui))).response;
        let width = res.rect.width();
        ui.memory_mut(|mem| mem.data.insert_temp(id, width));

        // Repaint if width changed
        match last_width {
            None => ui.ctx().request_repaint(),
            Some(last_width) if last_width != width => ui.ctx().request_repaint(),
            Some(_) => {}
        }
    });
}

pub fn run() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_title("PIMS")
            .with_inner_size(WINDOW_SIZE)
            .with_resizable(true),
        // decorations: Some(false),
        // title: "PIMS",
        // inner_size: Some([500., 400.].into()),
        // resizable: true,
        // icon: todo!(),
        // active: todo!(),
        // visible: todo!(),
        // drag_and_drop: todo!(),
        // fullsize_content_view: todo!(),
        // title_shown: todo!(),
        // titlebar_buttons_shown: todo!(),
        // titlebar_shown: todo!(),
        // close_button: todo!(),
        // minimize_button: todo!(),
        // maximize_button: todo!(),
        // window_level: todo!(),
        // mouse_passthrough: todo!(),
        follow_system_theme: true,
        run_and_return: true,
        centered: true,
        ..Default::default()
    };
    eframe::run_native("PIMS", options, Box::new(|cc| Box::new(Window::new(cc)))).unwrap();
}
