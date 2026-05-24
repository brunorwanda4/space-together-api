CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE users (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  name TEXT NOT NULL,
  email TEXT NOT NULL,
  username TEXT,
  password_hash TEXT,
  role TEXT,
  image TEXT,
  image_id TEXT,
  phone TEXT,
  gender TEXT,
  disabled BOOLEAN NOT NULL DEFAULT false,
  current_school_id TEXT CHECK (current_school_id IS NULL OR char_length(current_school_id) = 24),
  current_school_user_id TEXT CHECK (current_school_user_id IS NULL OR char_length(current_school_user_id) = 24),
  schools JSONB NOT NULL DEFAULT '[]'::jsonb,
  accessible_classes JSONB NOT NULL DEFAULT '[]'::jsonb,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX users_email_lower_unique ON users (lower(email));
CREATE UNIQUE INDEX users_username_lower_unique ON users (lower(username)) WHERE username IS NOT NULL;
CREATE INDEX users_current_school_id_idx ON users (current_school_id);
CREATE INDEX users_role_idx ON users (role);
CREATE TRIGGER users_set_updated_at BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE schools (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  creator_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  username TEXT NOT NULL,
  name TEXT NOT NULL,
  code TEXT,
  logo TEXT,
  logo_id TEXT,
  description TEXT,
  school_type TEXT,
  affiliation TEXT,
  database_name TEXT,
  is_active BOOLEAN NOT NULL DEFAULT true,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX schools_username_lower_unique ON schools (lower(username));
CREATE UNIQUE INDEX schools_code_lower_unique ON schools (lower(code)) WHERE code IS NOT NULL;
CREATE INDEX schools_creator_id_idx ON schools (creator_id);
CREATE TRIGGER schools_set_updated_at BEFORE UPDATE ON schools
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE school_memberships (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  school_user_id TEXT CHECK (school_user_id IS NULL OR char_length(school_user_id) = 24),
  member_type TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',
  joined_at TIMESTAMPTZ,
  ended_at TIMESTAMPTZ,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (school_id, user_id, member_type)
);

CREATE INDEX school_memberships_user_id_idx ON school_memberships (user_id);
CREATE INDEX school_memberships_school_status_idx ON school_memberships (school_id, status);
CREATE TRIGGER school_memberships_set_updated_at BEFORE UPDATE ON school_memberships
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE user_role_assignments (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  school_id TEXT REFERENCES schools(id) ON DELETE CASCADE,
  role TEXT NOT NULL,
  scope TEXT NOT NULL DEFAULT 'global',
  assigned_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  starts_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX user_role_assignments_user_scope_idx ON user_role_assignments (user_id, scope);
CREATE INDEX user_role_assignments_school_role_idx ON user_role_assignments (school_id, role);
CREATE TRIGGER user_role_assignments_set_updated_at BEFORE UPDATE ON user_role_assignments
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE student_profiles (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  user_id TEXT UNIQUE REFERENCES users(id) ON DELETE SET NULL,
  national_id TEXT,
  name TEXT NOT NULL,
  email TEXT,
  phone TEXT,
  gender TEXT,
  image TEXT,
  image_id TEXT,
  date_of_birth JSONB,
  merge_confidence TEXT NOT NULL DEFAULT 'source',
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX student_profiles_user_id_idx ON student_profiles (user_id);
CREATE INDEX student_profiles_email_lower_idx ON student_profiles (lower(email)) WHERE email IS NOT NULL;
CREATE INDEX student_profiles_phone_idx ON student_profiles (phone) WHERE phone IS NOT NULL;
CREATE INDEX student_profiles_national_id_idx ON student_profiles (national_id) WHERE national_id IS NOT NULL;
CREATE TRIGGER student_profiles_set_updated_at BEFORE UPDATE ON student_profiles
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE classes (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  creator_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  parent_class_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  username TEXT,
  code TEXT,
  level TEXT,
  is_active BOOLEAN NOT NULL DEFAULT true,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX classes_school_username_unique ON classes (school_id, lower(username)) WHERE username IS NOT NULL;
CREATE INDEX classes_school_id_idx ON classes (school_id);
CREATE INDEX classes_parent_class_id_idx ON classes (parent_class_id);
CREATE TRIGGER classes_set_updated_at BEFORE UPDATE ON classes
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE student_school_enrollments (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  student_id TEXT NOT NULL REFERENCES student_profiles(id) ON DELETE CASCADE,
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  class_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
  subclass_id TEXT REFERENCES classes(id) ON DELETE SET NULL,
  source_student_id TEXT CHECK (source_student_id IS NULL OR char_length(source_student_id) = 24),
  registration_number TEXT,
  admission_year INTEGER,
  status TEXT NOT NULL DEFAULT 'active',
  is_active BOOLEAN NOT NULL DEFAULT true,
  started_at TIMESTAMPTZ,
  ended_at TIMESTAMPTZ,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  UNIQUE (school_id, source_student_id),
  UNIQUE (school_id, registration_number)
);

CREATE INDEX student_school_enrollments_student_id_idx ON student_school_enrollments (student_id);
CREATE INDEX student_school_enrollments_school_status_idx ON student_school_enrollments (school_id, status);
CREATE INDEX student_school_enrollments_class_id_idx ON student_school_enrollments (class_id);
CREATE TRIGGER student_school_enrollments_set_updated_at BEFORE UPDATE ON student_school_enrollments
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE student_record_permissions (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  student_id TEXT NOT NULL REFERENCES student_profiles(id) ON DELETE CASCADE,
  owner_school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  viewer_school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  scope TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  requested_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  granted_by TEXT REFERENCES users(id) ON DELETE SET NULL,
  requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  granted_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (student_id, owner_school_id, viewer_school_id, scope)
);

CREATE INDEX student_record_permissions_viewer_idx ON student_record_permissions (viewer_school_id, status);
CREATE INDEX student_record_permissions_owner_idx ON student_record_permissions (owner_school_id, status);
CREATE TRIGGER student_record_permissions_set_updated_at BEFORE UPDATE ON student_record_permissions
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE class_subjects (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  class_id TEXT NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
  teacher_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  subject_id TEXT CHECK (subject_id IS NULL OR char_length(subject_id) = 24),
  name TEXT NOT NULL,
  code TEXT,
  category TEXT,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX class_subjects_school_id_idx ON class_subjects (school_id);
CREATE INDEX class_subjects_class_id_idx ON class_subjects (class_id);
CREATE INDEX class_subjects_teacher_user_id_idx ON class_subjects (teacher_user_id);
CREATE TRIGGER class_subjects_set_updated_at BEFORE UPDATE ON class_subjects
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE parents (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT REFERENCES schools(id) ON DELETE CASCADE,
  user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  student_id TEXT REFERENCES student_profiles(id) ON DELETE CASCADE,
  relationship TEXT,
  status TEXT NOT NULL DEFAULT 'active',
  permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX parents_user_id_idx ON parents (user_id);
CREATE INDEX parents_student_id_idx ON parents (student_id);
CREATE INDEX parents_school_id_idx ON parents (school_id);
CREATE TRIGGER parents_set_updated_at BEFORE UPDATE ON parents
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE teachers (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  status TEXT NOT NULL DEFAULT 'active',
  department TEXT,
  job_title TEXT,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX teachers_school_id_idx ON teachers (school_id);
CREATE INDEX teachers_user_id_idx ON teachers (user_id);
CREATE TRIGGER teachers_set_updated_at BEFORE UPDATE ON teachers
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE school_staff (
  id TEXT PRIMARY KEY CHECK (char_length(id) = 24),
  school_id TEXT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
  user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
  status TEXT NOT NULL DEFAULT 'active',
  department TEXT,
  job_title TEXT,
  raw_document JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX school_staff_school_id_idx ON school_staff (school_id);
CREATE INDEX school_staff_user_id_idx ON school_staff (user_id);
CREATE TRIGGER school_staff_set_updated_at BEFORE UPDATE ON school_staff
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
