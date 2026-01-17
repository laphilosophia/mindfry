//! MFBP Command Handler
//!
//! Executes requests against the MindFry database.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::MindFry;
use crate::arena::Lineage;
use crate::graph::Bond;

use super::Request;
use super::message::*;

/// Command handler for MFBP requests
pub struct CommandHandler {
    /// Reference to the MindFry database
    db: Arc<RwLock<MindFry>>,
    /// Server start time
    start_time: Instant,
    /// Is decay frozen?
    is_frozen: bool,
}

impl CommandHandler {
    /// Create a new command handler
    pub fn new(db: Arc<RwLock<MindFry>>) -> Self {
        Self {
            db,
            start_time: Instant::now(),
            is_frozen: false,
        }
    }

    /// Handle a request and return a response
    pub fn handle(&mut self, request: Request) -> Response {
        match request {
            // ═══════════════════════════════════════════════════════════════
            // LINEAGE OPERATIONS
            // ═══════════════════════════════════════════════════════════════
            Request::LineageCreate {
                id,
                energy,
                threshold,
                decay_rate,
            } => {
                let mut db = self.db.write().unwrap();
                let key = self.hash_key(&id);

                if db.psyche.lookup(key).is_some() {
                    return Response::Error {
                        code: ErrorCode::LineageExists,
                        message: format!("Lineage '{}' already exists", id),
                    };
                }

                let lineage = Lineage::with_config(energy, threshold, decay_rate);
                db.psyche.alloc_with_key(key, lineage);

                Response::Ok(ResponseData::Ack)
            }

            Request::LineageGet { id, flags } => {
                use crate::protocol::QueryFlags;

                let query_flags = QueryFlags::from_bits_truncate(flags);
                let _bypass = query_flags.contains(QueryFlags::BYPASS_FILTERS);
                let _include_repressed = query_flags.contains(QueryFlags::INCLUDE_REPRESSED);
                let no_side_effects = query_flags.contains(QueryFlags::NO_SIDE_EFFECTS);

                // Use read or write lock based on side effects
                if no_side_effects {
                    let db = self.db.read().unwrap();
                    let key = self.hash_key(&id);

                    match db.psyche.lookup(key) {
                        Some(lineage_id) => match db.psyche.get(lineage_id) {
                            Some(lineage) => {
                                // TODO: Check antagonism suppression here
                                // For now, return lineage directly
                                Response::Ok(ResponseData::Lineage(LineageInfo {
                                    id,
                                    energy: lineage.current_energy(),
                                    threshold: lineage.threshold,
                                    decay_rate: lineage.decay_rate,
                                    rigidity: lineage.rigidity,
                                    is_conscious: lineage.is_conscious(),
                                    last_access_ms: lineage.last_access / 1_000_000,
                                }))
                            }
                            None => Response::Error {
                                code: ErrorCode::LineageNotFound,
                                message: format!("Lineage '{}' not found", id),
                            },
                        },
                        None => Response::Error {
                            code: ErrorCode::LineageNotFound,
                            message: format!("Lineage '{}' not found", id),
                        },
                    }
                } else {
                    // Observer effect: stimulate on read
                    let mut db = self.db.write().unwrap();
                    let key = self.hash_key(&id);

                    match db.psyche.lookup(key) {
                        Some(lineage_id) => match db.psyche.get_mut(lineage_id) {
                            Some(lineage) => {
                                // Observer effect: reading strengthens memory
                                lineage.stimulate(0.01);

                                Response::Ok(ResponseData::Lineage(LineageInfo {
                                    id,
                                    energy: lineage.current_energy(),
                                    threshold: lineage.threshold,
                                    decay_rate: lineage.decay_rate,
                                    rigidity: lineage.rigidity,
                                    is_conscious: lineage.is_conscious(),
                                    last_access_ms: lineage.last_access / 1_000_000,
                                }))
                            }
                            None => Response::Error {
                                code: ErrorCode::LineageNotFound,
                                message: format!("Lineage '{}' not found", id),
                            },
                        },
                        None => Response::Error {
                            code: ErrorCode::LineageNotFound,
                            message: format!("Lineage '{}' not found", id),
                        },
                    }
                }
            }

            Request::LineageStimulate { id, delta } => {
                let mut db = self.db.write().unwrap();
                let key = self.hash_key(&id);

                match db.psyche.lookup(key) {
                    Some(lineage_id) => match db.psyche.get_mut(lineage_id) {
                        Some(lineage) => {
                            lineage.stimulate(delta);
                            Response::Ok(ResponseData::Ack)
                        }
                        None => Response::Error {
                            code: ErrorCode::LineageNotFound,
                            message: format!("Lineage '{}' not found", id),
                        },
                    },
                    None => Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Lineage '{}' not found", id),
                    },
                }
            }

            Request::LineageForget { id } => {
                let mut db = self.db.write().unwrap();
                let key = self.hash_key(&id);

                match db.psyche.lookup(key) {
                    Some(lineage_id) => {
                        if db.psyche.free(lineage_id) {
                            Response::Ok(ResponseData::Ack)
                        } else {
                            Response::Error {
                                code: ErrorCode::LineageNotFound,
                                message: format!("Lineage '{}' already forgotten", id),
                            }
                        }
                    }
                    None => Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Lineage '{}' not found", id),
                    },
                }
            }

