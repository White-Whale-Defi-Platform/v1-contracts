use core::result::Result::{Err, Ok};

use cosmwasm_std::{Decimal, StdError, StdResult};

use crate::contract::{
    MAX_DESC_LENGTH, MAX_LINK_LENGTH, MAX_TITLE_LENGTH, MIN_DESC_LENGTH, MIN_LINK_LENGTH,
    MIN_TITLE_LENGTH,
};

/**
 * Validates the quorum parameter used to instantiate the contract. It should be between [0,1].
 */
pub fn validate_quorum(quorum: Decimal) -> StdResult<()> {
    if quorum > Decimal::one() {
        Err(StdError::generic_err("quorum must be 0 to 1"))
    } else {
        Ok(())
    }
}

/**
 * Validates the threshold parameter used to instantiate the contract. It should be between [0,1].
 */
pub fn validate_threshold(threshold: Decimal) -> StdResult<()> {
    if threshold > Decimal::one() {
        Err(StdError::generic_err("threshold must be 0 to 1"))
    } else {
        Ok(())
    }
}

/**
 * Validates that the link is valid when creating a poll.
 */
pub fn validate_poll_link(link: &Option<String>) -> StdResult<()> {
    if let Some(link) = link {
        if link.len() < MIN_LINK_LENGTH {
            Err(StdError::generic_err("Link too short"))
        } else if link.len() > MAX_LINK_LENGTH {
            Err(StdError::generic_err("Link too long"))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/**
 * Validates that the title of the poll is valid, i.e. len() between [MIN_TITLE_LENGTH, MAX_TITLE_LENGTH].
 */
pub fn validate_poll_title(title: &str) -> StdResult<()> {
    if title.len() < MIN_TITLE_LENGTH {
        Err(StdError::generic_err("title too short"))
    } else if title.len() > MAX_TITLE_LENGTH {
        Err(StdError::generic_err("title too long"))
    } else {
        Ok(())
    }
}

/**
 * Validates that the description of the poll is valid, i.e. len() between [MIN_DESC_LENGTH, MAX_DESC_LENGTH].
 */
pub fn validate_poll_description(description: &str) -> StdResult<()> {
    if description.len() < MIN_DESC_LENGTH {
        Err(StdError::generic_err("Description too short"))
    } else if description.len() > MAX_DESC_LENGTH {
        Err(StdError::generic_err("Description too long"))
    } else {
        Ok(())
    }
}
