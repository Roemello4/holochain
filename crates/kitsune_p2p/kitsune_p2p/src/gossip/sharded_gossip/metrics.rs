use std::time::Instant;

use super::*;

/// Maximum amount of history we will track
/// per remote node.
const MAX_HISTORY: usize = 10;

#[derive(Debug, Clone, Default)]
/// Information about a remote node.
struct NodeInfo {
    /// Times we recorded errors for this node.
    errors: VecDeque<Instant>,
    /// Times we recorded initiates to this node.
    initiates: VecDeque<Instant>,
    /// Times we recorded remote rounds from this node.
    remote_rounds: VecDeque<Instant>,
    /// Times we recorded complete rounds for this node.
    complete_rounds: VecDeque<Instant>,
    /// Is this node currently in an active round?
    current_round: bool,
}

#[derive(Debug, Default)]
/// Metrics tracking for remote nodes to help
/// choose which remote node to initiate the next round with.
pub(super) struct Metrics {
    /// Map of remote nodes.
    map: HashMap<StateKey, NodeInfo>,
    // Number of times we need to force initiate
    // the next round.
    force_initiates: u8,
}

/// Outcome of a gossip round.
pub(super) enum RoundOutcome {
    Success(Instant),
    Error(Instant),
}

impl Metrics {
    #[cfg(test)]
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Record a gossip round has been initiated by us.
    pub(super) fn record_initiate(&mut self, key: StateKey) {
        let info = self.map.entry(key).or_default();
        record_instant(&mut info.initiates);
        info.current_round = true;
    }

    /// Record a remote gossip round has started.
    pub(super) fn record_remote_round(&mut self, key: StateKey) {
        let info = self.map.entry(key).or_default();
        record_instant(&mut info.remote_rounds);
        info.current_round = true;
    }

    /// Record a gossip round has completed successfully.
    pub(super) fn record_success(&mut self, key: StateKey) {
        let info = self.map.entry(key).or_default();
        record_instant(&mut info.complete_rounds);
        info.current_round = false;
        if info.is_initiate_round() {
            self.force_initiates = self.force_initiates.saturating_sub(1);
        }
    }

    /// Record a gossip round has finished with an error.
    pub(super) fn record_error(&mut self, key: StateKey) {
        let info = self.map.entry(key).or_default();
        record_instant(&mut info.errors);
        info.current_round = false;
    }

    /// Record that we should force initiate the next few rounds.
    pub(super) fn record_force_initiate(&mut self) {
        self.force_initiates = MAX_TRIGGERS;
    }

    /// Get the last successful round time.
    pub(super) fn last_success(&self, key: &StateKey) -> Option<&Instant> {
        self.map
            .get(key)
            .and_then(|info| info.complete_rounds.back())
    }

    /// Is this node currently in an active round?
    pub(super) fn is_current_round(&self, key: &StateKey) -> bool {
        self.map.get(key).map_or(false, |info| info.current_round)
    }

    /// What was the last outcome for this nodes gossip round?
    pub(super) fn last_outcome(&self, key: &StateKey) -> Option<RoundOutcome> {
        self.map.get(key).and_then(
            |info| match (info.errors.back(), info.complete_rounds.back()) {
                (Some(error), Some(success)) => {
                    if error > success {
                        Some(RoundOutcome::Error(*error))
                    } else {
                        Some(RoundOutcome::Success(*success))
                    }
                }
                (Some(error), None) => Some(RoundOutcome::Error(*error)),
                (None, Some(success)) => Some(RoundOutcome::Success(*success)),
                (None, None) => None,
            },
        )
    }

    /// Should we force initiate the next round?
    pub(super) fn forced_initiate(&self) -> bool {
        self.force_initiates > 0
    }
}

impl NodeInfo {
    /// Was the last round for this node initiated by us?
    fn is_initiate_round(&self) -> bool {
        match (self.remote_rounds.back(), self.initiates.back()) {
            (None, None) | (Some(_), None) => false,
            (None, Some(_)) => true,
            (Some(remote), Some(initiate)) => initiate > remote,
        }
    }
}

fn record_instant(buffer: &mut VecDeque<Instant>) {
    if buffer.len() > MAX_HISTORY {
        buffer.pop_front();
    }
    buffer.push_back(Instant::now());
}
