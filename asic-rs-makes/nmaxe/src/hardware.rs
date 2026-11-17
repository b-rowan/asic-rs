use crate::models::NMAxeModel;
use asic_rs_core::data::device::MinerHardware;

impl From<NMAxeModel> for MinerHardware {
    fn from(model: NMAxeModel) -> Self {
        match model {
            NMAxeModel::NMAxe | NMAxeModel::NMAxeGamma => Self {
                fans: Some(1),
                boards: Some(vec![Some(1)]),
            },
            NMAxeModel::NMQAxePlusPlus | NMAxeModel::NMQAxePlusPlusRev81 => Self {
                fans: Some(2),
                boards: Some(vec![Some(4)]),
            },
            NMAxeModel::NMQAxePlusPlusNexus => Self {
                fans: Some(2),
                boards: Some(vec![Some(2)]),
            },
            NMAxeModel::Unknown(_) => Self::default(),
        }
    }
}
