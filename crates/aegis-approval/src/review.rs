/// SHA-256 of a candidate payload. The machine never sees the text itself.
pub type Hash = [u8; 32];

/// Review state (a subset of the Review Center contract's `ReviewState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// A proposal exists; the user decides whether to apply it.
    AwaitingTransformDecision,
    /// A candidate payload is being re-inspected by the pipeline.
    Reinspecting,
    /// A candidate is cleared; the user decides whether to send it.
    AwaitingTransmitDecision,
    /// Transmit approved and a token must be minted. Terminal for the review.
    Allowed,
    /// The user rejected. Terminal: nothing is sent.
    Rejected,
    /// Mandatory block or fail-closed. Terminal: nothing is sent.
    Blocked,
    /// The approval timed out. Terminal: nothing is sent.
    Expired,
}

impl State {
    /// Whether no further decision is possible.
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Allowed | Self::Rejected | Self::Blocked | Self::Expired
        )
    }
}

/// Which text would be sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransmitContent {
    /// The user's original text, unmodified.
    Original,
    /// A transformed (engine-proposed or user-edited) text.
    Transformed,
}

/// How much friction the transmit approval needs (ADR-0008).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strength {
    /// One click.
    OneClick,
    /// The user must acknowledge each required finding.
    ExplicitConfirm,
}

/// A candidate payload: its hash and a version that increases with each
/// engine proposal or user edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Payload {
    /// SHA-256 of the exact bytes.
    pub sha256: Hash,
    /// Version, starting at 1.
    pub version: u32,
}

/// Inputs fixed when the review is opened.
#[derive(Debug, Clone)]
pub struct ReviewConfig {
    /// SHA-256 of the original payload.
    pub original_sha256: Hash,
    /// False when mandatory findings forbid ever sending the original.
    pub original_sendable: bool,
    /// The engine's proposal, if any.
    pub proposal: Option<Payload>,
    /// Transmit approval strength.
    pub strength: Strength,
    /// Finding ids that must be acknowledged for [`Strength::ExplicitConfirm`].
    pub required_acknowledgements: Vec<u32>,
    /// Absolute expiry, milliseconds since the Unix epoch.
    pub expires_at_ms: u64,
}

/// Result of re-inspecting a candidate payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReinspectOutcome {
    /// Nothing that needs a decision.
    Clean,
    /// Review-level findings remain.
    NeedsReview,
    /// A mandatory block applies (e.g. a secret was typed into an edit).
    Blocked,
    /// Re-inspection could not complete: fail closed.
    Failed,
}

/// Inputs to [`Review::step`]. All fields are trusted: the engine hashes text
/// itself and fills the revision from the UI command; the UI never supplies a
/// hash for an edit.
#[derive(Debug, Clone)]
pub enum Input {
    /// Apply the current proposal.
    ApproveTransform {
        /// Revision the user decided against.
        revision: u64,
        /// Proposal version the user saw.
        proposal_version: u32,
    },
    /// Keep the original text; go on to the transmit decision.
    DeclineTransform {
        /// Revision the user decided against.
        revision: u64,
    },
    /// The user edited the candidate; `new_payload` is the engine's hash of it.
    Edited {
        /// Revision the user decided against.
        revision: u64,
        /// SHA-256 of the edited text.
        new_payload: Hash,
    },
    /// Approve sending exactly the hash the user was shown.
    ApproveTransmit {
        /// Revision the user decided against.
        revision: u64,
        /// Hash echoed back by the UI.
        payload_sha256: Hash,
        /// Finding ids the user acknowledged.
        acknowledged: Vec<u32>,
    },
    /// Reject: nothing is sent.
    Reject {
        /// Revision the user decided against.
        revision: u64,
    },
    /// The pipeline finished re-inspecting the pending candidate.
    Reinspected {
        /// What it found.
        outcome: ReinspectOutcome,
    },
    /// Time has advanced.
    Tick {
        /// Current time in milliseconds since the Unix epoch.
        now_ms: u64,
    },
}

