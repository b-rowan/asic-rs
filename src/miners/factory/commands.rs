use crate::miners::commands::MinerCommand;

pub(crate) const RPC_DEVDETAILS: MinerCommand = MinerCommand::RPC {
    command: "devdetails",
};
pub(crate) const RPC_VERSION: MinerCommand = MinerCommand::RPC { command: "version" };
pub(crate) const HTTP_WEB_ROOT: MinerCommand = MinerCommand::WebAPI { command: "/" };
