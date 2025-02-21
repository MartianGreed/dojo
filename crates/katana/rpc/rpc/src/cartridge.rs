use core::panic;
use std::sync::Arc;

use jsonrpsee::core::{async_trait, RpcResult};
use katana_core::backend::Backend;
use katana_core::service::block_producer::{BlockProducer, BlockProducerMode, PendingExecutor};
use katana_executor::ExecutorFactory;
use katana_pool::{TxPool, TransactionPool};
use katana_primitives::{ContractAddress, Felt};
use katana_primitives::transaction::{ExecutableTx,ExecutableTxWithHash, InvokeTx, InvokeTxV1};
use katana_rpc_api::dev::DevApiServer;
use katana_rpc_api::cartridge::CartridgeApiServer;
use katana_rpc_types::account::Account;
use katana_rpc_types::error::{dev::DevApiError, starknet::StarknetApiError};
use katana_rpc_types::transaction::{AddExecuteOutsideTransaction, ExecuteOutside, InvokeTxResult};
use starknet::core::types::InvokeTransactionResult;

#[allow(missing_debug_implementations)]
pub struct CartridgeApi<EF: ExecutorFactory> {
    backend: Arc<Backend<EF>>,
    block_producer: BlockProducer<EF>,
    pool: TxPool,
}

impl<EF: ExecutorFactory> CartridgeApi<EF> {
    pub fn new(backend: Arc<Backend<EF>>, block_producer: BlockProducer<EF>, pool: TxPool) -> Self {
        Self { backend, block_producer, pool }
    }
    pub fn execute_outside(&self, address: ContractAddress, outside_execution: ExecuteOutside, signature: Vec<Felt>) -> Result<InvokeTxResult, StarknetApiError> {
        let tx = match outside_execution {
            ExecuteOutside::V2(v2) => InvokeTx::V1(InvokeTxV1 {
                chain_id: self.backend.chain_spec.id(),
                nonce: v2.nonce,
                calldata: v2.calls[0].calldata.clone(),
                signature: signature,
                sender_address: address.into(),
                max_fee: 0,
            }),
            ExecuteOutside::V3(v3) => InvokeTx::V1(InvokeTxV1 {
                chain_id: self.backend.chain_spec.id(),
                nonce: v3.nonce.0,
                calldata: v3.calls[0].calldata.clone(),
                signature: signature,
                sender_address: address.into(),
                max_fee: 0,
            }),
        };
        let tx = ExecutableTxWithHash::new(ExecutableTx::Invoke(tx));
         let hash = self.pool.add_transaction(tx)?;
         Ok(InvokeTxResult::new(hash))
    }
}


#[async_trait]
impl<EF: ExecutorFactory> CartridgeApiServer for CartridgeApi<EF> {
    async fn add_execute_outside_transaction(&self, address: ContractAddress, outside_execution: ExecuteOutside, signature: Vec<Felt>) -> RpcResult<InvokeTxResult> {
        println!(
            "{:#?}", outside_execution
        );

        Ok(self.execute_outside(address, outside_execution, signature)?)
    }
}