use crate::component::proof_verification::{commit_acknowledgement, commit_packet};

use anyhow::Result;
use async_trait::async_trait;

use ibc_types::path::{
    AckPath, ChannelEndPath, CommitmentPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
};

use ibc_types::core::channel::{ChannelEnd, ChannelId, Packet, PortId};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    fn put_channel_counter(&mut self, counter: u64) {
        self.put_proto::<u64>("ibc_channel_counter".into(), counter);
    }

    async fn next_channel_id(&mut self) -> Result<ChannelId> {
        let ctr = self.get_channel_counter().await?;
        self.put_channel_counter(ctr + 1);

        Ok(ChannelId::new(ctr))
    }

    fn put_channel(&mut self, channel_id: &ChannelId, port_id: &PortId, channel: ChannelEnd) {
        self.put(
            ChannelEndPath::new(port_id, channel_id).to_string(),
            channel,
        );
    }

    fn put_ack_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(SeqAckPath::new(port_id, channel_id).to_string(), sequence);
    }

    fn put_recv_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(SeqRecvPath::new(port_id, channel_id).to_string(), sequence);
    }

    fn put_send_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(SeqSendPath::new(port_id, channel_id).to_string(), sequence);
    }

    fn put_packet_receipt(&mut self, packet: &Packet) {
        self.put_proto::<String>(
            ReceiptPath::new(&packet.port_on_b, &packet.chan_on_b, packet.sequence).to_string(),
            "1".to_string(),
        );
    }

    fn put_packet_commitment(&mut self, packet: &Packet) {
        let commitment_key =
            CommitmentPath::new(&packet.port_on_a, &packet.chan_on_a, packet.sequence).to_string();
        let packet_hash = commit_packet(packet);

        self.put_proto::<Vec<u8>>(commitment_key, packet_hash);
    }

    fn delete_packet_commitment(
        &mut self,
        channel_id: &ChannelId,
        port_id: &PortId,
        sequence: u64,
    ) {
        self.put_proto::<Vec<u8>>(
            CommitmentPath::new(port_id, channel_id, sequence.into()).to_string(),
            vec![],
        );
    }

    fn put_packet_acknowledgement(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: u64,
        acknowledgement: &[u8],
    ) {
        self.put_proto::<Vec<u8>>(
            AckPath::new(port_id, channel_id, sequence.into()).to_string(),
            commit_acknowledgement(acknowledgement),
        );
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn get_channel_counter(&self) -> Result<u64> {
        self.get_proto::<u64>("ibc_channel_counter")
            .await
            .map(|counter| counter.unwrap_or(0))
    }

    async fn get_channel(
        &self,
        channel_id: &ChannelId,
        port_id: &PortId,
    ) -> Result<Option<ChannelEnd>> {
        self.get(&ChannelEndPath::new(port_id, channel_id).to_string())
            .await
    }

    async fn get_recv_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(&SeqRecvPath::new(port_id, channel_id).to_string())
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }

    async fn get_ack_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(&SeqAckPath::new(port_id, channel_id).to_string())
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }

    async fn get_send_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(&SeqSendPath::new(port_id, channel_id).to_string())
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }

    async fn seen_packet(&self, packet: &Packet) -> Result<bool> {
        self.get_proto::<String>(
            &ReceiptPath::new(&packet.port_on_b, &packet.chan_on_b, packet.sequence).to_string(),
        )
        .await
        .map(|res| res.is_some())
    }

    async fn seen_packet_by_channel(
        &self,
        channel_id: &ChannelId,
        port_id: &PortId,
        sequence: u64,
    ) -> Result<bool> {
        self.get_proto::<String>(
            &ReceiptPath::new(port_id, channel_id, sequence.into()).to_string(),
        )
        .await
        .map(|res| res.filter(|s| !s.is_empty()))
        .map(|res| res.is_some())
    }

    async fn get_packet_commitment(&self, packet: &Packet) -> Result<Option<Vec<u8>>> {
        let commitment = self
            .get_proto::<Vec<u8>>(
                &CommitmentPath::new(&packet.port_on_a, &packet.chan_on_a, packet.sequence.into())
                    .to_string(),
            )
            .await?;

        // this is for the special case where the commitment is empty, we consider this None.
        if let Some(commitment) = commitment.as_ref() {
            if commitment.is_empty() {
                return Ok(None);
            }
        }

        Ok(commitment)
    }

    async fn get_packet_commitment_by_id(
        &self,
        channel_id: &ChannelId,
        port_id: &PortId,
        sequence: u64,
    ) -> Result<Option<Vec<u8>>> {
        let commitment = self
            .get_proto::<Vec<u8>>(
                &CommitmentPath::new(port_id, channel_id, sequence.into()).to_string(),
            )
            .await?;

        // this is for the special case where the commitment is empty, we consider this None.
        if let Some(commitment) = commitment.as_ref() {
            if commitment.is_empty() {
                return Ok(None);
            }
        }

        Ok(commitment)
    }

    async fn get_packet_acknowledgement(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: u64,
    ) -> Result<Option<Vec<u8>>> {
        self.get_proto::<Vec<u8>>(&AckPath::new(port_id, channel_id, sequence.into()).to_string())
            .await
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
