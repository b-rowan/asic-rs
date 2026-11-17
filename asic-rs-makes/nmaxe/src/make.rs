use std::{fmt::Display, str::FromStr};

use asic_rs_core::{errors::ModelSelectionError, traits::make::MinerMake};

use crate::models::NMAxeModel;

#[derive(Default)]
pub struct NMAxeMake {}

impl Display for NMAxeMake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NMAxe")
    }
}

impl MinerMake for NMAxeMake {
    type Model = NMAxeModel;
    fn parse_model(model: String) -> Result<Self::Model, ModelSelectionError> {
        NMAxeModel::from_str(&model)
    }
}
