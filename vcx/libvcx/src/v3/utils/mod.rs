use crate::error::prelude::*;
use crate::v3::messages::connection::did_doc::Service;

pub mod encryption_envelope;

// ensure service keys are naked keys
pub(crate) fn normalize_service_keys(services: &mut Vec<Service>) -> VcxResult<()> {
    for service in services.iter_mut() {
        Service::transform_did_keys_to_naked_keys(&mut service.recipient_keys)?;
        if !service.routing_keys.is_empty() {
            Service::transform_did_keys_to_naked_keys(&mut service.routing_keys)?
        }
    }

    Ok(())
}