use std::collections::HashMap;

use noise::{NoiseFn, Perlin};

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;
pub const CHUNK_VOLUME: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT;
pub const START_CHUNK_RADIUS: i32 = 2;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum BlockId {
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
}

impl BlockId {
    fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::Grass,
            2 => Self::Dirt,
            3 => Self::Stone,
            _ => Self::Air,
        }
    }
}

pub struct Chunk {
    blocks: Vec<u8>,
}

impl Chunk {
    pub fn empty() -> Self {
        Self {
            blocks: vec![BlockId::Air as u8; CHUNK_VOLUME],
        }
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        (y * CHUNK_WIDTH + z) * CHUNK_WIDTH + x
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        if x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || z >= CHUNK_WIDTH {
            return BlockId::Air;
        }
        BlockId::from_byte(self.blocks[Self::index(x, y, z)])
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_WIDTH {
            self.blocks[Self::index(x, y, z)] = block as u8;
        }
    }
}

pub struct World {
    seed: u32,
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn empty(seed: u32) -> Self {
        Self {
            seed,
            chunks: HashMap::with_capacity((START_CHUNK_RADIUS * 2).pow(2) as usize),
        }
    }

    pub fn generate(seed: u32) -> Self {
        let perlin = Perlin::new(seed);
        let mut world = Self::empty(seed);

        for chunk_x in -START_CHUNK_RADIUS..START_CHUNK_RADIUS {
            for chunk_z in -START_CHUNK_RADIUS..START_CHUNK_RADIUS {
                let position = ChunkPos {
                    x: chunk_x,
                    z: chunk_z,
                };
                let mut chunk = Chunk::empty();

                for local_x in 0..CHUNK_WIDTH {
                    for local_z in 0..CHUNK_WIDTH {
                        let world_x = chunk_x * CHUNK_WIDTH as i32 + local_x as i32;
                        let world_z = chunk_z * CHUNK_WIDTH as i32 + local_z as i32;
                        let noise = perlin.get([world_x as f64 / 40.0, world_z as f64 / 40.0]);
                        let surface = (36.0 + noise * 12.0).round().clamp(12.0, 60.0) as usize;

                        for y in 0..=surface {
                            let block = if y == surface {
                                BlockId::Grass
                            } else if y + 3 >= surface {
                                BlockId::Dirt
                            } else {
                                BlockId::Stone
                            };
                            chunk.set(local_x, y, local_z, block);
                        }
                    }
                }

                world.chunks.insert(position, chunk);
            }
        }

        world
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn surface_height(&self, x: i32, z: i32) -> Option<i32> {
        (0..CHUNK_HEIGHT as i32)
            .rev()
            .find(|&y| self.get_block(x, y, z) != BlockId::Air)
            .map(|y| y + 1)
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockId {
        if !(0..CHUNK_HEIGHT as i32).contains(&y) {
            return BlockId::Air;
        }

        let chunk_position = ChunkPos {
            x: x.div_euclid(CHUNK_WIDTH as i32),
            z: z.div_euclid(CHUNK_WIDTH as i32),
        };
        self.chunks
            .get(&chunk_position)
            .map_or(BlockId::Air, |chunk| {
                chunk.get(
                    x.rem_euclid(CHUNK_WIDTH as i32) as usize,
                    y as usize,
                    z.rem_euclid(CHUNK_WIDTH as i32) as usize,
                )
            })
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockId) {
        if !(0..CHUNK_HEIGHT as i32).contains(&y) {
            return;
        }

        let chunk_position = ChunkPos {
            x: x.div_euclid(CHUNK_WIDTH as i32),
            z: z.div_euclid(CHUNK_WIDTH as i32),
        };
        let chunk = self
            .chunks
            .entry(chunk_position)
            .or_insert_with(Chunk::empty);
        chunk.set(
            x.rem_euclid(CHUNK_WIDTH as i32) as usize,
            y as usize,
            z.rem_euclid(CHUNK_WIDTH as i32) as usize,
            block,
        );
    }

    pub fn for_each_non_air(&self, mut visit: impl FnMut(i32, i32, i32, BlockId)) {
        for (chunk_position, chunk) in &self.chunks {
            let origin_x = chunk_position.x * CHUNK_WIDTH as i32;
            let origin_z = chunk_position.z * CHUNK_WIDTH as i32;

            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_WIDTH {
                    for x in 0..CHUNK_WIDTH {
                        let block = chunk.get(x, y, z);
                        if block != BlockId::Air {
                            visit(origin_x + x as i32, y as i32, origin_z + z as i32, block);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockId, Chunk, ChunkPos, World, CHUNK_HEIGHT, CHUNK_VOLUME, CHUNK_WIDTH};

    #[test]
    fn chunk_storage_is_compact_and_bounds_checked() {
        let mut chunk = Chunk::empty();
        assert_eq!(chunk.blocks.len(), CHUNK_VOLUME);
        assert_eq!(chunk.get(0, 0, 0), BlockId::Air);

        chunk.set(15, 255, 15, BlockId::Stone);
        assert_eq!(chunk.get(15, 255, 15), BlockId::Stone);
        assert_eq!(chunk.get(CHUNK_WIDTH, 0, 0), BlockId::Air);
        assert_eq!(chunk.get(0, CHUNK_HEIGHT, 0), BlockId::Air);
    }

    #[test]
    fn world_coordinates_cross_chunk_boundaries() {
        let mut world = World::generate(12345);
        world.set_block(15, 200, 15, BlockId::Stone);
        world.set_block(16, 200, 16, BlockId::Dirt);

        assert_eq!(world.get_block(15, 200, 15), BlockId::Stone);
        assert_eq!(world.get_block(16, 200, 16), BlockId::Dirt);
        assert_eq!(world.get_block(-1, 200, -1), BlockId::Air);
        assert!(world.chunks.contains_key(&ChunkPos { x: 0, z: 0 }));
        assert!(world.chunks.contains_key(&ChunkPos { x: 1, z: 1 }));
    }

    #[test]
    fn terrain_generation_is_seeded_and_has_multiple_materials() {
        let first = World::generate(9876);
        let second = World::generate(9876);
        assert_eq!(first.seed(), second.seed());

        let mut saw_grass = false;
        let mut saw_dirt = false;
        let mut saw_stone = false;
        first.for_each_non_air(|x, y, z, block| {
            assert_eq!(first.get_block(x, y, z), block);
            saw_grass |= block == BlockId::Grass;
            saw_dirt |= block == BlockId::Dirt;
            saw_stone |= block == BlockId::Stone;
        });
        assert!(saw_grass && saw_dirt && saw_stone);
    }

    #[test]
    fn generated_surface_is_inside_chunk_height() {
        let world = World::generate(7);
        let surface = world.surface_height(0, 0).unwrap();

        assert!(surface > 0);
        assert!(surface < CHUNK_HEIGHT as i32);
        assert_eq!(world.get_block(0, surface - 1, 0), BlockId::Grass);
    }
}
