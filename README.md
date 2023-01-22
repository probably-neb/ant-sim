# ANT SIM
![LOC](https://tokei.rs/b1/github/probably-neb/ant-sim?category=code)

This project is a personal exploration into ant simulations and the state of game engines in the rust programming language. It was heavily inspired by the excellent video by [Sebastian Lague](https://www.youtube.com/watch?v=X-iSQQgOd1A) that you should definitely check out (even if you don't think you're interested). 

The game engine used is bevy with the help of multiple plugins.

The choice of bevy for this project was intentional as it has excellent documentation on compilation into wasm binaries and the ability to run the simulation in the browser was and still is a primary goal of the project. This process proved to be quite painless and you can check it out [here](https://nebsite.website/ant_sim/bin/ant_sim.html) if you're interested. Do be warned however that while the performance is surprisingly good even on the few mobile devices I've tried, the binary file is quite large and does take a while to download even on fast internet connections.

### Network Mode
While I do have ideas for multiple simulation "modes" (see "wander" in the todo list below) currently there is only a mode referred to simply as "network". Network mode is an exploration into simulating the retrieval of data in peer-to-peer networks, where the destination or a route to the destination of the desired data is not known. The method for retrieving the data is taken from [this paper](https://www.researchgate.net/publication/220109707_Biology-Inspired_Optimizations_of_Peer-to-Peer_Overlay_Networks) with additional implementation details taken from the aforementioned Sebastian Lague video. It is inspired by the usage of pheromones by ants to guide future ants to food and other resources. A much deeper explanation into the network modes inner workings can be found [on my website](https://nebsite.website/ant_sim/ant_sim.html).

### Todo/Possible ideas list:
```yaml
performance:
  - entity (pheromone & ant) pools
  - further reducing of binary size
common:
  - don't just restart ants on same path
  - make sure ant's can't move through nests hitbox in one step
  - move from 'quick' egui ui to hand-made version:
      - make consts into resources that can be inspected and modified in egui:
        - give net plugin fields that act as settings:
          - plugin creates resource with those settings
          - nest coord gen type (hex / fib) [ can also add system that checks if plugin fields != resource and reloads ]
  - shader (not compute) for pheromones:
      problems:
        - how to keep track of multiple pheromone strengths at the same time:
          - could keep hashmap with strength info:
                - interesting to see if this would help or hurt
                - basically bevy component overhead vs. pixel by pixel hash map lookups
          - could still use grid system to reduce hmap size
          - would allow for easier fading
system:
  - pheromones:
      - initialize "lines" between nests at startup:
        - could hold pheromone, distance, etc info
        - ants use lines to guide paths (therefore removing wandering logic)
        - pheromones are initialized along paths at startup
        - need to figure out way to have paths w/ multiple colors:
          - current pathing alows parallel paths which is a nice effect
          - could create many parallel paths
        - all lines could be entities whose texture is repeating dots:
          - ants create new line whenever pathfinding
          - anchor is start of line
          - every "dot" length line length is increased
          - problems:
            - wouldn't overlap -> same colors fade in parallel
            - potentially hard to have line extend in real time as ant walks
      - compute shader:
        - doesn't work with wasm?
  - consider making jump to home/origin 1 step instead of pathfinding back (in theory more realistic because requests keeping track of where they came from is trivial)
  - consider clearing recent list when ant reaches target so recency has no impact on where ant goes after reaching target
  - ant pool:
    - convert nest/ant logic to despawn (return to ant pool) ants upon completion of mission
      allowing other nests to use them in new tasks
    - options:
        A: initialize `NUM_ANTS` on startup
        B: initialize ants on startup and only add to pool when despawned
  - decision weights:
    - monitoring tool to monitor and graph likelyhood of correct choices vs time to verify decision weights are effective
    - distance:
      - max travel distance?
      - prefer multiple short jumps over one long jump (to a point)?
wander:
  - clean up and create plugin like network model
  - use ant pool
  - convert logic to have one primary nest while other nests only provide food
```

A personal goal for this project was to practice iterative, data-driven performance optimizations as it is not a skill I have much experience in. You can see the current progress of this on the `test-data` branch.

