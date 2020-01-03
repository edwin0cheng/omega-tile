# omega-tile

#### WORK IN PROGRESS

## Omega Tile Generator

#### Clean Cache
cargo run --release -- clean

#### Generate tileset in out directory
cargo run --release -- build imgs/grass.png 256 --seed 102 --variation v16

#### Generate testset with numbers in out directory 
 cargo run --release -- test-set 256 --seed 102 --variation v16 --number