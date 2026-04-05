//! Benchmarks for v1.5.0 modules — quantization, evaluation, scheduler, multi-vault

use ai_model_vault::{
    BackupFrequency, BackupManager, EvalStore, MetricResult, QuantMethod, QuantProfile,
    QuantProfileStore, VaultRegistry,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn bench_quant_profile_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant_profile_store");

    group.bench_function("set_profile", |b| {
        b.iter_with_setup(
            || {
                let tmp = tempdir().unwrap();
                let store = QuantProfileStore::new(tmp.path()).unwrap();
                (tmp, store)
            },
            |(_tmp, store)| {
                store
                    .set(black_box(QuantProfile {
                        name: "test".into(),
                        method: QuantMethod::Q4KM,
                        description: Some("bench".into()),
                    }))
                    .unwrap();
            },
        );
    });

    group.bench_function("list_profiles", |b| {
        b.iter_with_setup(
            || {
                let tmp = tempdir().unwrap();
                let store = QuantProfileStore::new(tmp.path()).unwrap();
                for i in 0..10 {
                    store
                        .set(QuantProfile {
                            name: format!("profile-{i}"),
                            method: QuantMethod::Q4KM,
                            description: None,
                        })
                        .unwrap();
                }
                (tmp, store)
            },
            |(_tmp, store)| {
                black_box(store.list().unwrap());
            },
        );
    });

    group.bench_function("estimate_size", |b| {
        b.iter(|| {
            black_box(ai_model_vault::estimate_quantized_size(
                1_000_000_000,
                &QuantMethod::F32,
                &QuantMethod::Q4KM,
            ))
        });
    });

    group.finish();
}

fn bench_eval_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("eval_store");

    group.bench_function("record_run", |b| {
        b.iter_with_setup(
            || {
                let tmp = tempdir().unwrap();
                let store = EvalStore::new(tmp.path()).unwrap();
                (tmp, store)
            },
            |(_tmp, store)| {
                let metrics = vec![
                    MetricResult {
                        name: "accuracy".into(),
                        value: 0.85,
                        unit: "score".into(),
                    },
                    MetricResult {
                        name: "f1".into(),
                        value: 0.82,
                        unit: "score".into(),
                    },
                ];
                store
                    .record(black_box("model"), black_box(1), "mmlu", metrics, true)
                    .unwrap();
            },
        );
    });

    group.bench_function("get_runs", |b| {
        b.iter_with_setup(
            || {
                let tmp = tempdir().unwrap();
                let store = EvalStore::new(tmp.path()).unwrap();
                for i in 0..20 {
                    let metrics = vec![MetricResult {
                        name: "accuracy".into(),
                        value: 0.8 + (i as f64 * 0.005),
                        unit: "score".into(),
                    }];
                    store.record("model", i % 5, "mmlu", metrics, true).unwrap();
                }
                (tmp, store)
            },
            |(_tmp, store)| {
                black_box(store.get_runs("model", None).unwrap());
            },
        );
    });

    group.bench_function("suites", |b| {
        b.iter_with_setup(
            || {
                let tmp = tempdir().unwrap();
                let store = EvalStore::new(tmp.path()).unwrap();
                for suite in &["mmlu", "hellaswag", "arc", "winogrande", "truthfulqa"] {
                    let metrics = vec![MetricResult {
                        name: "accuracy".into(),
                        value: 0.85,
                        unit: "score".into(),
                    }];
                    store.record("model", 1, suite, metrics, true).unwrap();
                }
                (tmp, store)
            },
            |(_tmp, store)| {
                black_box(store.suites().unwrap());
            },
        );
    });

    group.finish();
}

fn bench_backup_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("backup_manager");

    group.bench_function("set_schedule", |b| {
        b.iter_with_setup(
            || {
                let tmp = tempdir().unwrap();
                let out = tempdir().unwrap();
                let mgr = BackupManager::new(tmp.path()).unwrap();
                (tmp, out, mgr)
            },
            |(_tmp, out, mgr)| {
                mgr.set_schedule("nightly", BackupFrequency::Daily, 7, out.path().to_path_buf())
                    .unwrap();
            },
        );
    });

    group.bench_function("list_schedules", |b| {
        b.iter_with_setup(
            || {
                let tmp = tempdir().unwrap();
                let out = tempdir().unwrap();
                let mgr = BackupManager::new(tmp.path()).unwrap();
                for i in 0..5 {
                    mgr.set_schedule(
                        &format!("sched-{i}"),
                        BackupFrequency::Daily,
                        7,
                        out.path().to_path_buf(),
                    )
                    .unwrap();
                }
                (tmp, out, mgr)
            },
            |(_tmp, _out, mgr)| {
                black_box(mgr.list_schedules().unwrap());
            },
        );
    });

    group.finish();
}

fn bench_vault_registry(c: &mut Criterion) {
    let mut group = c.benchmark_group("vault_registry");

    group.bench_function("register", |b| {
        b.iter_with_setup(
            || {
                let tmp = tempdir().unwrap();
                let reg = VaultRegistry::new(tmp.path()).unwrap();
                (tmp, reg)
            },
            |(_tmp, reg)| {
                reg.register(
                    black_box("vault1"),
                    "/data/vault1".into(),
                    Some("test".into()),
                )
                .unwrap();
            },
        );
    });

    group.bench_function("list_10_vaults", |b| {
        b.iter_with_setup(
            || {
                let tmp = tempdir().unwrap();
                let reg = VaultRegistry::new(tmp.path()).unwrap();
                for i in 0..10 {
                    reg.register(
                        &format!("vault-{i}"),
                        format!("/data/vault-{i}").into(),
                        None,
                    )
                    .unwrap();
                }
                (tmp, reg)
            },
            |(_tmp, reg)| {
                black_box(reg.list().unwrap());
            },
        );
    });

    group.bench_function("activate_deactivate", |b| {
        b.iter_with_setup(
            || {
                let tmp = tempdir().unwrap();
                let reg = VaultRegistry::new(tmp.path()).unwrap();
                reg.register("vault1", "/data/vault1".into(), None).unwrap();
                (tmp, reg)
            },
            |(_tmp, reg)| {
                reg.activate(black_box("vault1")).unwrap();
                reg.deactivate().unwrap();
            },
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_quant_profile_store,
    bench_eval_store,
    bench_backup_manager,
    bench_vault_registry,
);
criterion_main!(benches);
