use crate::domain::class_timetable::{DayStructureConfig, PeriodType};
use rand::prelude::IndexedMutRandom;

use chrono::Weekday;
use mongodb::bson::oid::ObjectId;
use rand::seq::SliceRandom;
use rand::rng;

// Import your structs
use crate::domain::{
    class_subject::ClassSubject,
    class_timetable::{ClassTimetable, Period, WeekSchedule},
};

pub fn auto_generate_schedule(
    class_id: ObjectId,
    education_year_id: ObjectId,
    term_order: i32,
    mut subjects: Vec<ClassSubject>,
    start_time_str: String,                // e.g. "08:00"
    days_to_schedule: Vec<Weekday>,        // e.g. [Mon, Tue, Wed, Thu, Fri]
    day_template: Vec<DayStructureConfig>, // The structure from step 1
) -> Result<ClassTimetable, String> {
    // 1. ANALYZE CAPACITY
    // How many "Subject" slots do we have per day?
    let subject_slots_per_day = day_template
        .iter()
        .filter(|s| matches!(s.period_type, PeriodType::Subject))
        .count();

    let total_weekly_capacity = subject_slots_per_day * days_to_schedule.len();

    // 2. CALCULATE SUBJECT WEIGHTS
    // Use 'credits' preferably, fall back to 'estimated_hours', default to 1
    let total_credits: i32 = subjects
        .iter()
        .map(|s| s.credits.or(Some(s.estimated_hours)).unwrap_or(1))
        .sum();

    if total_credits == 0 {
        return Err("Cannot generate schedule: Total subject credits is 0".into());
    }

    // 3. FILL THE "SUBJECT DECK"
    // Create a list of all subject instances needed.
    // If Math has high credits, it appears many times in this list.
    let mut subject_deck: Vec<ObjectId> = Vec::new();

    for sub in &subjects {
        let weight = sub.credits.or(Some(sub.estimated_hours)).unwrap_or(1);

        // Formula: (Subject Weight / Total Weight) * Available Slots
        let raw_count = (weight as f32 / total_credits as f32) * total_weekly_capacity as f32;
        let count = raw_count.round() as usize;

        if let Some(sid) = sub.id {
            for _ in 0..count {
                subject_deck.push(sid);
            }
        }
    }

    // 4. BALANCING THE DECK
    // If rounding caused us to have too few or too many subjects, fix it.
    let mut rng = rng();

    // Fill if short
    while subject_deck.len() < total_weekly_capacity {
        // Pick a random subject to fill the gap
        if let Some(random_sub) = subjects.choose_mut(&mut rng) {
            if let Some(sid) = random_sub.id {
                subject_deck.push(sid);
            }
        }
    }
    // Trim if over
    while subject_deck.len() > total_weekly_capacity {
        subject_deck.pop();
    }

    // Shuffle strictly to ensure randomness before distribution
    subject_deck.shuffle(&mut rng);

    // 5. DISTRIBUTE INTO DAYS (Round Robin)
    // We do this to ensure Math isn't scheduled 5 times on Monday and 0 on Friday.
    // We create 'buckets' for each day.
    let mut day_buckets: Vec<Vec<ObjectId>> = vec![Vec::new(); days_to_schedule.len()];

    for (i, subject_id) in subject_deck.iter().enumerate() {
        let day_index = i % days_to_schedule.len();
        day_buckets[day_index].push(*subject_id);
    }

    // 6. BUILD THE ACTUAL SCHEDULE
    let mut weekly_schedule: Vec<WeekSchedule> = Vec::new();

    for (day_idx, day_name) in days_to_schedule.into_iter().enumerate() {
        // Get the subjects assigned to this day and shuffle them *again* // so the order within the day is random
        let mut daily_subjects = day_buckets[day_idx].clone();
        daily_subjects.shuffle(&mut rng);
        let mut daily_subj_iter = daily_subjects.into_iter();

        let mut periods: Vec<Period> = Vec::new();
        let mut current_offset_minutes = 0;
        let mut order_counter = 1;

        for slot in &day_template {
            let period_oid = ObjectId::new();
            let mut final_subject_id = None;
            let mut final_title = slot.title.clone();
            let mut final_type = slot.period_type.clone();

            // If this slot is meant for a Subject, pull one from our daily bucket
            if matches!(slot.period_type, PeriodType::Subject) {
                if let Some(sid) = daily_subj_iter.next() {
                    final_subject_id = Some(sid);
                } else {
                    // If we ran out of subjects for this day (rare), make it Free
                    final_type = PeriodType::Free;
                    final_title = Some("Free Period".to_string());
                }
            }

            // Create the Period Struct
            let p = Period {
                period_id: period_oid,
                r#type: final_type,
                order: order_counter,
                title: final_title,
                description: None,
                subject_id: final_subject_id,
                start_offset: current_offset_minutes,
                duration_minutes: slot.duration_minutes,
                enabled: Some(true),
            };

            periods.push(p);

            // Advance time
            current_offset_minutes += slot.duration_minutes;
            order_counter += 1;
        }

        // Add the day to the week
        weekly_schedule.push(WeekSchedule {
            day: day_name,
            is_holiday: false, // Default
            start_on: Some(start_time_str.clone()),
            periods,
        });
    }

    // 7. RETURN FINAL STRUCT
    Ok(ClassTimetable {
        id: None, // Will be created by MongoDB on insert
        class_id,
        education_year_id,
        term_order,
        weekly_schedule,
        disabled: Some(false),
        created_at: None,
        updated_at: None,
    })
}
