use crate::{
    pow::{self, HeaderHasher},
    proto::{
        vecnod_message::Payload, GetBlockTemplateRequestMessage, GetInfoRequestMessage, NotifyBlockAddedRequestMessage,
        NotifyNewBlockTemplateRequestMessage, RpcBlock, SubmitBlockRequestMessage, VecnodMessage,
    },
    Hash,
};

impl VecnodMessage {
    #[must_use]
    #[inline(always)]
    pub fn get_info_request() -> Self {
        VecnodMessage { payload: Some(Payload::GetInfoRequest(GetInfoRequestMessage {})) }
    }
    #[must_use]
    #[inline(always)]
    pub fn notify_block_added() -> Self {
        VecnodMessage { payload: Some(Payload::NotifyBlockAddedRequest(NotifyBlockAddedRequestMessage {})) }
    }
    #[must_use]
    #[inline(always)]
    pub fn submit_block(block: RpcBlock) -> Self {
        VecnodMessage {
            payload: Some(Payload::SubmitBlockRequest(SubmitBlockRequestMessage {
                block: Some(block),
                allow_non_daa_blocks: false,
            })),
        }
    }
}

impl From<GetInfoRequestMessage> for VecnodMessage {
    #[inline(always)]
    fn from(a: GetInfoRequestMessage) -> Self {
        VecnodMessage { payload: Some(Payload::GetInfoRequest(a)) }
    }
}
impl From<NotifyBlockAddedRequestMessage> for VecnodMessage {
    #[inline(always)]
    fn from(a: NotifyBlockAddedRequestMessage) -> Self {
        VecnodMessage { payload: Some(Payload::NotifyBlockAddedRequest(a)) }
    }
}

impl From<GetBlockTemplateRequestMessage> for VecnodMessage {
    #[inline(always)]
    fn from(a: GetBlockTemplateRequestMessage) -> Self {
        VecnodMessage { payload: Some(Payload::GetBlockTemplateRequest(a)) }
    }
}

impl From<NotifyNewBlockTemplateRequestMessage> for VecnodMessage {
    fn from(a: NotifyNewBlockTemplateRequestMessage) -> Self {
        VecnodMessage { payload: Some(Payload::NotifyNewBlockTemplateRequest(a)) }
    }
}

impl RpcBlock {
    #[must_use]
    #[inline(always)]
    pub fn block_hash(&self) -> Option<Hash> {
        let mut hasher = HeaderHasher::new();
        pow::serialize_header(&mut hasher, self.header.as_ref()?, false);
        Some(hasher.finalize())
    }
}
