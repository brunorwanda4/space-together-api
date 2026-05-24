ALTER TABLE users DROP COLUMN IF EXISTS schools;
ALTER TABLE users DROP COLUMN IF EXISTS accessible_classes;
ALTER TABLE users DROP COLUMN IF EXISTS raw_document;
ALTER TABLE schools DROP COLUMN IF EXISTS raw_document;
ALTER TABLE school_memberships DROP COLUMN IF EXISTS raw_document;
ALTER TABLE user_role_assignments DROP COLUMN IF EXISTS raw_document;
ALTER TABLE student_profiles DROP COLUMN IF EXISTS raw_document;
ALTER TABLE classes DROP COLUMN IF EXISTS raw_document;
ALTER TABLE student_school_enrollments DROP COLUMN IF EXISTS raw_document;
ALTER TABLE student_record_permissions DROP COLUMN IF EXISTS raw_document;
ALTER TABLE class_subjects DROP COLUMN IF EXISTS raw_document;
ALTER TABLE parents DROP COLUMN IF EXISTS raw_document;
ALTER TABLE parents DROP COLUMN IF EXISTS permissions;
ALTER TABLE teachers DROP COLUMN IF EXISTS raw_document;
ALTER TABLE school_staff DROP COLUMN IF EXISTS raw_document;
ALTER TABLE conversations DROP COLUMN IF EXISTS participant_user_ids;
ALTER TABLE conversations DROP COLUMN IF EXISTS participant_student_ids;
ALTER TABLE conversations DROP COLUMN IF EXISTS raw_document;
ALTER TABLE conversation_keys DROP COLUMN IF EXISTS raw_document;
ALTER TABLE messages DROP COLUMN IF EXISTS recipient_user_ids;
ALTER TABLE messages DROP COLUMN IF EXISTS attachments;
ALTER TABLE messages DROP COLUMN IF EXISTS raw_document;
ALTER TABLE assignments DROP COLUMN IF EXISTS raw_document;
ALTER TABLE submissions DROP COLUMN IF EXISTS raw_document;
ALTER TABLE exams DROP COLUMN IF EXISTS raw_document;
ALTER TABLE assessment_categories DROP COLUMN IF EXISTS raw_document;
ALTER TABLE scores DROP COLUMN IF EXISTS raw_document;
ALTER TABLE attendance DROP COLUMN IF EXISTS raw_document;
ALTER TABLE student_term_results DROP COLUMN IF EXISTS raw_document;
ALTER TABLE finance DROP COLUMN IF EXISTS raw_document;
ALTER TABLE learning_materials DROP COLUMN IF EXISTS raw_document;
ALTER TABLE audit_logs DROP COLUMN IF EXISTS metadata;
ALTER TABLE audit_logs DROP COLUMN IF EXISTS raw_document;

CREATE TABLE IF NOT EXISTS parent_student_links (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  parent_id TEXT NOT NULL REFERENCES parents(id) ON DELETE CASCADE,
  parent_user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
  student_id TEXT NOT NULL REFERENCES student_profiles(id) ON DELETE CASCADE,
  school_id TEXT REFERENCES schools(id) ON DELETE CASCADE,
  relationship TEXT,
  can_view_academics BOOLEAN NOT NULL DEFAULT true,
  can_view_attendance BOOLEAN NOT NULL DEFAULT true,
  can_view_finance BOOLEAN NOT NULL DEFAULT true,
  can_message_school BOOLEAN NOT NULL DEFAULT true,
  status TEXT NOT NULL DEFAULT 'active',
  starts_at TIMESTAMPTZ,
  ends_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (parent_id, student_id, school_id)
);

