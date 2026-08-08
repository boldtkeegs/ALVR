use crate::dashboard::ServerRequest;
use alvr_common::ConnectionState;
use alvr_gui_common::theme::{self, log_colors};
use alvr_packets::ClientConnectionsAction;
use alvr_session::{ClientConnectionConfig, SessionConfig};
use alvr_sockets::WIRED_CLIENT_HOSTNAME;
use eframe::{
    egui::{self, Frame, Grid, Layout, ProgressBar, RichText, ScrollArea, TextEdit, Ui, Window},
    emath::{Align, Align2},
    epaint::Color32,
};

struct EditPopupState {
    new_devices: bool,
    hostname: String,
    ips: Vec<String>,
}

pub struct DevicesTab {
    new_devices: Option<Vec<(String, ClientConnectionConfig)>>,
    trusted_devices: Option<Vec<(String, ClientConnectionConfig)>>,
    edit_popup_state: Option<EditPopupState>,
    adb_download_progress: Option<f32>,
}

impl DevicesTab {
    pub fn new() -> Self {
        Self {
            new_devices: None,
            trusted_devices: None,
            edit_popup_state: None,
            adb_download_progress: None,
        }
    }

    pub fn update_client_list(&mut self, session: &SessionConfig) {
        let (trusted_clients, untrusted_clients) =
            session
                .client_connections
                .clone()
                .into_iter()
                .partition::<Vec<_>, _>(|(_, data)| data.trusted);

        self.trusted_devices = Some(trusted_clients);
        self.new_devices = Some(untrusted_clients);
    }

    pub fn update_adb_download_progress(&mut self, progress: f32) {
        self.adb_download_progress = Some(progress);
    }

    pub fn ui(&mut self, ui: &mut Ui, connected_to_server: bool) -> Vec<ServerRequest> {
        let mut requests = vec![];

        if self.new_devices.is_none() {
            requests.push(ServerRequest::GetSession);
        }

        if !connected_to_server {
            Frame::group(ui.style())
                .inner_margin(theme::FRAME_PADDING)
                .fill(log_colors::WARNING_LIGHT)
                .show(ui, |ui| {
                    Grid::new(0).num_columns(2).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(theme::FRAME_TEXT_SPACING);
                            ui.heading(
                                RichText::new(
                                    "ALVR requires running SteamVR! \
                                    Devices will not be discovered or connected.",
                                )
                                .color(Color32::BLACK)
                                .size(16.0),
                            );
                        });

                        #[cfg(not(target_arch = "wasm32"))]
                        ui.with_layout(Layout::right_to_left(eframe::emath::Align::Center), |ui| {
                            if ui.button("Launch SteamVR").clicked() {
                                crate::steamvr_launcher::LAUNCHER.lock().launch_steamvr();
                            }
                        });
                    })
                });
            ui.add_space(theme::FRAME_PADDING);
        }

        ui.vertical_centered_justified(|ui| {
            if let Some(clients) = &mut self.trusted_devices
                && let Some(request) = wired_client_section(
                    ui,
                    clients
                        .iter()
                        .find(|(hostname, _)| hostname == WIRED_CLIENT_HOSTNAME),
                    self.adb_download_progress,
                )
            {
                requests.push(request);
            }

            ui.add_space(theme::FRAME_PADDING);

            if let Some(clients) = &self.new_devices
                && let Some(request) = new_clients_section(ui, clients)
            {
                requests.push(request);
            }

            ui.add_space(theme::FRAME_PADDING);

            if let Some(clients) = &mut self.trusted_devices
                && let Some(request) = trusted_clients_section(
                    ui,
                    clients
                        .iter()
                        .filter(|(hostname, _)| hostname != WIRED_CLIENT_HOSTNAME)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &mut self.edit_popup_state,
                )
            {
                requests.push(request);
            }
        });

        if let Some(mut state) = self.edit_popup_state.take() {
            let panel_rect = ui.clip_rect();
            let offset_y = panel_rect.center().y - ui.ctx().content_rect().center().y;
            Window::new("Edit connection")
                .anchor(Align2::CENTER_CENTER, (0.0, offset_y))
                .resizable(false)
                .collapsible(false)
                .max_height(panel_rect.height() - theme::FRAME_PADDING * 2f32)
                .show(ui.ctx(), |ui| {
                    ui.add_space(theme::FRAME_TEXT_SPACING);
                    Grid::new("connection dialogue")
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Hostname:");
                            ui.add_enabled(
                                state.new_devices,
                                TextEdit::singleline(&mut state.hostname),
                            );
                            ui.end_row();

                            ui.label("IP Addresses:");
                            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                if !state.ips.is_empty() {
                                    ScrollArea::new([false, true])
                                        .auto_shrink([false, true])
                                        // The max height is set inflexible to UI changes, and still doesn't accomplish the visual goal of the panel height minus padding
                                        .max_height(
                                            panel_rect.height()
                                                - (ui.spacing().window_margin.top
                                                    + ui.spacing().window_margin.bottom)
                                                    as f32
                                                - theme::FRAME_TEXT_SPACING
                                                - theme::FRAME_PADDING * 2f32
                                                - (ui.spacing().interact_size.y
                                                    + ui.spacing().item_spacing.y)
                                                    * 3f32
                                                - (ui.text_style_height(&egui::TextStyle::Heading)
                                                    + ui.spacing().item_spacing.y * 2f32),
                                        )
                                        .show(ui, |ui| {
                                            let mut to_remove: Option<usize> = None;
                                            for (i, address) in state.ips.iter_mut().enumerate() {
                                                ui.horizontal(|ui| {
                                                    // Putting the remove button to the right of the textbox would look more appealing, but it causes strange alignment issues
                                                    if ui.button("❌").clicked() {
                                                        to_remove = Some(i);
                                                    }

                                                    ui.text_edit_singleline(address);
                                                });
                                            }
                                            if let Some(index) = to_remove {
                                                state.ips.remove(index);
                                            }
                                        });
                                }
                                if ui.button("Add new").clicked() {
                                    state.ips.push("192.168.X.X".into());
                                }
                            });
                            ui.end_row();
                        });

                    ui.columns(2, |ui| {
                        if ui[0].button("Cancel").clicked() {
                            return;
                        }

                        if ui[1].button("Save").clicked() {
                            let manual_ips =
                                state.ips.iter().filter_map(|s| s.parse().ok()).collect();

                            if state.new_devices {
                                requests.push(ServerRequest::UpdateClientList {
                                    hostname: state.hostname,
                                    action: ClientConnectionsAction::AddIfMissing {
                                        trusted: true,
                                        manual_ips,
                                    },
                                });
                            } else {
                                requests.push(ServerRequest::UpdateClientList {
                                    hostname: state.hostname,
                                    action: ClientConnectionsAction::SetManualIps(manual_ips),
                                });
                            }
                        } else {
                            self.edit_popup_state = Some(state);
                        }
                    })
                });
        }

        requests
    }
}

