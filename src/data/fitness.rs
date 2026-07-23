//! Fitness archive database IO: the snapshot's three full-table loads and
//! the import write path.
//!
//! The import transaction is a port of the Worker's single `D1.batch`:
//! insert-if-missing workouts and exercises, replace tags only for
//! exercises whose sorted tag signature changed, insert only new sets,
//! bump the version once. A `(workout_id, ordinal)` collision with an
//! already-stored set aborts the whole transaction — a conflicting
//! ordinal is a hard error, exactly as documented.

use std::collections::{HashMap, HashSet};

use toasty::Db;
use toasty::stmt::{List, Query};

use super::models::{Exercise, ExerciseTag, FitnessMeta, LiftSet, Workout};
use crate::fitness::import::{IncomingTag, Payload, tag_signature};

/// The data version; 0 when the row does not exist yet.
pub async fn current_version(db: &Db) -> toasty::Result<i64> {
    let mut db = db.clone();
    let row = FitnessMeta::filter_by_k("version")
        .first()
        .exec(&mut db)
        .await?;
    Ok(row.map(|meta| meta.v).unwrap_or(0))
}

/// Everything the snapshot needs, in three loads. Row order is
/// irrelevant — the snapshot sorts.
pub async fn load_archive(
    db: &Db,
) -> toasty::Result<(Vec<Workout>, Vec<LiftSet>, Vec<ExerciseTag>)> {
    let mut handle = db.clone();
    let workouts = Workout::all().exec(&mut handle).await?;
    let sets = LiftSet::all().exec(&mut handle).await?;
    let tags = ExerciseTag::all().exec(&mut handle).await?;
    Ok((workouts, sets, tags))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportOutcome {
    pub received: usize,
    pub added: usize,
    pub skipped: usize,
    pub version: i64,
    /// Whether anything was written (sets added or tags replaced) — the
    /// caller rebuilds the snapshot only then.
    pub mutated: bool,
}

pub async fn apply_import(
    db: &Db,
    payload: &Payload,
    imported_at: i64,
) -> toasty::Result<ImportOutcome> {
    let handle = db.clone();

    let set_ids: Vec<String> = payload.sets.iter().map(|set| set.id.clone()).collect();
    let existing_sets: HashSet<String> = {
        let mut db = handle.clone();
        Query::<List<LiftSet>>::all()
            .filter(LiftSet::fields().id().in_list(set_ids))
            .select(LiftSet::fields().id())
            .exec(&mut db)
            .await?
            .into_iter()
            .collect()
    };
    let candidates: Vec<_> = payload
        .sets
        .iter()
        .filter(|set| !existing_sets.contains(&set.id))
        .collect();

    let exercise_names: Vec<String> = payload
        .exercises
        .iter()
        .map(|exercise| exercise.name.clone())
        .collect();
    let stored_tags: HashMap<String, Vec<IncomingTag>> = {
        let mut db = handle.clone();
        let rows = ExerciseTag::filter(
            ExerciseTag::fields()
                .exercise_name()
                .in_list(exercise_names),
        )
        .exec(&mut db)
        .await?;
        let mut by_exercise: HashMap<String, Vec<IncomingTag>> = HashMap::new();
        for row in rows {
            by_exercise
                .entry(row.exercise_name)
                .or_default()
                .push(IncomingTag {
                    kind: row.kind,
                    value: row.value,
                });
        }
        by_exercise
    };
    let changed_exercises: HashSet<&str> = payload
        .exercises
        .iter()
        .filter(|exercise| {
            let stored = stored_tags
                .get(&exercise.name)
                .map(Vec::as_slice)
                .unwrap_or_default();
            tag_signature(&exercise.tags) != tag_signature(stored)
        })
        .map(|exercise| exercise.name.as_str())
        .collect();

    let received = payload.sets.len();
    if candidates.is_empty() && changed_exercises.is_empty() {
        return Ok(ImportOutcome {
            received,
            added: 0,
            skipped: received,
            version: current_version(&handle).await?,
            mutated: false,
        });
    }

    let mut db = handle.clone();
    let mut tx = db.transaction().await?;

    let workout_ids: Vec<String> = payload
        .workouts
        .iter()
        .map(|workout| workout.id.clone())
        .collect();
    let existing_workouts: HashSet<String> = Query::<List<Workout>>::all()
        .filter(Workout::fields().id().in_list(workout_ids))
        .select(Workout::fields().id())
        .exec(&mut tx)
        .await?
        .into_iter()
        .collect();
    let missing_workouts: Vec<_> = payload
        .workouts
        .iter()
        .filter(|workout| !existing_workouts.contains(&workout.id))
        .collect();
    if !missing_workouts.is_empty() {
        let mut create = Workout::create_many();
        for workout in missing_workouts {
            create = create.item(toasty::create!(Workout {
                id: workout.id.clone(),
                title: workout.title.clone(),
                raw_title: workout.raw_title.clone(),
                started_at_utc: workout.started_at_utc.clone(),
                started_at_local: workout.started_at_local.clone(),
                eastern_offset_minutes: workout.eastern_offset_minutes,
                duration_seconds: workout.duration_seconds,
                duration_suspicious: workout.duration_suspicious,
                notes: workout.notes.clone(),
                description: workout.description.clone(),
                source: workout.source.clone(),
                imported_at,
            }));
        }
        create.exec(&mut tx).await?;
    }

    let names: Vec<String> = payload
        .exercises
        .iter()
        .map(|exercise| exercise.name.clone())
        .collect();
    let existing_exercises: HashSet<String> = Query::<List<Exercise>>::all()
        .filter(Exercise::fields().name().in_list(names))
        .select(Exercise::fields().name())
        .exec(&mut tx)
        .await?
        .into_iter()
        .collect();
    let missing_exercises: Vec<_> = payload
        .exercises
        .iter()
        .filter(|exercise| !existing_exercises.contains(&exercise.name))
        .collect();
    if !missing_exercises.is_empty() {
        let mut create = Exercise::create_many();
        for exercise in missing_exercises {
            create = create.item(toasty::create!(Exercise {
                name: exercise.name.clone(),
            }));
        }
        create.exec(&mut tx).await?;
    }

    for exercise in &payload.exercises {
        if !changed_exercises.contains(exercise.name.as_str()) {
            continue;
        }
        Query::<List<ExerciseTag>>::all()
            .filter(
                ExerciseTag::fields()
                    .exercise_name()
                    .eq(exercise.name.clone()),
            )
            .delete()
            .exec(&mut tx)
            .await?;
        if !exercise.tags.is_empty() {
            let mut create = ExerciseTag::create_many();
            for tag in &exercise.tags {
                create = create.item(toasty::create!(ExerciseTag {
                    exercise_name: exercise.name.clone(),
                    kind: tag.kind.clone(),
                    value: tag.value.clone(),
                }));
            }
            create.exec(&mut tx).await?;
        }
    }

    if !candidates.is_empty() {
        let mut create = LiftSet::create_many();
        for set in &candidates {
            create = create.item(toasty::create!(LiftSet {
                id: set.id.clone(),
                workout_id: set.workout_id.clone(),
                exercise_name: set.exercise_name.clone(),
                raw_exercise_name: set.raw_exercise_name.clone(),
                ordinal: set.ordinal,
                exercise_note: set.exercise_note.clone(),
                superset_id: set.superset_id,
                weight_milli: set.weight_milli,
                weight_unit: set.weight_unit.clone(),
                reps: set.reps,
                effort_hundredths: set.effort_hundredths,
                distance_milli: set.distance_milli,
                set_time_seconds: set.set_time_seconds,
                set_type: set.set_type.clone(),
                incomplete: set.incomplete,
            }));
        }
        create.exec(&mut tx).await?;
    }

    match FitnessMeta::filter_by_k("version")
        .first()
        .exec(&mut tx)
        .await?
    {
        Some(meta) => {
            let next = meta.v + 1;
            let mut meta = meta;
            toasty::update!(meta { v: next }).exec(&mut tx).await?;
        }
        None => {
            toasty::create!(FitnessMeta {
                k: "version",
                v: 1i64,
            })
            .exec(&mut tx)
            .await?;
        }
    }
    tx.commit().await?;

    let added = candidates.len();
    Ok(ImportOutcome {
        received,
        added,
        skipped: received - added,
        version: current_version(&handle).await?,
        mutated: true,
    })
}
