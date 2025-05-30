use {
    super::TimerWindowState,
    crate::{
        controller::ProgressBarStyleChange, fl, render::TextFont, ControllerEvent,
        CONTROLLER_SENDER, SETTINGS,
    },
    nexus::imgui::{ComboBox, Condition, Selectable, Slider, TreeNode, TreeNodeFlags, Ui},
    strum::IntoEnumIterator,
};

pub struct ConfigTabState {
    pub katrender: bool,
}

impl ConfigTabState {
    pub fn new() -> Self {
        Self { katrender: false }
    }

    pub fn draw(&mut self, ui: &Ui, timer_window_state: &mut TimerWindowState) {
        if let Some(settings) = SETTINGS.get().and_then(|settings| settings.try_read().ok()) {
            self.katrender = settings.enable_katrender;
        };
        ui.text_wrapped(&fl!("imgui-notice"));
        ui.dummy([4.0, 4.0]);
        ui.text_wrapped(&fl!("keybind-triggers"));
        ui.dummy([4.0, 4.0]);
        #[cfg(feature = "space")]
        if ui.checkbox("Experimental KatRender", &mut self.katrender) {
            let sender = CONTROLLER_SENDER.get().unwrap();
            let event_send = sender.try_send(ControllerEvent::ToggleKatRender);
            drop(event_send);
        };
        let timers_window_closure = || {
            ui.dummy([4.0, 4.0]);
            if let Some(settings) = SETTINGS.get().and_then(|settings| settings.try_read().ok()) {
                timer_window_state.progress_bar.stock = settings.progress_bar.stock;
            };
            if ui.checkbox(
                &fl!("stock-imgui-progress-bar"),
                &mut timer_window_state.progress_bar.stock,
            ) {
                let sender = CONTROLLER_SENDER.get().unwrap();
                let event_send = sender.try_send(ControllerEvent::ProgressBarStyle(
                    ProgressBarStyleChange::Stock(timer_window_state.progress_bar.stock),
                ));
                drop(event_send);
            };
            if ui.checkbox(&fl!("shadow"), &mut timer_window_state.progress_bar.shadow) {
                let sender = CONTROLLER_SENDER.get().unwrap();
                let event_send = sender.try_send(ControllerEvent::ProgressBarStyle(
                    ProgressBarStyleChange::Shadow(timer_window_state.progress_bar.shadow),
                ));
                drop(event_send);
            }
            if ui.checkbox(
                &fl!("centre-text-after-icon"),
                &mut timer_window_state.progress_bar.centre_after,
            ) {
                let sender = CONTROLLER_SENDER.get().unwrap();
                let event_send = sender.try_send(ControllerEvent::ProgressBarStyle(
                    ProgressBarStyleChange::Centre(timer_window_state.progress_bar.centre_after),
                ));
                drop(event_send);
            }
            if Slider::new(&fl!("height"), 8.0, 256.0)
                .display_format("%.0f")
                .build(ui, &mut timer_window_state.progress_bar.height)
            {
                let sender = CONTROLLER_SENDER.get().unwrap();
                let event_send = sender.try_send(ControllerEvent::ProgressBarStyle(
                    ProgressBarStyleChange::Height(timer_window_state.progress_bar.height),
                ));
                drop(event_send);
            }
            let font_closure = || {
                let mut selected = timer_window_state.progress_bar.font.clone();
                for font in TextFont::iter() {
                    if Selectable::new(font.to_string())
                        .selected(font == selected)
                        .build(ui)
                    {
                        let sender = CONTROLLER_SENDER.get().unwrap();
                        let event_send = sender.try_send(ControllerEvent::ProgressBarStyle(
                            ProgressBarStyleChange::Font(font.clone()),
                        ));
                        selected = font;
                        drop(event_send);
                    }
                }
                selected
            };
            if let Some(_selection) = ComboBox::new(&fl!("font"))
                .preview_value(&timer_window_state.progress_bar.font.to_string())
                .build(ui, font_closure)
            {}
        };
        let _timers_window = TreeNode::new(&fl!("timer-window"))
            .flags(TreeNodeFlags::FRAMED)
            .opened(true, Condition::Once)
            .tree_push_on_open(true)
            .build(ui, timers_window_closure);
    }
}