/// What the caller must do after a transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Run the full inspection pipeline over this payload, then feed
    /// [`Input::Reinspected`].
    Reinspect {
        /// Hash of the payload to re-inspect.
        payload_sha256: Hash,
    },
    /// Mint an approval token for exactly this payload.
    Mint {
        /// Hash to bind the token to.
        payload_sha256: Hash,
        /// Whether this is the original or a transformed text.
        content: TransmitContent,
    },
    /// The review ended without approval; release held requests with an error.
    Closed(State),
}

/// Why an input was refused. A refused input never changes the state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewError {
    /// The decision was made against an older revision.
    StaleRevision,
    /// The input is not valid in the current state.
    WrongState,
    /// The review already ended.
    Terminal,
    /// The echoed hash is not the one that would be sent.
    HashMismatch,
    /// The user approved a proposal that has since been replaced.
    ProposalSuperseded,
    /// A required acknowledgement is missing.
    MissingAcknowledgement,
    /// Mandatory findings forbid sending the original text.
    OriginalNotSendable,
}

/// The approval state machine for one request.
#[derive(Debug, Clone)]
pub struct Review {
    state: State,
    revision: u64,
    original: Hash,
    original_sendable: bool,
    proposal: Option<Payload>,
    pending: Option<Payload>,
    chosen: Option<(TransmitContent, Hash)>,
    strength: Strength,
    required: Vec<u32>,
    expires_at_ms: u64,
}

impl Review {
    /// Open a review. With a proposal the first decision is the transform; with
    /// none, the original goes straight to the transmit decision, or the
    /// request is blocked when the original may never be sent.
    #[must_use]
    pub fn new(cfg: ReviewConfig) -> Self {
        let (state, chosen) = match (cfg.proposal, cfg.original_sendable) {
            (Some(_), _) => (State::AwaitingTransformDecision, None),
            (None, true) => (
                State::AwaitingTransmitDecision,
                Some((TransmitContent::Original, cfg.original_sha256)),
            ),
            (None, false) => (State::Blocked, None),
        };
        Self {
            state,
            revision: 1,
            original: cfg.original_sha256,
            original_sendable: cfg.original_sendable,
            proposal: cfg.proposal,
            pending: None,
            chosen,
            strength: cfg.strength,
            required: cfg.required_acknowledgements,
            expires_at_ms: cfg.expires_at_ms,
        }
    }

    /// A review that is blocked from the start (mandatory block; nothing is
    /// approvable).
    #[must_use]
    pub fn blocked(original_sha256: Hash) -> Self {
        Self {
            state: State::Blocked,
            revision: 1,
            original: original_sha256,
            original_sendable: false,
            proposal: None,
            pending: None,
            chosen: None,
            strength: Strength::OneClick,
            required: Vec::new(),
            expires_at_ms: 0,
        }
    }

    /// Current state.
    #[must_use]
    pub fn state(&self) -> State {
        self.state
    }

    /// Current revision; commands must carry it.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The hash that approving transmit would bind to, if a candidate is
    /// cleared. This is what the UI must echo.
    #[must_use]
    pub fn transmit_binding(&self) -> Option<(TransmitContent, Hash)> {
        (self.state == State::AwaitingTransmitDecision)
            .then_some(self.chosen)
            .flatten()
    }

    fn check_revision(&self, revision: u64) -> Result<(), ReviewError> {
        if revision == self.revision {
            Ok(())
        } else {
            Err(ReviewError::StaleRevision)
        }
    }

    fn advance(&mut self, state: State) {
        self.state = state;
        self.revision += 1;
    }