fn wired_client_section(
    ui: &mut Ui,
    maybe_client: Option<&(String, ClientConnectionConfig)>,
    adb_download_progress: Option<f32>,
) -> Option<ServerRequest> {
    let mut request = None;

    Frame::group(ui.style())
        .fill(theme::SECTION_BG)
        .inner_margin(egui::vec2(
            theme::FRAME_PADDING + theme::FRAME_TEXT_SPACING,
            theme::FRAME_PADDING,
        ))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                Grid::new("wired-client")
                    .num_columns(2)
                    .spacing(egui::vec2(8.0, 8.0))
                    .show(ui, |ui| {
                        ui.heading("Wired Connection");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let mut wired = maybe_client.is_some();

                            if alvr_gui_common::switch(ui, &mut wired).changed() {
                                if wired {
                                    request = Some(ServerRequest::UpdateClientList {
                                        hostname: WIRED_CLIENT_HOSTNAME.to_owned(),
                                        action: ClientConnectionsAction::AddIfMissing {
                                            trusted: true,
                                            manual_ips: Vec::new(),
                                        },
                                    });
                                } else {
                                    request = Some(ServerRequest::UpdateClientList {
                                        hostname: WIRED_CLIENT_HOSTNAME.to_owned(),
                                        action: ClientConnectionsAction::RemoveEntry,
                                    });
                                }
                            }
                            ui.horizontal(|ui| {
                                ui.add_space(theme::FRAME_TEXT_SPACING);
                            });
                        });
                        ui.end_row();

                        if let Some(progress) = adb_download_progress.filter(|p| *p < 1.0) {
                            ui.horizontal(|ui| {
                                ui.label("ADB download progress");
                            });
                            ui.horizontal(|ui| {
                                ui.add(ProgressBar::new(progress).animate(true).show_percentage());
                            });
                            ui.end_row();
                        } else if let Some((_, data)) = maybe_client {
                            ui.horizontal(|ui| {
                                ui.label(&data.display_name);
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                connection_label(ui, &data.connection_state);
                            });
                            ui.end_row();
                        }
                    });
            });
        });

    request
}