CREATE INDEX IF NOT EXISTS parent_student_links_parent_user_idx ON parent_student_links (parent_user_id, status);
CREATE INDEX IF NOT EXISTS parent_student_links_student_idx ON parent_student_links (student_id, status);
CREATE TRIGGER parent_student_links_set_updated_at BEFORE UPDATE ON parent_student_links
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS education_years (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  curriculum_id TEXT CHECK (curriculum_id IS NULL OR char_length(curriculum_id) = 24),
  name TEXT NOT NULL,
  starts_on DATE,
  ends_on DATE,
  is_active BOOLEAN NOT NULL DEFAULT true,
  created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS education_years_school_active_idx ON education_years (school_id, is_active);
CREATE TRIGGER education_years_set_updated_at BEFORE UPDATE ON education_years
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS terms (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  education_year_id TEXT NOT NULL REFERENCES education_years(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  starts_on DATE NOT NULL,
  ends_on DATE NOT NULL,
  is_active BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (education_year_id, name)
);

CREATE INDEX IF NOT EXISTS terms_school_active_idx ON terms (school_id, is_active);
CREATE TRIGGER terms_set_updated_at BEFORE UPDATE ON terms
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS conversation_participants (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  school_id TEXT REFERENCES schools(id) ON DELETE CASCADE,
  user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
  student_id TEXT REFERENCES student_profiles(id) ON DELETE CASCADE,
  role TEXT,
  joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  left_at TIMESTAMPTZ,
  last_read_message_id TEXT CHECK (last_read_message_id IS NULL OR char_length(last_read_message_id) = 24),
  CHECK (user_id IS NOT NULL OR student_id IS NOT NULL),
  UNIQUE (conversation_id, user_id, student_id, role)
);

CREATE INDEX IF NOT EXISTS conversation_participants_user_idx ON conversation_participants (user_id, left_at);
CREATE INDEX IF NOT EXISTS conversation_participants_student_idx ON conversation_participants (student_id, left_at);

CREATE TABLE IF NOT EXISTS message_attachments (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  url TEXT NOT NULL,
  public_id TEXT,
  file_name TEXT,
  mime_type TEXT,
  size_bytes BIGINT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS message_attachments_message_idx ON message_attachments (message_id);

CREATE TABLE IF NOT EXISTS finance_records (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  student_id TEXT REFERENCES student_profiles(id) ON DELETE SET NULL,
  enrollment_id TEXT REFERENCES student_school_enrollments(id) ON DELETE SET NULL,
  user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  record_type TEXT NOT NULL,
  description TEXT,
  amount NUMERIC NOT NULL,
  currency TEXT NOT NULL DEFAULT 'RWF',
  status TEXT NOT NULL DEFAULT 'pending',
  due_at TIMESTAMPTZ,
  paid_at TIMESTAMPTZ,
  created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS finance_records_school_idx ON finance_records (school_id, status);
CREATE INDEX IF NOT EXISTS finance_records_student_idx ON finance_records (student_id, status);
CREATE TRIGGER finance_records_set_updated_at BEFORE UPDATE ON finance_records
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

INSERT INTO finance_records (
  id, school_id, student_id, enrollment_id, user_id, record_type, amount, currency,
  status, due_at, paid_at, created_at, updated_at, deleted_at
)
SELECT id, school_id, student_id, enrollment_id, user_id, COALESCE(finance_type, 'fee'),
       COALESCE(amount, 0), COALESCE(currency, 'RWF'), COALESCE(status, 'pending'),
       due_at, paid_at, created_at, updated_at, deleted_at
FROM finance
ON CONFLICT (id) DO NOTHING;

DROP TABLE IF EXISTS finance;

CREATE TABLE IF NOT EXISTS announcements (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT REFERENCES schools(id) ON DELETE CASCADE,
  class_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
  author_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  audience TEXT NOT NULL DEFAULT 'school',
  is_pinned BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS announcements_school_created_idx ON announcements (school_id, created_at DESC);
CREATE INDEX IF NOT EXISTS announcements_class_created_idx ON announcements (class_id, created_at DESC);
CREATE TRIGGER announcements_set_updated_at BEFORE UPDATE ON announcements
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS comments (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT REFERENCES schools(id) ON DELETE CASCADE,
  target_type TEXT NOT NULL,
  target_id TEXT NOT NULL CHECK (char_length(target_id) = 24),
  parent_comment_id TEXT REFERENCES comments(id) ON DELETE CASCADE,
  author_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  body TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS comments_target_idx ON comments (target_type, target_id, created_at);
CREATE INDEX IF NOT EXISTS comments_author_idx ON comments (author_user_id, created_at DESC);
CREATE TRIGGER comments_set_updated_at BEFORE UPDATE ON comments
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS likes (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT REFERENCES schools(id) ON DELETE CASCADE,
  target_type TEXT NOT NULL,
  target_id TEXT NOT NULL CHECK (char_length(target_id) = 24),
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (target_type, target_id, user_id)
);

CREATE INDEX IF NOT EXISTS likes_target_idx ON likes (target_type, target_id);
CREATE INDEX IF NOT EXISTS likes_user_idx ON likes (user_id);

CREATE TABLE IF NOT EXISTS roles (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT REFERENCES schools(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  code TEXT NOT NULL,
  description TEXT,
  is_system BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (school_id, code)
);

CREATE INDEX IF NOT EXISTS roles_school_idx ON roles (school_id);
CREATE TRIGGER roles_set_updated_at BEFORE UPDATE ON roles
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS permissions (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  code TEXT NOT NULL UNIQUE,
  description TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS role_permissions (
  role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  permission_id TEXT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
  PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS school_user_accessible_classes (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  class_id TEXT NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
  granted_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (school_id, user_id, class_id)
);

CREATE INDEX IF NOT EXISTS school_user_accessible_classes_user_idx ON school_user_accessible_classes (user_id);

CREATE TABLE IF NOT EXISTS score_audit_logs (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  score_id TEXT NOT NULL REFERENCES scores(id) ON DELETE CASCADE,
  changed_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  old_score NUMERIC,
  new_score NUMERIC,
  reason TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS score_audit_logs_score_idx ON score_audit_logs (score_id, created_at DESC);

ALTER TABLE audit_logs
  ADD COLUMN IF NOT EXISTS ip_address TEXT,
  ADD COLUMN IF NOT EXISTS user_agent TEXT;

CREATE INDEX IF NOT EXISTS school_memberships_school_user_idx ON school_memberships (school_id, user_id);
CREATE INDEX IF NOT EXISTS student_profiles_user_email_phone_idx ON student_profiles (user_id, lower(email), phone);
CREATE INDEX IF NOT EXISTS scores_history_idx ON scores (student_id, school_id, academic_year, term);
CREATE INDEX IF NOT EXISTS attendance_history_idx ON attendance (student_id, school_id, date DESC);
CREATE INDEX IF NOT EXISTS messages_sender_history_idx ON messages (sender_user_id, school_id, created_at DESC);
