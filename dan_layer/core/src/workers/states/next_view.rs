// Copyright 2021. The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use log::*;
use tari_shutdown::ShutdownSignal;

use crate::{
    digital_assets_error::DigitalAssetError,
    models::{AssetDefinition, Committee, HotStuffMessage, Payload, View},
    services::infrastructure_services::{NodeAddressable, OutboundService},
    storage::DbFactory,
    workers::states::ConsensusWorkerStateEvent,
};

const LOG_TARGET: &str = "tari::dan::workers::states::next_view";

#[derive(Default)]
pub struct NextViewState {}

impl NextViewState {
    pub async fn next_event<
        TPayload: Payload,
        TOutboundService: OutboundService<TAddr, TPayload>,
        TAddr: NodeAddressable + Clone + Send,
        TDbFactory: DbFactory,
    >(
        &mut self,
        current_view: &View,
        db_factory: &TDbFactory,
        broadcast: &mut TOutboundService,
        committee: &Committee<TAddr>,
        node_id: TAddr,
        asset_definition: &AssetDefinition,
        _shutdown: &ShutdownSignal,
    ) -> Result<ConsensusWorkerStateEvent, DigitalAssetError> {
        let db = db_factory.get_or_create_chain_db(&asset_definition.public_key)?;
        let prepare_qc = db.find_highest_prepared_qc()?;
        let message = HotStuffMessage::new_view(prepare_qc, current_view.view_id, asset_definition.public_key.clone());
        let next_view = current_view.view_id.next();
        let leader = committee.leader_for_view(next_view);
        broadcast.send(node_id, leader.clone(), message).await?;
        info!(target: LOG_TARGET, "End of view: {}", current_view.view_id.0);
        debug!(target: LOG_TARGET, "--------------------------------");
        Ok(ConsensusWorkerStateEvent::NewView { new_view: next_view })
    }
}