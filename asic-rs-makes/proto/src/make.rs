use std::{fmt::Display, str::FromStr};

use asic_rs_core::{errors::ModelSelectionError, traits::make::MinerMake};

use crate::{
    hardware::{ProtoControlBoard, ProtoHardwareConfig},
    models::ProtoModel,
};

#[derive(Default)]
pub struct ProtoMake {}

impl Display for ProtoMake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Proto")
    }
}

impl MinerMake for ProtoMake {
    type Model = ProtoModel;

    fn parse_model(model: String) -> Result<Self::Model, ModelSelectionError> {
        ProtoModel::from_str(&model)
    }

    fn parse_control_board(
        &self,
        cb_type: &str,
    ) -> Option<asic_rs_core::data::board::MinerControlBoard> {
        Some(ProtoControlBoard::parse(cb_type).into())
    }
}

impl ProtoMake {
    pub fn configured_model(name: impl Into<String>, hardware: ProtoHardwareConfig) -> ProtoModel {
        ProtoModel::with_hardware(name, hardware)
    }
}
