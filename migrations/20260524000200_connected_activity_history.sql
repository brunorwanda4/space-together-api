CREATE TABLE conversations (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT REFERENCES schools(id) ON DELETE CASCADE,
  created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  conversation_type TEXT,
  title TEXT,
  participant_user_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
  participant_student_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
  last_message_at TIMESTAMPTZ,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX conversations_school_id_idx ON conversations (school_id);
CREATE INDEX conversations_last_message_at_idx ON conversations (last_message_at DESC);
CREATE TRIGGER conversations_set_updated_at BEFORE UPDATE ON conversations
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE conversation_keys (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  user_role TEXT,
  encrypted_key TEXT NOT NULL,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (conversation_id, user_id, user_role)
);

CREATE INDEX conversation_keys_user_id_idx ON conversation_keys (user_id);
CREATE TRIGGER conversation_keys_set_updated_at BEFORE UPDATE ON conversation_keys
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE messages (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  school_id TEXT REFERENCES schools(id) ON DELETE CASCADE,
  sender_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  sender_student_id TEXT REFERENCES student_profiles(id) ON DELETE SET NULL,
  recipient_user_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
  body TEXT,
  message_type TEXT,
  attachments JSONB NOT NULL DEFAULT '[]'::jsonb,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX messages_conversation_created_idx ON messages (conversation_id, created_at DESC);
CREATE INDEX messages_school_id_idx ON messages (school_id);
CREATE INDEX messages_sender_user_id_idx ON messages (sender_user_id);
CREATE TRIGGER messages_set_updated_at BEFORE UPDATE ON messages
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE assignments (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  class_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
  class_subject_id TEXT REFERENCES class_subjects(id) ON DELETE SET NULL,
  teacher_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  title TEXT NOT NULL,
  description TEXT,
  due_at TIMESTAMPTZ,
  status TEXT NOT NULL DEFAULT 'active',
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX assignments_school_id_idx ON assignments (school_id);
CREATE INDEX assignments_class_id_idx ON assignments (class_id);
CREATE INDEX assignments_class_subject_id_idx ON assignments (class_subject_id);
CREATE TRIGGER assignments_set_updated_at BEFORE UPDATE ON assignments
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE submissions (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  assignment_id TEXT NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
  student_id TEXT NOT NULL REFERENCES student_profiles(id) ON DELETE CASCADE,
  enrollment_id TEXT REFERENCES student_school_enrollments(id) ON DELETE SET NULL,
  submitted_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  status TEXT NOT NULL DEFAULT 'submitted',
  score NUMERIC,
  feedback TEXT,
  submitted_at TIMESTAMPTZ,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  UNIQUE (assignment_id, student_id)
);

CREATE INDEX submissions_school_id_idx ON submissions (school_id);
CREATE INDEX submissions_student_id_idx ON submissions (student_id);
CREATE INDEX submissions_enrollment_id_idx ON submissions (enrollment_id);
CREATE TRIGGER submissions_set_updated_at BEFORE UPDATE ON submissions
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE exams (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  class_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  term TEXT,
  academic_year TEXT,
  starts_at TIMESTAMPTZ,
  ends_at TIMESTAMPTZ,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX exams_school_id_idx ON exams (school_id);
CREATE INDEX exams_class_id_idx ON exams (class_id);
CREATE TRIGGER exams_set_updated_at BEFORE UPDATE ON exams
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE assessment_categories (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  class_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
  class_subject_id TEXT REFERENCES class_subjects(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  weight NUMERIC,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX assessment_categories_school_id_idx ON assessment_categories (school_id);
CREATE INDEX assessment_categories_class_subject_id_idx ON assessment_categories (class_subject_id);
CREATE TRIGGER assessment_categories_set_updated_at BEFORE UPDATE ON assessment_categories
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE scores (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  student_id TEXT NOT NULL REFERENCES student_profiles(id) ON DELETE CASCADE,
  enrollment_id TEXT REFERENCES student_school_enrollments(id) ON DELETE SET NULL,
  class_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
  class_subject_id TEXT REFERENCES class_subjects(id) ON DELETE SET NULL,
  exam_id TEXT REFERENCES exams(id) ON DELETE SET NULL,
  assessment_category_id TEXT REFERENCES assessment_categories(id) ON DELETE SET NULL,
  score NUMERIC,
  max_score NUMERIC,
  term TEXT,
  academic_year TEXT,
  recorded_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX scores_student_id_idx ON scores (student_id);
CREATE INDEX scores_school_student_idx ON scores (school_id, student_id);
CREATE INDEX scores_class_subject_id_idx ON scores (class_subject_id);
CREATE TRIGGER scores_set_updated_at BEFORE UPDATE ON scores
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE attendance (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  student_id TEXT NOT NULL REFERENCES student_profiles(id) ON DELETE CASCADE,
  enrollment_id TEXT REFERENCES student_school_enrollments(id) ON DELETE SET NULL,
  class_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
  date DATE NOT NULL,
  status TEXT NOT NULL,
  recorded_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  note TEXT,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  UNIQUE (school_id, student_id, date)
);

CREATE INDEX attendance_student_date_idx ON attendance (student_id, date DESC);
CREATE INDEX attendance_school_date_idx ON attendance (school_id, date DESC);
CREATE TRIGGER attendance_set_updated_at BEFORE UPDATE ON attendance
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE student_term_results (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  student_id TEXT NOT NULL REFERENCES student_profiles(id) ON DELETE CASCADE,
  enrollment_id TEXT REFERENCES student_school_enrollments(id) ON DELETE SET NULL,
  class_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
  term TEXT NOT NULL,
  academic_year TEXT NOT NULL,
  total_score NUMERIC,
  average_score NUMERIC,
  rank INTEGER,
  status TEXT,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  UNIQUE (school_id, student_id, term, academic_year)
);

CREATE INDEX student_term_results_student_idx ON student_term_results (student_id, academic_year, term);
CREATE INDEX student_term_results_school_idx ON student_term_results (school_id, academic_year, term);
CREATE TRIGGER student_term_results_set_updated_at BEFORE UPDATE ON student_term_results
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE finance (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  student_id TEXT REFERENCES student_profiles(id) ON DELETE SET NULL,
  enrollment_id TEXT REFERENCES student_school_enrollments(id) ON DELETE SET NULL,
  user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  finance_type TEXT,
  amount NUMERIC,
  currency TEXT,
  status TEXT,
  due_at TIMESTAMPTZ,
  paid_at TIMESTAMPTZ,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX finance_school_id_idx ON finance (school_id);
CREATE INDEX finance_student_id_idx ON finance (student_id);
CREATE INDEX finance_user_id_idx ON finance (user_id);
CREATE TRIGGER finance_set_updated_at BEFORE UPDATE ON finance
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE learning_materials (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  class_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
  class_subject_id TEXT REFERENCES class_subjects(id) ON DELETE SET NULL,
  uploaded_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  title TEXT NOT NULL,
  description TEXT,
  material_type TEXT,
  url TEXT,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX learning_materials_school_id_idx ON learning_materials (school_id);
CREATE INDEX learning_materials_class_subject_id_idx ON learning_materials (class_subject_id);
CREATE TRIGGER learning_materials_set_updated_at BEFORE UPDATE ON learning_materials
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE audit_logs (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT REFERENCES schools(id) ON DELETE CASCADE,
  actor_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT,
  action TEXT NOT NULL,
  before_data JSONB,
  after_data JSONB,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX audit_logs_school_created_idx ON audit_logs (school_id, created_at DESC);
CREATE INDEX audit_logs_actor_created_idx ON audit_logs (actor_user_id, created_at DESC);
CREATE INDEX audit_logs_entity_idx ON audit_logs (entity_type, entity_id);
