use conways_game_of_life::game::{Config, World};
use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};

fn bench_tick(c: &mut Criterion) {
    let config = Config {
        frames: 30,
        col: 100,
        row: 100,
    };

    c.bench_function("World::tick 100x100", |b| {
        b.iter_batched(
            || {
                let mut world = World::init_world(config.clone());
                let _ = world.generate_random();
                world
            },
            |mut world| {
                world.tick();
                black_box(world);
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_tick);
criterion_main!(benches);
