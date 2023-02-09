use crate::{Error, Keypair, Result};
use futures::TryFutureExt;
use helium_crypto::Sign;
use helium_proto::{
    services::{
        iot_config, poc_lora,
        router::{PacketRouterPacketUpV1, PacketRouterRegisterV1},
    },
    BlockchainTxnAddGatewayV1, Message,
};

#[async_trait::async_trait]
pub trait MsgSign: Message + std::clone::Clone {
    async fn sign<T>(&self, keypair: T) -> Result<Vec<u8>>
    where
        Self: std::marker::Sized,
        T: AsRef<Keypair> + std::marker::Send + 'static;
}

macro_rules! impl_msg_sign {
    ($txn_type:ty, $( $sig: ident ),+ ) => {
        #[async_trait::async_trait]
        impl MsgSign for $txn_type {
            async fn sign<T>(&self, keypair: T) -> Result<Vec<u8>>
            where T: AsRef<Keypair> + std::marker::Send + 'static {
                let mut txn = self.clone();
                $(txn.$sig = vec![];)+
                let buf = txn.encode_to_vec();
                let join_handle: tokio::task::JoinHandle<Result<Vec<u8>>> = tokio::task::spawn_blocking(move ||  {
                    keypair.as_ref().sign(&buf).map_err(Error::from)
                });
                join_handle.map_err(|err| helium_crypto::Error::from(signature::Error::from_source(err))).await?
            }
        }
    };
}

impl_msg_sign!(PacketRouterPacketUpV1, signature);
impl_msg_sign!(PacketRouterRegisterV1, signature);
impl_msg_sign!(
    BlockchainTxnAddGatewayV1,
    owner_signature,
    payer_signature,
    gateway_signature
);

impl_msg_sign!(iot_config::GatewayRegionParamsReqV1, signature);
impl_msg_sign!(poc_lora::LoraBeaconReportReqV1, signature);
impl_msg_sign!(poc_lora::LoraWitnessReportReqV1, signature);
