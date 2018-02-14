pub mod parse;
pub mod grid;

use std::ops::{Add, Sub};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::sync::RwLock;

use serde_json;

use arch::grid::Grid;

use routing::Path;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Location {
    pub y: i32,
    pub x: i32,
}

impl Location {
    pub fn distance_to(&self, other: &Self) -> u32 {
        (self - other).norm()
    }
    pub fn norm(&self) -> u32 {
        (self.y.abs() + self.x.abs()) as u32
    }

    pub fn from_index(i: u32, width: u32) -> Location {
        Location {
            y: (i / width) as i32,
            x: (i % width) as i32,
        }
    }
}

impl<'a> Add for &'a Location {
    type Output = Location;
    fn add(self, other: &Location) -> Location {
        Location {
            y: self.y + other.y,
            x: self.x + other.x,
        }
    }
}

impl<'a> Sub for &'a Location {
    type Output = Location;
    fn sub(self, other: &Location) -> Location {
        Location {
            y: self.y - other.y,
            x: self.x - other.x,
        }
    }
}

pub type DropletId = usize;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Droplet {
    pub location: Location,
    pub shape: HashSet<Location>,
    // TODO should droplets really know about their destinations?
    pub destination: Option<Location>,
    pub collision_group: usize,
}

// Factory for generating default shapes for droplets
fn default_shape_factory() -> HashSet<Location> {
    [Location{ x: 0, y: 0 }].iter().cloned().collect()
}

impl Droplet {
    fn from_location(location: Location) -> Droplet {
        Droplet {
            location: location,
            shape: default_shape_factory(),
            destination: None,
            collision_group: 0,
        }
    }

    #[allow(dead_code)]
    fn from_location_and_shape(location: Location, shape: HashSet<Location>) -> Droplet {
        if !shape.contains(&Location{ x: 0, y: 0}) {
            panic!("Invalid shape for droplet");
        }
        // TODO: check that the droplet is contiguous
        // TODO: use Grid
        Droplet {
            location: location,
            shape: shape,
            destination: None,
            collision_group: 0,
        }
    }