    /// Apply one input. On error the state is unchanged.
    ///
    /// # Errors
    /// See [`ReviewError`].
    pub fn step(&mut self, input: Input) -> Result<Vec<Effect>, ReviewError> {
        if let Input::Tick { now_ms } = input {
            if self.state.is_terminal() || now_ms < self.expires_at_ms {
                return Ok(Vec::new());
            }
            self.chosen = None;
            self.pending = None;
            self.advance(State::Expired);
            return Ok(vec![Effect::Closed(State::Expired)]);
        }
        if self.state.is_terminal() {
            return Err(ReviewError::Terminal);
        }
        match input {
            Input::Tick { .. } => Ok(Vec::new()),
            Input::Reject { revision } => {
                self.check_revision(revision)?;
                self.chosen = None;
                self.pending = None;
                self.advance(State::Rejected);
                Ok(vec![Effect::Closed(State::Rejected)])
            }
            Input::ApproveTransform {
                revision,
                proposal_version,
            } => {
                self.check_revision(revision)?;
                if self.state != State::AwaitingTransformDecision {
                    return Err(ReviewError::WrongState);
                }
                let proposal = self
                    .proposal
                    .filter(|p| p.version == proposal_version)
                    .ok_or(ReviewError::ProposalSuperseded)?;
                self.pending = Some(proposal);
                self.chosen = None;
                self.advance(State::Reinspecting);
                Ok(vec![Effect::Reinspect {
                    payload_sha256: proposal.sha256,
                }])
            }
            Input::DeclineTransform { revision } => {
                self.check_revision(revision)?;
                if self.state != State::AwaitingTransformDecision {
                    return Err(ReviewError::WrongState);
                }
                if !self.original_sendable {
                    return Err(ReviewError::OriginalNotSendable);
                }
                self.chosen = Some((TransmitContent::Original, self.original));
                self.advance(State::AwaitingTransmitDecision);
                Ok(Vec::new())
            }
            Input::Edited {
                revision,
                new_payload,
            } => {
                self.check_revision(revision)?;
                if !matches!(
                    self.state,
                    State::AwaitingTransformDecision | State::AwaitingTransmitDecision
                ) {
                    return Err(ReviewError::WrongState);
                }
                let version = self.proposal.map_or(1, |p| p.version + 1);
                let edited = Payload {
                    sha256: new_payload,
                    version,
                };
                self.proposal = Some(edited);
                self.pending = Some(edited);
                // Any earlier clearance or grant is void (ADR-0008).
                self.chosen = None;
                self.advance(State::Reinspecting);
                Ok(vec![Effect::Reinspect {
                    payload_sha256: new_payload,
                }])
            }
            Input::Reinspected { outcome } => {
                if self.state != State::Reinspecting {
                    return Err(ReviewError::WrongState);
                }
                match (outcome, self.pending) {
                    (ReinspectOutcome::Clean, Some(p)) => {
                        self.chosen = Some((TransmitContent::Transformed, p.sha256));
                        self.advance(State::AwaitingTransmitDecision);
                        Ok(Vec::new())
                    }
                    (ReinspectOutcome::NeedsReview, Some(_)) => {
                        self.chosen = None;
                        self.advance(State::AwaitingTransformDecision);
                        Ok(Vec::new())
                    }
                    // Blocked, Failed, or an impossible missing candidate: fail closed.
                    _ => {
                        self.chosen = None;
                        self.pending = None;
                        self.advance(State::Blocked);
                        Ok(vec![Effect::Closed(State::Blocked)])
                    }
                }
            }
            Input::ApproveTransmit {
                revision,
                payload_sha256,
                acknowledged,
            } => {
                self.check_revision(revision)?;
                if self.state != State::AwaitingTransmitDecision {
                    return Err(ReviewError::WrongState);
                }
                let (content, bound) = self.chosen.ok_or(ReviewError::WrongState)?;
                if payload_sha256 != bound {
                    return Err(ReviewError::HashMismatch);
                }
                if content == TransmitContent::Original && !self.original_sendable {
                    return Err(ReviewError::OriginalNotSendable);
                }
                if self.strength == Strength::ExplicitConfirm
                    && !self.required.iter().all(|r| acknowledged.contains(r))
                {
                    return Err(ReviewError::MissingAcknowledgement);
                }
                self.advance(State::Allowed);
                Ok(vec![Effect::Mint {
                    payload_sha256: bound,
                    content,
                }])
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn h(n: u8) -> Hash {
        [n; 32]
    }

    const ORIG: u8 = 1;
    const PROP: u8 = 2;
    const EDIT: u8 = 3;
    const EDIT2: u8 = 4;
    const WRONG: u8 = 9;

    fn cfg(proposal: bool, original_sendable: bool, strength: Strength) -> ReviewConfig {
        ReviewConfig {
            original_sha256: h(ORIG),
            original_sendable,
            proposal: proposal.then_some(Payload {
                sha256: h(PROP),
                version: 1,
            }),
            strength,
            required_acknowledgements: vec![7],
            expires_at_ms: 1_000,
        }
    }

    fn rev(r: &Review) -> u64 {
        r.revision()
    }

    #[test]
    fn happy_path_transform_then_transmit_mints_for_the_proposal_hash() {
        let mut r = Review::new(cfg(true, true, Strength::OneClick));
        assert_eq!(r.state(), State::AwaitingTransformDecision);
        let fx = r
            .step(Input::ApproveTransform {
                revision: rev(&r),
                proposal_version: 1,
            })
            .unwrap();
        assert_eq!(
            fx,
            vec![Effect::Reinspect {
                payload_sha256: h(PROP)
            }]
        );
        assert_eq!(r.state(), State::Reinspecting);
        r.step(Input::Reinspected {
            outcome: ReinspectOutcome::Clean,
        })
        .unwrap();
        assert_eq!(
            r.transmit_binding(),
            Some((TransmitContent::Transformed, h(PROP)))
        );
        let fx = r
            .step(Input::ApproveTransmit {
                revision: rev(&r),
                payload_sha256: h(PROP),
                acknowledged: vec![],
            })
            .unwrap();
        assert_eq!(
            fx,
            vec![Effect::Mint {
                payload_sha256: h(PROP),
                content: TransmitContent::Transformed
            }]
        );
        assert_eq!(r.state(), State::Allowed);
    }

    #[test]
    fn at05_rejection_sends_nothing_and_cannot_be_undone() {
        let mut r = Review::new(cfg(true, true, Strength::OneClick));
        let fx = r.step(Input::Reject { revision: rev(&r) }).unwrap();
        assert_eq!(fx, vec![Effect::Closed(State::Rejected)]);
        for input in [
            Input::ApproveTransform {
                revision: rev(&r),
                proposal_version: 1,
            },
            Input::ApproveTransmit {
                revision: rev(&r),
                payload_sha256: h(PROP),
                acknowledged: vec![],
            },
        ] {
            assert_eq!(r.step(input), Err(ReviewError::Terminal));
        }
    }

    #[test]
    fn timeout_expires_and_later_approval_is_refused() {
        let mut r = Review::new(cfg(false, true, Strength::OneClick));
        assert_eq!(r.step(Input::Tick { now_ms: 999 }).unwrap(), vec![]);
        assert_eq!(
            r.step(Input::Tick { now_ms: 1_000 }).unwrap(),
            vec![Effect::Closed(State::Expired)]
        );
        assert_eq!(
            r.step(Input::ApproveTransmit {
                revision: rev(&r),
                payload_sha256: h(ORIG),
                acknowledged: vec![]
            }),
            Err(ReviewError::Terminal)
        );
    }

    #[test]
    fn at06_edit_is_reinspected_and_a_new_secret_blocks() {
        let mut r = Review::new(cfg(true, true, Strength::OneClick));
        r.step(Input::Edited {
            revision: rev(&r),
            new_payload: h(EDIT),
        })
        .unwrap();
        assert_eq!(r.state(), State::Reinspecting);
        // Approving transmit before re-inspection finishes is refused.
        assert_eq!(
            r.step(Input::ApproveTransmit {
                revision: rev(&r),
                payload_sha256: h(EDIT),
                acknowledged: vec![]
            }),
            Err(ReviewError::WrongState)
        );
        let fx = r
            .step(Input::Reinspected {
                outcome: ReinspectOutcome::Blocked,
            })
            .unwrap();
        assert_eq!(fx, vec![Effect::Closed(State::Blocked)]);
        assert_eq!(
            r.step(Input::ApproveTransmit {
                revision: rev(&r),
                payload_sha256: h(EDIT),
                acknowledged: vec![]
            }),
            Err(ReviewError::Terminal)
        );
    }

    #[test]
    fn reinspection_failure_fails_closed() {
        let mut r = Review::new(cfg(true, true, Strength::OneClick));
        r.step(Input::Edited {
            revision: rev(&r),
            new_payload: h(EDIT),
        })
        .unwrap();
        r.step(Input::Reinspected {
            outcome: ReinspectOutcome::Failed,
        })
        .unwrap();
        assert_eq!(r.state(), State::Blocked);
    }

    #[test]
    fn edit_after_clearance_voids_the_old_hash() {
        let mut r = Review::new(cfg(false, true, Strength::OneClick));
        assert_eq!(
            r.transmit_binding(),
            Some((TransmitContent::Original, h(ORIG)))
        );
        r.step(Input::Edited {
            revision: rev(&r),
            new_payload: h(EDIT),
        })
        .unwrap();
        r.step(Input::Reinspected {
            outcome: ReinspectOutcome::Clean,
        })
        .unwrap();
        assert_eq!(
            r.step(Input::ApproveTransmit {
                revision: rev(&r),
                payload_sha256: h(ORIG),
                acknowledged: vec![]
            }),
            Err(ReviewError::HashMismatch)
        );
        assert!(r
            .step(Input::ApproveTransmit {
                revision: rev(&r),
                payload_sha256: h(EDIT),
                acknowledged: vec![]
            })
            .is_ok());
    }

    #[test]
    fn hash_mismatch_changes_nothing_and_mints_nothing() {
        let mut r = Review::new(cfg(false, true, Strength::OneClick));
        let before = (r.state(), r.revision());
        assert_eq!(
            r.step(Input::ApproveTransmit {
                revision: rev(&r),
                payload_sha256: h(WRONG),
                acknowledged: vec![]
            }),
            Err(ReviewError::HashMismatch)
        );
        assert_eq!((r.state(), r.revision()), before);
    }

    #[test]
    fn stale_revision_is_refused() {
        let mut r = Review::new(cfg(true, true, Strength::OneClick));
        let old = rev(&r);
        r.step(Input::Edited {
            revision: old,
            new_payload: h(EDIT),
        })
        .unwrap();
        assert_eq!(
            r.step(Input::Reject { revision: old }),
            Err(ReviewError::StaleRevision)
        );
    }

    #[test]
    fn superseded_proposal_cannot_be_approved() {
        let mut r = Review::new(cfg(true, true, Strength::OneClick));
        r.step(Input::Edited {
            revision: rev(&r),
            new_payload: h(EDIT),
        })
        .unwrap();
        r.step(Input::Reinspected {
            outcome: ReinspectOutcome::NeedsReview,
        })
        .unwrap();
        assert_eq!(r.state(), State::AwaitingTransformDecision);
        assert_eq!(
            r.step(Input::ApproveTransform {
                revision: rev(&r),
                proposal_version: 1
            }),
            Err(ReviewError::ProposalSuperseded)
        );
    }

    #[test]
    fn explicit_confirm_needs_every_acknowledgement() {
        let mut r = Review::new(cfg(false, true, Strength::ExplicitConfirm));
        assert_eq!(
            r.step(Input::ApproveTransmit {
                revision: rev(&r),
                payload_sha256: h(ORIG),
                acknowledged: vec![]
            }),
            Err(ReviewError::MissingAcknowledgement)
        );
        assert!(r
            .step(Input::ApproveTransmit {
                revision: rev(&r),
                payload_sha256: h(ORIG),
                acknowledged: vec![7]
            })
            .is_ok());
    }

    #[test]
    fn original_with_mandatory_findings_can_never_be_sent() {
        let mut r = Review::new(cfg(true, false, Strength::OneClick));
        assert_eq!(
            r.step(Input::DeclineTransform { revision: rev(&r) }),
            Err(ReviewError::OriginalNotSendable)
        );
        assert_eq!(
            Review::new(cfg(false, false, Strength::OneClick)).state(),
            State::Blocked
        );
        assert_eq!(Review::blocked(h(ORIG)).state(), State::Blocked);
    }

    /// Exhaustive check of every input sequence up to length 5: a token is
    /// minted only for a hash that was cleared by re-inspection (or the
    /// original, when sendable), never twice, never after a terminal state.
    #[test]
    fn no_sequence_of_inputs_can_mint_an_unapproved_payload() {
        #[derive(Clone, Copy)]
        enum Op {
            ApproveTransform(u32),
            Decline,
            Edit(u8),
            Transmit(u8),
            Reject,
            Reinspect(ReinspectOutcome),
            Tick(u64),
            StaleReject,
        }
        let ops = [
            Op::ApproveTransform(1),
            Op::ApproveTransform(9),
            Op::Decline,
            Op::Edit(EDIT),
            Op::Edit(EDIT2),
            Op::Transmit(ORIG),
            Op::Transmit(PROP),
            Op::Transmit(EDIT),
            Op::Transmit(WRONG),
            Op::Reject,
            Op::Reinspect(ReinspectOutcome::Clean),
            Op::Reinspect(ReinspectOutcome::NeedsReview),
            Op::Reinspect(ReinspectOutcome::Blocked),
            Op::Reinspect(ReinspectOutcome::Failed),
            Op::Tick(5_000),
            Op::StaleReject,
        ];
        let configs = [
            cfg(true, true, Strength::OneClick),
            cfg(true, false, Strength::OneClick),
            cfg(false, true, Strength::ExplicitConfirm),
            cfg(false, false, Strength::OneClick),
        ];
        let mut sequences = 0u64;
        for c in &configs {
            let mut stack: Vec<Vec<Op>> = vec![Vec::new()];
            while let Some(seq) = stack.pop() {
                // Replay `seq` from a fresh review, checking invariants.
                let mut r = Review::new(c.clone());
                let mut cleared: HashSet<Hash> = HashSet::new();
                if c.proposal.is_none() && c.original_sendable {
                    cleared.insert(h(ORIG));
                }
                let mut declined = false;
                let mut pending: Option<Hash> = None;
                let mut minted = 0;
                for op in &seq {
                    let before_state = r.state();
                    let input = match *op {
                        Op::ApproveTransform(v) => Input::ApproveTransform {
                            revision: r.revision(),
                            proposal_version: v,
                        },
                        Op::Decline => Input::DeclineTransform {
                            revision: r.revision(),
                        },
                        Op::Edit(e) => Input::Edited {
                            revision: r.revision(),
                            new_payload: h(e),
                        },
                        Op::Transmit(t) => Input::ApproveTransmit {
                            revision: r.revision(),
                            payload_sha256: h(t),
                            acknowledged: vec![7],
                        },
                        Op::Reject => Input::Reject {
                            revision: r.revision(),
                        },
                        Op::Reinspect(o) => Input::Reinspected { outcome: o },
                        Op::Tick(t) => Input::Tick { now_ms: t },
                        Op::StaleReject => Input::Reject {
                            revision: r.revision().wrapping_sub(1),
                        },
                    };
                    let is_decline = matches!(input, Input::DeclineTransform { .. });
                    let result = r.step(input);
                    if before_state.is_terminal() {
                        assert!(
                            !matches!(&result, Ok(fx) if fx.iter().any(|e| matches!(e, Effect::Mint { .. }))),
                            "minted after terminal state"
                        );
                    }
                    if let Ok(effects) = &result {
                        if is_decline && c.original_sendable {
                            declined = true;
                            cleared.insert(h(ORIG));
                        }
                        for e in effects {
                            match e {
                                Effect::Reinspect { payload_sha256 } => {
                                    pending = Some(*payload_sha256)
                                }
                                Effect::Mint {
                                    payload_sha256,
                                    content,
                                } => {
                                    minted += 1;
                                    assert!(
                                        cleared.contains(payload_sha256),
                                        "minted an uncleared hash"
                                    );
                                    assert!(*payload_sha256 != h(WRONG), "minted the wrong hash");
                                    if *content == TransmitContent::Original {
                                        assert!(
                                            c.original_sendable,
                                            "original minted though forbidden"
                                        );
                                    }
                                }
                                Effect::Closed(_) => {}
                            }
                        }
                    }
                    if let Op::Reinspect(ReinspectOutcome::Clean) = op {
                        if result.is_ok() {
                            if let Some(p) = pending {
                                cleared.insert(p);
                            }
                        }
                    }
                    if let Op::Edit(_) | Op::ApproveTransform(_) = op {
                        if result.is_ok() {
                            // A new candidate is under review; earlier clearances
                            // of *other* hashes must not be usable for it.
                            cleared.retain(|x| *x == h(ORIG) && (declined || c.proposal.is_none()));
                        }
                    }
                    assert!(minted <= 1, "minted twice");
                }
                sequences += 1;
                if seq.len() < 5 {
                    for op in &ops {
                        let mut next = seq.clone();
                        next.push(*op);
                        stack.push(next);
                    }
                }
            }
        }
        assert!(sequences > 100_000, "explored only {sequences} sequences");
    }
}
