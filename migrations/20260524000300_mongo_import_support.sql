CREATE TABLE mongo_import_runs (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  source_main_db_name TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'running',
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  finished_at TIMESTAMPTZ,
  summary JSONB NOT NULL DEFAULT '{}'::jsonb,
  error_message TEXT
);

CREATE TABLE mongo_id_map (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  import_run_id TEXT NOT NULL REFERENCES mongo_import_runs(id) ON DELETE CASCADE,
  source_database_name TEXT NOT NULL,
  source_collection_name TEXT NOT NULL,
  source_id TEXT NOT NULL,
  target_table_name TEXT NOT NULL,
  target_id TEXT NOT NULL,
  school_id TEXT REFERENCES schools(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (import_run_id, source_database_name, source_collection_name, source_id)
);

CREATE INDEX mongo_id_map_target_idx ON mongo_id_map (target_table_name, target_id);
CREATE INDEX mongo_id_map_school_idx ON mongo_id_map (school_id);

CREATE TABLE student_identity_review_items (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  import_run_id TEXT NOT NULL REFERENCES mongo_import_runs(id) ON DELETE CASCADE,
  source_database_name TEXT NOT NULL,
  source_student_id TEXT NOT NULL,
  candidate_student_id TEXT REFERENCES student_profiles(id) ON DELETE SET NULL,
  school_id TEXT REFERENCES schools(id) ON DELETE SET NULL,
  reason TEXT NOT NULL,
  evidence JSONB NOT NULL DEFAULT '{}'::jsonb,
  status TEXT NOT NULL DEFAULT 'pending',
  reviewed_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  reviewed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX student_identity_review_items_status_idx ON student_identity_review_items (status);
CREATE INDEX student_identity_review_items_run_idx ON student_identity_review_items (import_run_id);