fn new_clients_section(
    ui: &mut Ui,
    clients: &[(String, ClientConnectionConfig)],
) -> Option<ServerRequest> {
    let mut request = None;

    Frame::group(ui.style())
        .inner_margin(theme::FRAME_PADDING)
        .fill(theme::SECTION_BG)
        .show(ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.add_space(theme::FRAME_TEXT_SPACING);
                    ui.heading("New Wireless Devices");

                    // Extend to the right
                    ui.with_layout(Layout::right_to_left(Align::Center), |_| ());
                });
            });
            if !clients.is_empty() {
                ScrollArea::new([false, true]).show(ui, |ui| {
                    for (hostname, _) in clients {
                        Frame::group(ui.style())
                            .fill(theme::DARKER_BG)
                            .inner_margin(egui::vec2(15.0, 12.0))
                            .show(ui, |ui| {
                                Grid::new(format!("{hostname}-new-clients"))
                                    .num_columns(2)
                                    .spacing(egui::vec2(8.0, 8.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(hostname);
                                        });
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if ui.button("Trust").clicked() {
                                                request = Some(ServerRequest::UpdateClientList {
                                                    hostname: hostname.clone(),
                                                    action: ClientConnectionsAction::Trust,
                                                });
                                            };
                                        });
                                        ui.end_row();
                                    });
                            });
                    }
                });
            }
        });

    request
}

fn trusted_clients_section(
    ui: &mut Ui,
    clients: &[&(String, ClientConnectionConfig)],
    edit_popup_state: &mut Option<EditPopupState>,
) -> Option<ServerRequest> {
    let mut request = None;

    Frame::group(ui.style())
        .fill(theme::SECTION_BG)
        .inner_margin(theme::FRAME_PADDING)
        .show(ui, |ui| {
            Grid::new(0).num_columns(2).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(theme::FRAME_TEXT_SPACING);
                    ui.heading("Trusted Wireless Devices");
                });

                ui.with_layout(Layout::right_to_left(eframe::emath::Align::Center), |ui| {
                    if ui.button("Add device manually").clicked() {
                        *edit_popup_state = Some(EditPopupState {
                            hostname: "XXXX.client.local.".into(),
                            new_devices: true,
                            ips: Vec::new(),
                        });
                    }
                });
            });
            if !clients.is_empty() {
                ScrollArea::new([false, true])
                    .show(ui, |ui| {
                        for (hostname, data) in clients {
                            Frame::group(ui.style())
                                .fill(theme::DARKER_BG)
                                .inner_margin(egui::vec2(15.0, 12.0))
                                .show(ui, |ui| {
                                    Grid::new(format!("{hostname}-clients"))
                                        .num_columns(2)
                                        .spacing(egui::vec2(8.0, 8.0))
                                        .show(ui, |ui| {
                                            ui.label(&data.display_name);
                                            ui.horizontal(|ui| {
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    connection_label(ui, &data.connection_state)
                                                });
                                            });

                                            ui.end_row();

                                            ui.label(format!(
                                                "{hostname}: {}",
                                                data.current_ip
                                                    .map_or_else(|| "Unknown IP".into(), |ip| ip.to_string()),
                                            ));
                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                if ui.button("Remove").clicked() {
                                                    request = Some(ServerRequest::UpdateClientList {
                                                        hostname: hostname.clone(),
                                                        action: ClientConnectionsAction::RemoveEntry,
                                                    });
                                                }
                                                if ui.button("Edit").clicked() {
                                                    *edit_popup_state = Some(EditPopupState {
                                                        new_devices: false,
                                                        hostname: hostname.to_owned(),
                                                        ips: data
                                                            .manual_ips
                                                            .iter()
                                                            .map(|addr| addr.to_string())
                                                            .collect::<Vec<String>>(),
                                                    });
                                                }
                                            });
                                        });
                                });
                        }
                    });
            }
        });

    request
}

fn connection_label(ui: &mut Ui, connection_state: &ConnectionState) {
    match connection_state {
        ConnectionState::Disconnected => ui.colored_label(Color32::GRAY, "Disconnected"),
        ConnectionState::Connecting => ui.colored_label(log_colors::WARNING_LIGHT, "Connecting"),
        ConnectionState::Connected => ui.colored_label(theme::OK_GREEN, "Connected"),
        ConnectionState::Streaming => ui.colored_label(theme::OK_GREEN, "Streaming"),
        ConnectionState::Disconnecting => {
            ui.colored_label(log_colors::WARNING_LIGHT, "Disconnecting")
        }
    };
}
