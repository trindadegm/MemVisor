use std::sync::Arc;
use egui::{Response, Ui, Widget};
use crate::dap::dap_interface::DapInterface;

pub struct VarView {
    dap_interface: Arc<DapInterface>,
}

impl VarView {
    pub fn new(dap_interface: Arc<DapInterface>) -> Self {
        Self {
            dap_interface,
        }
    }
}
impl Widget for &mut VarView {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {

        });

        ui.response()
    }
}