            Request::LineageTouch { id } => {
                let mut db = self.db.write().unwrap();
                let key = self.hash_key(&id);

                match db.psyche.lookup(key) {
                    Some(lineage_id) => match db.psyche.get_mut(lineage_id) {
                        Some(lineage) => {
                            lineage.touch();
                            Response::Ok(ResponseData::Ack)
                        }
                        None => Response::Error {
                            code: ErrorCode::LineageNotFound,
                            message: format!("Lineage '{}' not found", id),
                        },
                    },
                    None => Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Lineage '{}' not found", id),
                    },
                }
            }

            // ═══════════════════════════════════════════════════════════════
            // BOND OPERATIONS
            // ═══════════════════════════════════════════════════════════════
            Request::BondConnect {
                source,
                target,
                strength,
                polarity,
            } => {
                use crate::setun::Trit;

                let mut db = self.db.write().unwrap();
                let src_key = self.hash_key(&source);
                let tgt_key = self.hash_key(&target);

                let src_id = match db.psyche.lookup(src_key) {
                    Some(id) => id,
                    None => {
                        return Response::Error {
                            code: ErrorCode::LineageNotFound,
                            message: format!("Source lineage '{}' not found", source),
                        };
                    }
                };

                let tgt_id = match db.psyche.lookup(tgt_key) {
                    Some(id) => id,
                    None => {
                        return Response::Error {
                            code: ErrorCode::LineageNotFound,
                            message: format!("Target lineage '{}' not found", target),
                        };
                    }
                };

                let mut bond = Bond::new(src_id, tgt_id, strength);
                // Apply polarity
                bond.polarity = match polarity {
                    1 => Trit::True,    // Synergy
                    0 => Trit::Unknown, // Neutral
                    -1 => Trit::False,  // Antagonism
                    _ => Trit::True,    // Default to synergy
                };

                match db.bonds.connect(bond) {
                    Some(_) => Response::Ok(ResponseData::Ack),
                    None => Response::Error {
                        code: ErrorCode::Internal,
                        message: "Failed to create bond".into(),
                    },
                }
            }

            Request::BondReinforce {
                source,
                target,
                delta,
            } => {
                let mut db = self.db.write().unwrap();
                let src_key = self.hash_key(&source);
                let tgt_key = self.hash_key(&target);

                let src_id = match db.psyche.lookup(src_key) {
                    Some(id) => id,
                    None => {
                        return Response::Error {
                            code: ErrorCode::LineageNotFound,
                            message: format!("Source lineage '{}' not found", source),
                        };
                    }
                };

                let tgt_id = match db.psyche.lookup(tgt_key) {
                    Some(id) => id,
                    None => {
                        return Response::Error {
                            code: ErrorCode::LineageNotFound,
                            message: format!("Target lineage '{}' not found", target),
                        };
                    }
                };

                match db.bonds.find_bond(src_id, tgt_id) {
                    Some(bond_id) => {
                        if let Some(bond) = db.bonds.get_mut(bond_id) {
                            bond.reinforce(delta);
                            Response::Ok(ResponseData::Ack)
                        } else {
                            Response::Error {
                                code: ErrorCode::BondNotFound,
                                message: "Bond not found".into(),
                            }
                        }
                    }
                    None => Response::Error {
                        code: ErrorCode::BondNotFound,
                        message: format!("No bond between '{}' and '{}'", source, target),
                    },
                }
            }

            Request::BondSever { source, target } => {
                let mut db = self.db.write().unwrap();
                let src_key = self.hash_key(&source);
                let tgt_key = self.hash_key(&target);

                let src_id = match db.psyche.lookup(src_key) {
                    Some(id) => id,
                    None => {
                        return Response::Error {
                            code: ErrorCode::LineageNotFound,
                            message: format!("Source lineage '{}' not found", source),
                        };
                    }
                };

                let tgt_id = match db.psyche.lookup(tgt_key) {
                    Some(id) => id,
                    None => {
                        return Response::Error {
                            code: ErrorCode::LineageNotFound,
                            message: format!("Target lineage '{}' not found", target),
                        };
                    }
                };

                match db.bonds.find_bond(src_id, tgt_id) {
                    Some(bond_id) => {
                        db.bonds.disconnect(bond_id);
                        Response::Ok(ResponseData::Ack)
                    }
                    None => Response::Error {
                        code: ErrorCode::BondNotFound,
                        message: format!("No bond between '{}' and '{}'", source, target),
                    },
                }
            }

            Request::BondNeighbors { id } => {
                let db = self.db.read().unwrap();
                let key = self.hash_key(&id);

                match db.psyche.lookup(key) {
                    Some(lineage_id) => {
                        let neighbors: Vec<NeighborInfo> = db
                            .bonds
                            .neighbors_with_strength(lineage_id)
                            .map(|(neighbor_id, strength)| {
                                // TODO: Reverse lookup ID to string
                                // For now, use numeric ID as string
                                NeighborInfo {
                                    id: format!("lineage_{}", neighbor_id.0),
                                    bond_strength: strength,
                                    is_learned: false, // TODO: Track this
                                }
                            })
                            .collect();

                        Response::Ok(ResponseData::Neighbors(neighbors))
                    }
                    None => Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Lineage '{}' not found", id),
                    },
                }
            }

            // ═══════════════════════════════════════════════════════════════
            // QUERY OPERATIONS
            // ═══════════════════════════════════════════════════════════════
            Request::QueryConscious { min_energy } => {
                use crate::setun::Trit;

                let db = self.db.read().unwrap();

                // Use Cortex for ternary consciousness evaluation
                // Lucid (+1) or Dreaming (0) count as "aware"
                let lineages: Vec<LineageInfo> = db
                    .psyche
                    .iter()
                    .filter(|(_, l)| {
                        // First check energy threshold
                        if l.current_energy() < min_energy {
                            return false;
                        }

                        // Use Cortex to evaluate consciousness state
                        let state = db
                            .cortex
                            .consciousness_state(l.current_energy() as f64, l.threshold as f64);

                        // Accept Lucid (+1) and Dreaming (0), reject Dormant (-1)
                        state != Trit::False
                    })
                    .map(|(id, l)| {
                        // Calculate ternary state for response
                        let state = db
                            .cortex
                            .consciousness_state(l.current_energy() as f64, l.threshold as f64);

                        LineageInfo {
                            id: format!("lineage_{}", id.0),
                            energy: l.current_energy(),
                            threshold: l.threshold,
                            decay_rate: l.decay_rate,
                            rigidity: l.rigidity,
                            // True if Lucid (+1), false if Dreaming (0)
                            is_conscious: state == Trit::True,
                            last_access_ms: l.last_access / 1_000_000,
                        }
                    })
                    .collect();

                Response::Ok(ResponseData::Lineages(lineages))
            }

            Request::QueryTopK { k } => {
                let db = self.db.read().unwrap();
                let mut lineages: Vec<_> = db
                    .psyche
                    .iter()
                    .map(|(id, l)| (id, l.current_energy()))
                    .collect();

                lineages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                let top_k: Vec<LineageInfo> = lineages
                    .into_iter()
                    .take(k as usize)
                    .filter_map(|(id, _)| {
                        db.psyche.get(id).map(|l| LineageInfo {
                            id: format!("lineage_{}", id.0),
                            energy: l.current_energy(),
                            threshold: l.threshold,
                            decay_rate: l.decay_rate,
                            rigidity: l.rigidity,
                            is_conscious: l.is_conscious(),
                            last_access_ms: l.last_access / 1_000_000,
                        })
                    })
                    .collect();

                Response::Ok(ResponseData::Lineages(top_k))
            }

            Request::QueryTrauma { min_rigidity } => {
                let db = self.db.read().unwrap();
                let traumatized: Vec<LineageInfo> = db
                    .psyche
                    .iter()
                    .filter(|(_, l)| l.rigidity >= min_rigidity)
                    .map(|(id, l)| LineageInfo {
                        id: format!("lineage_{}", id.0),
                        energy: l.current_energy(),
                        threshold: l.threshold,
                        decay_rate: l.decay_rate,
                        rigidity: l.rigidity,
                        is_conscious: l.is_conscious(),
                        last_access_ms: l.last_access / 1_000_000,
                    })
                    .collect();

                Response::Ok(ResponseData::Lineages(traumatized))
            }

            Request::QueryPattern { pattern: _pattern } => {
                // TODO: Implement pattern matching
                Response::Error {
                    code: ErrorCode::Internal,
                    message: "Pattern query not yet implemented".into(),
                }
            }

            // ═══════════════════════════════════════════════════════════════
            // SYSTEM OPERATIONS
            // ═══════════════════════════════════════════════════════════════
            Request::Ping => Response::Ok(ResponseData::Pong),

            Request::Stats => {
                let db = self.db.read().unwrap();
                let stats = db
                    .psyche
                    .iter()
                    .fold((0usize, 0f32), |(conscious, energy), (_, l)| {
                        (
                            conscious + if l.is_conscious() { 1 } else { 0 },
                            energy + l.current_energy(),
                        )
                    });

                Response::Ok(ResponseData::Stats(StatsInfo {
                    lineage_count: db.psyche.len(),
                    bond_count: db.bonds.len(),
                    conscious_count: stats.0,
                    total_energy: stats.1,
                    is_frozen: self.is_frozen,
                    uptime_secs: self.start_time.elapsed().as_secs(),
                }))
            }

            Request::Snapshot { name } => {
                let db = self.db.read().unwrap();

                // Check if store is attached
                if let Some(ref store) = db.store {
                    use crate::persistence::snapshot::PhysicsSnapshot;

                    let physics = PhysicsSnapshot::default();

                    match store.take_snapshot(
                        Some(&name),
                        &db.psyche,
                        &db.strata,
                        &db.bonds,
                        Some(&db.cortex),
                        physics,
                    ) {
                        Ok(meta) => {
                            tracing::info!(
                                "📸 Snapshot '{}' saved ({} lineages, {} bonds)",
                                name,
                                meta.lineage_count,
                                meta.bond_count
                            );
                            Response::Ok(ResponseData::SnapshotCreated { name })
                        }
                        Err(e) => {
                            tracing::error!("Failed to save snapshot: {}", e);
                            Response::Error {
                                code: ErrorCode::Internal,
                                message: format!("Snapshot failed: {}", e),
                            }
                        }
                    }
                } else {
                    // No store attached - just ack
                    Response::Ok(ResponseData::SnapshotCreated { name })
                }
            }

            Request::Restore { name } => {
                let mut db = self.db.write().unwrap();

                if let Some(ref store) = db.store {
                    // Find snapshot by name
                    match store.get_snapshot_by_name(&name) {
                        Ok(Some(snapshot)) => {
                            // Restore arenas
                            match store.restore_snapshot(
                                &snapshot,
                                db.psyche.capacity(),
                                db.bonds.capacity(),
                                64,
                            ) {
                                Ok((psyche, strata, bonds, _physics)) => {
                                    db.psyche = psyche;
                                    db.strata = strata;
                                    db.bonds = bonds;

                                    // Restore Cortex if available
                                    if let Some(ref cortex_data) = snapshot.cortex_data {
                                        if let Ok(cortex) = bincode::deserialize(cortex_data) {
                                            db.cortex = cortex;
                                        }
                                    }

                                    tracing::info!(
                                        "🔄 Restored from snapshot '{}' ({} lineages)",
                                        name,
                                        db.psyche.len()
                                    );
                                    Response::Ok(ResponseData::Ack)
                                }
                                Err(e) => Response::Error {
                                    code: ErrorCode::Internal,
                                    message: format!("Restore failed: {}", e),
                                },
                            }
                        }
                        Ok(None) => Response::Error {
                            code: ErrorCode::SnapshotNotFound,
                            message: format!("Snapshot '{}' not found", name),
                        },
                        Err(e) => Response::Error {
                            code: ErrorCode::Internal,
                            message: format!("Restore error: {}", e),
                        },
                    }
                } else {
                    Response::Error {
                        code: ErrorCode::Internal,
                        message: "No storage attached".into(),
                    }
                }
            }

            Request::Freeze { frozen } => {
                self.is_frozen = frozen;
                Response::Ok(ResponseData::Ack)
            }

            Request::PhysicsTune {
                param: _param,
                value: _value,
            } => {
                // TODO: Implement physics tuning
                Response::Ok(ResponseData::Ack)
            }

            Request::MoodSet { mood } => {
                let mut db = self.db.write().unwrap();
                db.cortex.set_mood(mood as f64);
                Response::Ok(ResponseData::Ack)
            }

            // ═══════════════════════════════════════════════════════════════
            // STREAM OPERATIONS
            // ═══════════════════════════════════════════════════════════════
            Request::Subscribe {
                events_mask: _events_mask,
            } => {
                // TODO: Implement event subscription
                Response::Ok(ResponseData::Ack)
            }

            Request::Unsubscribe => {
                // TODO: Implement unsubscribe
                Response::Ok(ResponseData::Ack)
            }
        }
    }

    /// Hash a string key using DefaultHasher
    fn hash_key(&self, key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_handler() -> CommandHandler {
        let db = Arc::new(RwLock::new(MindFry::new()));
        CommandHandler::new(db)
    }

    #[test]
    fn test_ping() {
        let mut handler = setup_handler();
        let response = handler.handle(Request::Ping);

        match response {
            Response::Ok(ResponseData::Pong) => {}
            _ => panic!("Expected Pong"),
        }
    }

    #[test]
    fn test_lineage_create_and_get() {
        let mut handler = setup_handler();

        // Create
        let response = handler.handle(Request::LineageCreate {
            id: "test".into(),
            energy: 0.8,
            threshold: 0.5,
            decay_rate: 0.001,
        });
        assert!(matches!(response, Response::Ok(ResponseData::Ack)));

        // Get
        let response = handler.handle(Request::LineageGet {
            id: "test".into(),
            flags: 0,
        });
        match response {
            Response::Ok(ResponseData::Lineage(info)) => {
                assert_eq!(info.id, "test");
                assert!(info.energy > 0.7);
            }
            _ => panic!("Expected Lineage"),
        }
    }

    #[test]
    fn test_lineage_not_found() {
        let mut handler = setup_handler();
        let response = handler.handle(Request::LineageGet {
            id: "nonexistent".into(),
            flags: 0,
        });

        match response {
            Response::Error { code, .. } => {
                assert_eq!(code, ErrorCode::LineageNotFound);
            }
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_stats() {
        let mut handler = setup_handler();

        // Create some lineages
        handler.handle(Request::LineageCreate {
            id: "a".into(),
            energy: 1.0,
            threshold: 0.5,
            decay_rate: 0.001,
        });
        handler.handle(Request::LineageCreate {
            id: "b".into(),
            energy: 0.3,
            threshold: 0.5,
            decay_rate: 0.001,
        });

        let response = handler.handle(Request::Stats);
        match response {
            Response::Ok(ResponseData::Stats(stats)) => {
                assert_eq!(stats.lineage_count, 2);
                assert_eq!(stats.conscious_count, 1); // Only "a" is conscious
            }
            _ => panic!("Expected Stats"),
        }
    }
}
