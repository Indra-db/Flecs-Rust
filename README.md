# Flecs ECS Bridge :: Rust Wrapper for Flecs

Hello there! Welcome to my Rust wrapper around the ECS library called [Flecs](https://github.com/SanderMertens/flecs). This project endeavors to offer Rust developers an intuitive and streamlined interface to harness the full potential of Flecs.

## ⚠️ Status: Early Stages

Please note that this wrapper is currently in its **early stages**. It's heavily under work in progress and is **not recommended for consumption**. While you're encouraged to explore, test, and provide feedback, please be aware of potential bugs and incomplete features.

This library has not been advertized nor published yet to crates.io for that reason.

### The progress

#### core
```[tasklist]
- [x] id
- [x] entity_view
- [x] entity
- [x] component
- [x] 4 byte sized enum component
- [ ] rust enum variant component (wip changes to flecs)
- [x] component id registration
- [x] enum constants registration
- [x] ref component
- [x] lifecycle_traits (which adds support to hold dynamic memory in components, e.g. vector, string, etc.)
- [x] table + table_range
- [x] c_types
- [x] type (archetype)
- [x] world + scoped_world
- [x] term
- [x] filter
    - [x] create filter & loop over components
    - [x] add support for optional components
    - [x] filter builder
- [ ] query
```

#### non-core
```[tasklist]
- [ ] event
- [ ] observer
```
#### addons
```[tasklist]
- [ ] systems
- [ ] alerts
- [ ] app
- [ ] doc
- [ ] doc
- [ ] json
- [ ] meta
- [ ] modules
- [ ] monitor
- [ ] pipelines
- [ ] plecs
- [ ] rest
- [ ] rules
- [ ] snapshots
- [ ] timer
- [ ] units
- [ ] metrics
- [ ] logging
    - [x] leveling, colors, timestamp, timedelta
    - [ ] log with level
```

#### future plans
```[tasklist]
- [ ] make the API rustier, no more pointers.
- [ ] use custom string type to avoid re-allocating C allocated strings instead of using String with new allocation
- [x] add a non-heap allocating way of creating filters. This is useful for creating filters regularly in hot path code, e.g. systems. 
```

## The Aim

The plan is to match feature parity of the C++ API, starting with the core library while also being fully documented and tested.

## Contributions

If you're excited about this project and would like to contribute, or if you've found any bugs, please feel free to raise an issue or submit a pull request. We'd love to have your involvement!

## License

This wrapper is licensed under the [MIT License](LICENSE).

## Acknowledgements

A big shoutout to [Sander Mertens](https://github.com/SanderMertens) and all contributors to Flecs for their excellent work on the original library. This project wouldn't be possible without them.


---

**Thank you for stopping by, and stay tuned for updates as we progress on this exciting journey to bridge Rust and Flecs!
es.io for that reason.**
