use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use katana_rpc_types::trace::TxExecutionInfo;
use katana_rpc_types::transaction::BroadcastedInvokeTx;

#[cfg_attr(not(feature = "client"), rpc(server, namespace = "cartridge"))]
#[cfg_attr(feature = "client", rpc(client, server, namespace = "cartridge"))]
pub trait CartridgeApi {
    /// Exectues transaction on behalf of caller.
    #[method(name = "addExecuteOutsideTransaction")]
    async fn add_execute_outside_transaction(
        &self,
        invoke_transaction: BroadcastedInvokeTx,
    ) -> RpcResult<Vec<TxExecutionInfo>>;
}
