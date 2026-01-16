//! MFBP Command Handler
//!
//! Executes requests against the MindFry database.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::arena::Lineage;
use crate::graph::Bond;
use crate::MindFry;

use super::message::*;
use super::Request;

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

            Request::LineageCreate { id, energy, threshold, decay_rate } => {
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

            Request::LineageGet { id } => {
                let db = self.db.read().unwrap();
                let key = self.hash_key(&id);

                match db.psyche.lookup(key) {
                    Some(lineage_id) => {
                        match db.psyche.get(lineage_id) {
                            Some(lineage) => {
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
                        }
                    }
                    None => Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Lineage '{}' not found", id),
                    },
                }
            }

            Request::LineageStimulate { id, delta } => {
                let mut db = self.db.write().unwrap();
                let key = self.hash_key(&id);

                match db.psyche.lookup(key) {
                    Some(lineage_id) => {
                        match db.psyche.get_mut(lineage_id) {
                            Some(lineage) => {
                                lineage.stimulate(delta);
                                Response::Ok(ResponseData::Ack)
                            }
                            None => Response::Error {
                                code: ErrorCode::LineageNotFound,
                                message: format!("Lineage '{}' not found", id),
                            },
                        }
                    }
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
                    Some(lineage_id) => {
                        match db.psyche.get_mut(lineage_id) {
                            Some(lineage) => {
                                lineage.touch();
                                Response::Ok(ResponseData::Ack)
                            }
                            None => Response::Error {
                                code: ErrorCode::LineageNotFound,
                                message: format!("Lineage '{}' not found", id),
                            },
                        }
                    }
                    None => Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Lineage '{}' not found", id),
                    },
                }
            }

            // ═══════════════════════════════════════════════════════════════
            // BOND OPERATIONS
            // ═══════════════════════════════════════════════════════════════

            Request::BondConnect { source, target, strength } => {
                let mut db = self.db.write().unwrap();
                let src_key = self.hash_key(&source);
                let tgt_key = self.hash_key(&target);

                let src_id = match db.psyche.lookup(src_key) {
                    Some(id) => id,
                    None => return Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Source lineage '{}' not found", source),
                    },
                };

                let tgt_id = match db.psyche.lookup(tgt_key) {
                    Some(id) => id,
                    None => return Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Target lineage '{}' not found", target),
                    },
                };

                let bond = Bond::new(src_id, tgt_id, strength);
                match db.bonds.connect(bond) {
                    Some(_) => Response::Ok(ResponseData::Ack),
                    None => Response::Error {
                        code: ErrorCode::Internal,
                        message: "Failed to create bond".into(),
                    },
                }
            }

            Request::BondReinforce { source, target, delta } => {
                let mut db = self.db.write().unwrap();
                let src_key = self.hash_key(&source);
                let tgt_key = self.hash_key(&target);

                let src_id = match db.psyche.lookup(src_key) {
                    Some(id) => id,
                    None => return Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Source lineage '{}' not found", source),
                    },
                };

                let tgt_id = match db.psyche.lookup(tgt_key) {
                    Some(id) => id,
                    None => return Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Target lineage '{}' not found", target),
                    },
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
                    None => return Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Source lineage '{}' not found", source),
                    },
                };

                let tgt_id = match db.psyche.lookup(tgt_key) {
                    Some(id) => id,
                    None => return Response::Error {
                        code: ErrorCode::LineageNotFound,
                        message: format!("Target lineage '{}' not found", target),
                    },
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
                        let neighbors: Vec<NeighborInfo> = db.bonds
                            .neighbors_with_strength(lineage_id)
                            .filter_map(|(neighbor_id, strength)| {
                                // TODO: Reverse lookup ID to string
                                // For now, use numeric ID as string
                                Some(NeighborInfo {
                                    id: format!("lineage_{}", neighbor_id.0),
                                    bond_strength: strength,
                                    is_learned: false, // TODO: Track this
                                })
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
                let db = self.db.read().unwrap();
                let lineages: Vec<LineageInfo> = db.psyche
                    .iter()
                    .filter(|(_, l)| l.is_conscious() && l.current_energy() >= min_energy)
                    .map(|(id, l)| LineageInfo {
                        id: format!("lineage_{}", id.0),
                        energy: l.current_energy(),
                        threshold: l.threshold,
                        decay_rate: l.decay_rate,
                        rigidity: l.rigidity,
                        is_conscious: true,
                        last_access_ms: l.last_access / 1_000_000,
                    })
                    .collect();

                Response::Ok(ResponseData::Lineages(lineages))
            }

            Request::QueryTopK { k } => {
                let db = self.db.read().unwrap();
                let mut lineages: Vec<_> = db.psyche
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
                let traumatized: Vec<LineageInfo> = db.psyche
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
                let stats = db.psyche.iter().fold(
                    (0usize, 0f32),
                    |(conscious, energy), (_, l)| {
                        (
                            conscious + if l.is_conscious() { 1 } else { 0 },
                            energy + l.current_energy(),
                        )
                    },
                );

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
                // TODO: Implement snapshot with sled
                Response::Ok(ResponseData::SnapshotCreated { name })
            }

            Request::Restore { name } => {
                // TODO: Implement restore with sled
                Response::Error {
                    code: ErrorCode::SnapshotNotFound,
                    message: format!("Snapshot '{}' not found", name),
                }
            }

            Request::Freeze { frozen } => {
                self.is_frozen = frozen;
                Response::Ok(ResponseData::Ack)
            }

            Request::PhysicsTune { param: _param, value: _value } => {
                // TODO: Implement physics tuning
                Response::Ok(ResponseData::Ack)
            }

            // ═══════════════════════════════════════════════════════════════
            // STREAM OPERATIONS
            // ═══════════════════════════════════════════════════════════════

            Request::Subscribe { events_mask: _events_mask } => {
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
        let response = handler.handle(Request::LineageGet { id: "test".into() });
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
        let response = handler.handle(Request::LineageGet { id: "nonexistent".into() });

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