    pub fn info(&self) -> DropletInfo {
        DropletInfo {
            location: self.location,
            volume: 1,
            shape: self.shape.iter().cloned().collect(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize)]
pub struct DropletInfo {
    location: Location,
    volume: i32,
    shape: Vec<Location>,
}

#[derive(Debug, Clone)]
pub struct Architecture {
    pub grid: Grid,
    pub droplets: HashMap<DropletId, Droplet>,
    next_droplet_id: DropletId,
    next_collision_group: usize,
}

impl Architecture {
    pub fn from_grid(grid: Grid) -> Architecture {
        Architecture {
            grid: grid,
            droplets: HashMap::new(),
            next_droplet_id: 0,
            next_collision_group: 0,
        }
    }

    pub fn from_reader<R: Read>(reader: R) -> Architecture {
        let grid = serde_json::from_reader(reader).expect("parse failed");
        Architecture::from_grid(grid)
    }

    pub fn new_droplet_id(&mut self) -> DropletId {
        let id = self.next_droplet_id;
        self.next_droplet_id += 1;
        id
    }

    fn new_collision_group(&mut self) -> usize {
        let cg = self.next_collision_group;
        self.next_collision_group += 1;
        cg
    }

    pub fn droplet_from_location(&mut self, location: Location) -> Droplet {
        let mut droplet = Droplet::from_location(location);
        droplet.collision_group = self.new_collision_group();
        droplet
    }

    pub fn instantiate_droplet(&mut self, id: DropletId, droplet: Droplet) {
        assert!(id < self.next_droplet_id);

        let was_not_present = self.droplets.insert(id, droplet).is_none();
        assert!(was_not_present);

        // self.droplets.get_mut(id).unwrap()
    }

    pub fn get_collision(&self) -> Option<(DropletId, DropletId)> {
        for (id1, droplet1) in self.droplets.iter() {
            for (id2, droplet2) in self.droplets.iter() {
                if id1 == id2 {
                    continue;
                }
                if droplet1.collision_group == droplet2.collision_group {
                    continue;
                }

                let collide = self.grid.neighbors_shape(&droplet1.location, &droplet1.shape)
                    .into_iter()
                    // TODO this check will be more complicated when there are
                    // droplet shapes
                    .any(|loc| self.grid.neighbors_shape(&droplet2.location, &droplet2.shape).contains(&loc));

                if collide {
                    return Some((*id1, *id2));
                }
            }
        }
        None
    }

    pub fn get_destination_collision(&self) -> Option<(DropletId, DropletId)> {
        for (id1, droplet1) in self.droplets.iter() {
            for (id2, droplet2) in self.droplets.iter() {
                if id1 == id2 {
                    continue;
                }
                if droplet1.collision_group == droplet2.collision_group {
                    continue;
                }

                if droplet1.destination.is_none() || droplet2.destination.is_none() {
                    continue;
                }

                let dest1 = droplet1.destination.unwrap();
                let dest2 = droplet2.destination.unwrap();

                let collide = self.grid.neighbors9(&dest1)
                    .into_iter()
                // TODO this check will be more complicated when there are
                // droplet shapes
                    .any(|loc| loc == dest2);

                if collide {
                    return Some((*id1, *id2));
                }
            }
        }
        None
    }

    pub fn take_paths<F>(lock: &RwLock<Self>, paths: HashMap<DropletId, Path>, callback: F)
    where
        F: Fn(),
    {

        use std::mem::drop;
        let arch = lock.write().unwrap();

        #[cfg(test)]
        for (id, path) in paths.iter() {
            use routing::tests::check_path_on_grid;
            let d = &arch.droplets[id];
            check_path_on_grid(d, path, &arch.grid);
        }
        // println!("paths: {:?}", paths);

        drop(arch);

        let max_len = paths.values().map(|p| p.len()).max().unwrap_or(0);
        for i in 0..max_len {
            callback();
            let mut arch = lock.write().unwrap();
            for (id, path) in paths.iter() {
                let mut d = arch.droplets.get_mut(id).unwrap();
                if i < path.len() {
                    d.location = path[i];
                }
            }
            let coll = arch.get_collision();
            if coll.is_some() {
                let (id1, id2) = coll.unwrap();
                panic!(
                    "Paths: {:?}\n Collision:\n  {:?} {:?}\n  {:?} {:?}",
                    paths,
                    id1,
                    arch.droplets[&id1],
                    id2,
                    arch.droplets[&id2]
                );
            }
            // assert!(arch.get_collision().is_none())
        }

    }
}



#[cfg(test)]
pub mod tests {

    use super::*;

    use proptest::prelude::*;
    use proptest::collection::hash_map;
    use proptest::sample::select;

    use std::ops::Range;

    prop_compose! {
        fn arb_droplet(locations: Vec<Location>)
            (loc  in select(locations.clone()),
             dest in select(locations),
             cg in prop::num::usize::ANY)
            -> Droplet
        {
            Droplet {
                location: loc,
                shape: default_shape_factory(),
                destination: Some(dest),
                collision_group: cg,
            }
        }
    }

    pub fn arb_arch_from_grid(grid: Grid, n_droplets: Range<usize>) -> BoxedStrategy<Architecture> {
        let ht_gen = hash_map(
            prop::num::usize::ANY,
            arb_droplet(grid.locations().map(|(loc, _)| loc).collect()),
            n_droplets,
        );
        // can't use prop_compose! because we need to move the hash map here
        ht_gen
            .prop_map(move |ht| {

                let next_id = ht.keys().max().map_or(0, |max| max + 1);
                let next_cg = ht.values().map(|d| d.collision_group).max().map_or(
                    0,
                    |max| max + 1,
                );
                Architecture {
                    grid: grid.clone(),
                    next_droplet_id: next_id,
                    next_collision_group: next_cg,
                    droplets: ht,
                }
            })
            .boxed()
    }
}
