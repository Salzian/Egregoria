use crate::map::{LaneKind, Map, ParkingSpot, ParkingSpotID, ParkingSpots};
use common::PtrCmp;
use geom::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};
use std::option::Option::None;

#[derive(Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct SpotReservation(ParkingSpotID);

#[derive(Default, Serialize, Deserialize)]
pub struct ParkingManagement {
    reserved_spots: BTreeSet<ParkingSpotID>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ParkingReserveError {
    FindingNearestLane,
    FetchingLaneData,
    NoSpotFoundAfterSearch,
}

impl ParkingManagement {
    #[allow(unknown_lints)]
    #[allow(clippy::forget_non_drop)]
    pub fn free(&mut self, spot: SpotReservation) {
        if !self.reserved_spots.remove(&spot.0) {
            log::warn!("{:?} wasn't reserved", spot.0);
        }
        std::mem::forget(spot);
    }

    pub fn is_free(&self, spot: SpotReservation) -> bool {
        self.is_spot_free(spot.0)
    }

    pub fn is_spot_free(&self, spot: ParkingSpotID) -> bool {
        !self.reserved_spots.contains(&spot)
    }

    pub fn reserve_near(
        &mut self,
        near: Vec3,
        map: &Map,
    ) -> Result<SpotReservation, ParkingReserveError> {
        use ParkingReserveError as E;
        let lane = map
            .nearest_lane(near, LaneKind::Driving, None)
            .ok_or(E::FindingNearestLane)?;
        let lane = map.lanes().get(lane).ok_or(E::FetchingLaneData)?;

        let depth = 7;

        let mut potential = HashSet::new();
        potential.insert(PtrCmp(lane));
        let mut next = HashSet::new();
        let intersections = map.intersections();
        let roads = map.roads();
        for _ in 0..depth {
            for lane in potential.drain() {
                let lane = lane.0;

                let inter_dst = unwrap_or!(intersections.get(lane.dst), continue);
                let inter_src = unwrap_or!(intersections.get(lane.src), continue);

                next.extend(
                    inter_dst
                        .turns_from(lane.id)
                        .flat_map(|(turn, _)| Some(PtrCmp(map.lanes().get(turn.dst)?))),
                );

                next.extend(
                    inter_src
                        .turns_to(lane.id)
                        .flat_map(|(turn, _)| Some(PtrCmp(map.lanes().get(turn.src)?))),
                );

                let parent = unwrap_or!(roads.get(lane.parent), continue);
                let plane = unwrap_or!(parent.parking_next_to(lane), continue);

                if let Some(p_iter) = map.parking.closest_spots(plane, near) {
                    for spot in p_iter {
                        if self.reserved_spots.insert(spot) {
                            return Ok(SpotReservation(spot));
                        }
                    }
                }
            }
            std::mem::swap(&mut potential, &mut next);
        }
        Err(E::NoSpotFoundAfterSearch)
    }
}

impl SpotReservation {
    pub fn exists(&self, spots: &ParkingSpots) -> bool {
        spots.contains(self.0)
    }

    pub fn get<'a>(&self, spots: &'a ParkingSpots) -> Option<&'a ParkingSpot> {
        spots.get(self.0)
    }

    pub fn park_pos(&self, map: &Map) -> Option<Vec3> {
        map.parking_to_drive_pos(self.0)
    }
}
